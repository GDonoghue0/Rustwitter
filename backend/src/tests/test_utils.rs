use std::collections::HashMap;
use tide::http::{Request, Method, Url, Response};
use tide::StatusCode;
use crate::State;
use crate::Server;
use sqlx::{PgConnection,Connection,Postgres,PgPool};
use serde::Serialize;
use serde_json::Value;
use shared::responses::{ApiResponse, TokenResponse};
use shared::payloads::CreateUserPayload;

pub use shared::payloads;
pub use shared::responses;

pub fn get(url: &str) -> TestRequest {
    TestRequest {
        url : url.to_string(),
        headers: HashMap::new(),
        kind: TestRequestKind::Get
     }
}

pub fn post<T: Serialize>(url: &str, body: Option<T>) -> TestRequest {
    let body = body.map(|body| {
      serde_json::to_value(body).unwrap()  
    });
    let kind = TestRequestKind::Post(body);
    TestRequest {
        url: url.to_string(),
        headers: HashMap::new(),
        kind,
     }
}

pub fn delete(url: &str) -> TestRequest {
    TestRequest {
        url: url.to_string(),
        headers: HashMap::new(),
        kind: TestRequestKind::Delete,
    }
}

#[derive(Debug)]
pub struct TestRequest {
    url: String,
    headers: HashMap<String, String>,
    kind: TestRequestKind,
}

#[derive(Debug)]
pub enum TestRequestKind {
    Get,
    Post(Option<Value>),
    Delete,
}

impl TestRequest {
    pub(crate) async fn send(self, server: &Server<State>) -> (Value, StatusCode, HashMap<String,String>) {
        let url = Url::parse(&format!("http://example.com{}", self.url)).unwrap();
        let mut req = match self.kind {
            TestRequestKind::Get => Request::new(Method::Get, url),
            TestRequestKind::Post(body) => {
                let mut req = Request::new(Method::Post, url);

                if let Some(body) = body {
                    req.set_body(body.to_string());
                    req.set_content_type("application/json".parse().unwrap());
                };
                req
            }
            TestRequestKind::Delete => Request::new(Method::Delete, url),
        };


        for (key, value) in &self.headers {
            req.append_header(key.as_str(), value);
        }

        let mut res: Response = server.respond(req).await.unwrap();
        let status = res.status();
        let headers = res
            .iter()
            .flat_map(|(key, values)| {
                values
                    .iter()
                    .map(move |value| (key.as_str().to_string(), value.as_str().to_string()))
            })
            .collect::<HashMap<_, _>>();
        let json = res.body_json::<Value>().await;

        (json.unwrap(), status, headers)
    }

    pub fn header(mut self, key: &str, value: impl ToString) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self 
    }
}

fn db_url() -> String {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    let rng = thread_rng();
    let suffix: String = rng.sample_iter(&Alphanumeric).take(16).map(char::from).collect();
    let db_url = std::env::var("DATABASE_URL").expect("Unable to find DATABASE_URL");
    format!("{}_{}", db_url, suffix)
}

fn parse_db_url(db_url: &str) -> (&str, &str) {
    let sep_pos = db_url.rfind("/").unwrap();
    let pg_conn = &db_url[..=sep_pos];
    let db_name = &db_url[sep_pos + 1..];
    (pg_conn, db_name)
}

async fn create_db(db_url: &str) {
    let (pg_conn, db_name) = parse_db_url(db_url);
    let mut conn = PgConnection::connect(pg_conn).await.unwrap();

    let sql = format!(r#"CREATE DATABASE "{}""#, &db_name);
    sqlx::query::<Postgres>(&sql).execute(&mut conn).await.unwrap();
} 

async fn drop_db(db_url: &str) {
    let (pg_conn, db_name) = parse_db_url(db_url);
    let mut conn = PgConnection::connect(pg_conn).await.unwrap();

    let sql = format!(
        r#"SELECT pg_terminate_backend(pg_stat_activity.pid)
        FROM pg_stat_activity
        WHERE pg_stat_activity.datname = '{db}'
        AND pid <> pg_backend_pid();"#,
        db = db_name
    );
    sqlx::query::<Postgres>(&sql).execute(&mut conn).await.unwrap();

    let sql = format!(r#"DROP DATABASE "{db}";"#, db = db_name);
    sqlx::query::<Postgres>(&sql).execute(&mut conn).await.unwrap();
}

async fn run_migrations(db_url: &str) {
    let (pg_conn, db_name) = parse_db_url(db_url);
    let mut conn = PgConnection::connect(&format!("{}/{}",pg_conn,db_name)).await.unwrap();

    let sql = async_std::fs::read_to_string("setup.sql").await.unwrap();
    for query in sql.split(';') {
        sqlx::query::<Postgres>(&query).execute(&mut conn).await.unwrap();
    }
}

pub struct TestDb {
    db_url: String,
    db_pool: Option<PgPool>,
}

impl TestDb {
    pub async fn new() -> Self {
        std::env::set_var("APP_ENV", "test");
        dotenv::dotenv().ok();
        pretty_env_logger::try_init().ok();

        let db_url = db_url();
        create_db(&db_url).await;
        run_migrations(&db_url).await;

        let db_pool = PgPool::connect(&db_url).await.unwrap();

        Self {db_url, db_pool: Some(db_pool),}
    }

    pub fn db(&self) -> PgPool {
        self.db_pool.clone().unwrap()
    }
}

impl Drop for TestDb {
    fn drop(&mut self) {
        let _ = self.db_pool.take();
        futures::executor::block_on(drop_db(&self.db_url));
    }
}


pub(crate) async fn create_user_and_authenticate(server: &mut Server<State>, username: Option<String>) -> TokenResponse {
    let (json, status, _) = post("/users", 
        Some(CreateUserPayload {
            username: username.unwrap_or_else(|| "Geoff".to_string()),
            password: "123456".to_string(),
        })).send(server).await;
    assert_eq!(status, 201);

    serde_json::from_value::<ApiResponse<TokenResponse>>(json)
        .unwrap()
        .data
}







