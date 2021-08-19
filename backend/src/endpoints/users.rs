use shared::{GetUser, NoPayload, Login, ApiEndpoint, CreateUser};
use crate::BackendApiEndpoint;
use sqlx::PgPool;
use sqlx::{query, query_as};
use tide::{Error,Request};
use tide::http::StatusCode;
use argonautica::{Hasher, Verifier};
use rand::Rng;
use rand::rngs::OsRng;
use chrono::prelude::*;
use uuid::Uuid;
use rand::distributions::Alphanumeric;
use futures::compat::Compat01As03;
use failure::Fail;
use crate::State;
use crate::responses::BuildApiResponse;
use shared::responses::{TokenResponse, UserResponse};
use crate::env;
use shared::payloads::{CreateUserPayload, LoginPayload};
use crate::endpoints::{authenticate, empty_response, get_auth_token, something_went_wrong};
use async_trait::async_trait;


#[async_trait]
impl BackendApiEndpoint for CreateUser {
    async fn handler(req: Request<State>, create_user: CreateUserPayload,) -> tide::Result<(<Self as ApiEndpoint>::Response, StatusCode)> {
        let db_pool = &req.state().db_pool;


        let result = query!("select 1 as one from users where username = $1", create_user.username)
            .fetch_optional(db_pool)
            .await?;

        if result.is_some() {
            return Err(tide::Error::from_str(StatusCode::Conflict, "Submitted username already taken"));
        }

        let secret_key = std::env::var("SECRET_KEY")?;
        let clear_text_password = create_user.password.clone();

        let mut hasher = Hasher::default();
        if env::current().is_test() {
            hasher.configure_iterations(10);
        }

        let hashed_password = Compat01As03::new(
            hasher
            .with_password(clear_text_password)
            .with_secret_key(secret_key)
            .hash_non_blocking()
        ).await
        .map_err(|err| err.compat())?;

        let now = Utc::now();
        let row = query!(
            r#"
                insert into users (id, username, hashed_password, created_at, updated_at)
                values ($1, $2, $3, $4, $5) returning id
            "#,
            Uuid::new_v4(), 
            create_user.username,
            hashed_password,
            now,
            now,
        ).fetch_one(db_pool).await?;


        let raw_token: String = OsRng.sample_iter(&Alphanumeric).take(32).map(char::from).collect();
        let token = query!(
            r#"
                insert into auth_tokens (id, user_id, token, created_at, updated_at)
                values ($1, $2, $3, $4, $5) returning token
            "#,
            Uuid::new_v4(), 
            row.id,
            raw_token,
            now,
            now,
        ).fetch_one(db_pool).await?;


        Ok((TokenResponse::new(&token.token),StatusCode::Created))
    }
}

#[async_trait]
impl BackendApiEndpoint for Login {
    async fn handler(req: Request<State>, payload: LoginPayload) -> tide::Result<(<Self as ApiEndpoint>::Response, StatusCode)> {
        let db_pool = req.state().db_pool.clone();
        let username = req.param("username")?;
       
        let user = query!(
            r#"
                select id, hashed_password
                from users
                where username = $1
            "#,
            username
        )
        .fetch_optional(&db_pool)
        .await?;

        let password = payload.password;
        let user = match user {
            Some(user) => user,
            None => return Err(Error::from_str(StatusCode::NotFound, "User not found")),
        };
        let user_password = user.hashed_password;
        let secret_key = std::env::var("SECRET_KEY")?;

        let mut verifier = Verifier::default();
        let is_valid = Compat01As03::new(
        verifier
            .with_hash(user_password)
            .with_password(password)
            .with_secret_key(secret_key)
            .verify_non_blocking()
        ).await
        .map_err(|err| err.compat())?;

        if is_valid {
            let token_row = query!(
                r#"
                    select token
                    from auth_tokens
                    where user_id = $1
                "#,
                user.id
            ).fetch_one(&db_pool).await?;

            Ok((TokenResponse::new(&token_row.token),StatusCode::Created))
        } else {
            Err(something_went_wrong(StatusCode::Forbidden))
        }
    }
}

pub(crate) async fn follow(req: Request<State>) -> tide::Result {
    let db_pool = req.state().db_pool.clone();
    let current_user = authenticate(&req).await?;
    let username = req.param("username")?;

    let row = query!("select id from users where username = $1", username).fetch_optional(&db_pool).await?;

    let followed_id: Uuid = if let Some(row) = row {
        row.id
    } else {
        todo!();
    };

    if current_user.id == followed_id {
        return Err(tide::Error::from_str(StatusCode::Conflict, "You cannot follow yourself"));
    }

    if user_following(current_user.id, followed_id, &db_pool).await? {
        return Err(tide::Error::from_str(StatusCode::Conflict, "You cannot follow the same user twice"));
    }

    let now = Utc::now();
    let pg_res = query!(
        r#"
            insert into follows (id, follower_id, followed_id, created_at, updated_at)
            values ($1, $2, $3, $4, $5)
        "#,
        Uuid::new_v4(), 
        current_user.id,
        followed_id,
        now,
        now,
    ).execute(&db_pool).await?;

    if pg_res.rows_affected() == 1 {
        Ok(serde_json::Value::Null.to_response(StatusCode::Created))
    } else {
        todo!()
    }
}

pub(crate) async fn following(req: Request<State>) -> tide::Result {
    let db_pool = req.state().db_pool.clone();
    // let current_user = authenticate(&req).await?;
    let username = req.param("username")?;

    let row = query!("select id from users where username = $1", username).fetch_optional(&db_pool).await?;

    let user_id: Uuid = if let Some(row) = row {
        row.id
    } else {
        todo!();
    };

    let rows = query_as!(UserResponse,
        r#"
            select users.id, users.username
            from users
            inner join follows on follows.follower_id = $1
            and follows.followed_id = users.id
        "#, user_id).fetch_all(&db_pool).await?;

    Ok(rows.to_response(StatusCode::Ok))
}

pub(crate) async fn followers(req: Request<State>) -> tide::Result {
    let db_pool = req.state().db_pool.clone();
    let username = req.param("username")?;

    let row = query!("select id from users where username = $1", username).fetch_optional(&db_pool).await?;

    let user_id: Uuid = if let Some(row) = row {
        row.id
    } else {
        todo!();
    };

    let rows = query_as!(UserResponse,
        r#"
            select users.id, users.username
            from users
            inner join follows on follows.followed_id = $1
            and follows.follower_id = users.id
        "#, user_id).fetch_all(&db_pool).await?;

    Ok(rows.to_response(StatusCode::Ok))
}

async fn user_following(current_user_id: Uuid, followee_id: Uuid, db_pool: &PgPool,) -> tide::Result<bool> {
    let row = query!(
    r#"
        select 1 as one from follows
        where follower_id = $1 and followed_id = $2
    "#,
        current_user_id,
        followee_id
    )
    .fetch_optional(db_pool)
    .await?;

    Ok(row.is_some())
}

#[async_trait]
impl BackendApiEndpoint for GetUser {
    async fn handler(req: Request<State>, _: NoPayload) -> tide::Result<(<Self as ApiEndpoint>::Response, StatusCode)> {
        let db_pool = &req.state().db_pool;
        let username = req.param("username")?;


        let user = query_as!(UserResponse,
            r#"
                select id, username
                from users
                where username = $1
            "#,
            username
        )
        .fetch_optional(db_pool).await?;


        let resp = user.ok_or_else(|| tide::Error::from_str(StatusCode::NotFound, "User does not exist"))?;
        Ok((resp, StatusCode::Ok))

    }
}

pub(crate) async fn logout(req: Request<State>) -> tide::Result {
    authenticate(&req).await?;
    let auth_token = get_auth_token(&req)?;

    let db_pool = &req.state().db_pool;
    query!("delete from auth_tokens where token = $1", auth_token)
        .execute(db_pool)
        .await?;

    empty_response()
}


