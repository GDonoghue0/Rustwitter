use shared::{Login, Me, Timeline};
use tide::Server;
use tide::security::CorsMiddleware;
use sqlx::{Pool, PgPool};
use http_types::headers::HeaderValue;
use tide::security::Origin;
use tide::http::Method;
use tide::{Request, Response, Body, StatusCode};
use shared::{ApiEndpoint, GetUser, PostEvent, NoPayload, CreateUser};
use shared::payloads::{CreateEventPayload, LoginPayload, CreateUserPayload};
use async_trait::async_trait;

#[cfg(test)]
mod tests;
mod env;
mod responses;
mod endpoints;
mod middlewares;

#[async_std::main]
async fn main() -> tide::Result<()>{
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    // let transport = web3::transports::Http::new("https://mainnet.infura.io/v3/1bed5f7f726247469b45f078c2d559e5")?;
    // println!("transport = {:?}", transport);
    // let web3 = web3::Web3::new(transport);
    // println!("web3 = {:?}", web3);
    // let block_number = web3.eth().block_number().await?;

    // println!("block_number: {:?}", block_number);


    let db_pool = make_db_pool().await;
    let server = server(db_pool).await;

    server.listen("127.0.0.1:8080").await?;

    Ok(())
}

async fn server(db_pool: PgPool) -> Server<State> {
    let mut server: Server<State> = Server::with_state(State{db_pool});

    server.with(CorsMiddleware::new()
        .allow_methods("GET, POST, PUT, PATCH, DELETE, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::Any)
        .allow_credentials(true));
    server.with(middlewares::ErrResponseToJson);

    add_endpoint::<CreateUser>(&mut server);

    // server.at("/me").get(endpoints::me::get);
    add_endpoint::<Me>(&mut server);

    // server.at("/me/timeline").get(endpoints::me::timeline);
    add_endpoint::<Timeline>(&mut server);

    add_endpoint::<Login>(&mut server);
    server.at("/users/:username/session").delete(endpoints::users::logout);

    server.at("/users/:username/follow").post(endpoints::users::follow);

    server.at("/users/:username/following").get(endpoints::users::following);

    server.at("/users/:username/followers").get(endpoints::users::followers);

    // server.at("/users/:username").get(endpoints::users::get);
    add_endpoint::<GetUser>(&mut server);

    // server.at("/events").post(endpoints::events::create);
    add_endpoint::<PostEvent>(&mut server);

    server
}

async fn make_db_pool() -> PgPool {
    let db_url = std::env::var("DATABASE_URL").unwrap();
    Pool::connect(&db_url).await.unwrap()
}

#[derive(Debug, Clone)]
struct State{
    db_pool: PgPool,
}

#[async_trait]
trait BackendApiEndpoint: ApiEndpoint {
    async fn handler(req: Request<State>, payload: Self::Payload) -> tide::Result<(Self::Response, StatusCode)>;
}

#[async_trait]
trait GetRequestPayload: Sized {
    async fn get_payload(req: &mut Request<State>) -> tide::Result<Self>;
}

#[async_trait]
impl GetRequestPayload for NoPayload {
    async fn get_payload(_: &mut Request<State>) -> tide::Result<Self> {
        Ok(NoPayload)
    }
}

macro_rules! impl_get_request_payload {
    ($name:ident) => {
        #[async_trait]
        impl GetRequestPayload for $name {
            async fn get_payload(req: &mut Request<State>) -> tide::Result<Self> {
                req.body_json().await
            }
        }
    };
}

impl_get_request_payload!(CreateEventPayload);
impl_get_request_payload!(LoginPayload);
impl_get_request_payload!(CreateUserPayload);

fn add_endpoint<E>(server: &mut Server<State>)
where 
    E: 'static + BackendApiEndpoint,
    E::Payload: GetRequestPayload + Send,
{
    let mut route = server.at(<E::Url as shared::Url>::URL_SPEC);

    let handler = |mut req: Request<State>| async {
        let payload = E::Payload::get_payload(&mut req).await?;

        let (data, status) = E::handler(req, payload).await?;
        let mut resp = Response::new(status);
        let body = Body::from_json(&serde_json::json!({"data" : data}))?;
        resp.set_body(body);
        Ok(resp)
    };

    match E::METHOD {
        Method::Get => route.get(handler),
        Method::Post => route.post(handler),
        Method::Head => route.head(handler),
        Method::Put => route.put(handler),
        Method::Delete => route.options(handler),
        Method::Connect => route.connect(handler),
        Method::Options => route.options(handler),
        Method::Trace => route.trace(handler),
        Method::Patch => route.patch(handler),
        _ => todo!("Unknown HTTP type"),
    };
}




