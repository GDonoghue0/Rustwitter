use tide::{Request, Response};
use crate::State;
use sqlx::query_as;
use tide::http::{StatusCode, Error};
use lazy_static::lazy_static;
use regex::Regex;
use shared::responses::UserResponse;
use tide::http::headers::HeaderName;
use serde_json::Value;
use crate::responses::BuildApiResponse;

pub mod me;
pub mod users;
pub mod events;

fn get_header<'a>(header_key: &str, req: &'a Request<State>) -> Result<&'a str, Error> {
    let auth_header_key: HeaderName = header_key.parse()?;

    let header_value = (|| {
        let value = req.header(&auth_header_key)?.get(0)?;
        Some(value.as_str())
    })();
    match header_value {
        Some(value) => Ok(value),
        None => {
            return Err(Error::from_str(
                StatusCode::BadRequest,
                format!("Missing value for '{}' header", header_key),
            ))
        }
    }
}

pub(crate) async fn authenticate(req: &Request<State>) -> Result<UserResponse, Error> {
    let auth_token = get_auth_token(req)?;

    let db_pool = &req.state().db_pool;
    let user = query_as!(UserResponse, r#"
        select users.id, users.username
        from users
        inner join auth_tokens 
            on auth_tokens.user_id = users.id
            and auth_tokens.token = $1
        "#,
        auth_token)
    .fetch_optional(db_pool)
    .await?;

    user.ok_or_else(|| Error::from_str(StatusCode::Unauthorized, "Invalid auth token"))
}

pub(crate) fn get_auth_token(req: &Request<State>) -> Result<&str, Error> {
    let header_value = get_header("Authorization", req)?;

    let caps = match BEARER_TOKEN_REGEX.captures(header_value) {
        Some(caps) => caps,
        None => {
            return Err(Error::from_str(
                StatusCode::BadRequest,
                "Unable to parse Authorization header value",
            ))
        }
    };

    Ok(caps.get(1).expect("missing capture group").as_str())
}

pub(crate) fn empty_response() -> Result<Response, Error> {
    Ok(Value::Null.to_response(StatusCode::Ok))
}

pub fn something_went_wrong(status_code: StatusCode) -> tide::Error {
    tide::Error::from_str(status_code, "Something went wrong")
}

lazy_static! {
    static ref BEARER_TOKEN_REGEX: Regex = Regex::new("^Bearer (.*)$").unwrap() ;
}