use http_types::Method;
use serde::{Serialize, de::DeserializeOwned};

pub mod responses;
pub mod payloads;

pub trait Url {
    const URL_SPEC: &'static str;

    fn url(&self) -> String;
}

pub trait ApiEndpoint {
    type Url: Url;
    const METHOD: Method;
    type Payload;
    type Response: Serialize + DeserializeOwned;
}

pub struct GetUser;

pub struct NoPayload;

impl ApiEndpoint for GetUser {
    type Url = GetUserUrl;
    const METHOD: Method = Method::Get;
    type Payload = NoPayload;
    type Response = responses::UserResponse;
}

pub struct GetUserUrl {
    pub username: String,
}

impl Url for GetUserUrl {
    const URL_SPEC: &'static str = "/users/:username";

    fn url(&self) -> String {
        format!("/users/{}", self.username)
    }
}

pub struct PostEvent;

impl ApiEndpoint for PostEvent {
    type Url = PostEventUrl;
    const METHOD: Method = Method::Post;
    type Payload = payloads::CreateEventPayload;
    type Response = responses::PostEventResponse;
}

pub struct PostEventUrl;

impl Url for PostEventUrl {
    const URL_SPEC: &'static str = "/events";

    fn url(&self) -> String {
        format!("/events")
    }
}

pub struct Me;

impl ApiEndpoint for Me {
    type Url = MeUrl;
    const METHOD: Method = Method::Get;
    type Payload = NoPayload;
    type Response = responses::UserResponse;
}

pub struct MeUrl;

impl Url for MeUrl {
    const URL_SPEC: &'static str = "/me";

    fn url(&self) -> String {
        format!("/me")
    }
}

pub struct Login;

impl ApiEndpoint for Login {
    type Url = LoginUrl;
    const METHOD: Method = Method::Post;
    type Payload = payloads::LoginPayload;
    type Response = responses::TokenResponse;
}

pub struct LoginUrl{
    pub username: String,
}

impl Url for LoginUrl {
    const URL_SPEC: &'static str = "/users/:username/session";

    fn url(&self) -> String {
        format!("/users/{}/session", self.username)
    }
}

pub struct CreateUser;

impl ApiEndpoint for CreateUser {
    type Url = CreateUserUrl;
    const METHOD: Method = Method::Post;
    type Payload = payloads::CreateUserPayload;
    type Response = responses::TokenResponse;
}

pub struct CreateUserUrl;

impl Url for CreateUserUrl {
    const URL_SPEC: &'static str = "/users";

    fn url(&self) -> String {
        format!("/users")
    }
}

pub struct Timeline;

impl ApiEndpoint for Timeline {
    type Url = TimelineUrl;
    const METHOD: Method = Method::Get;
    type Payload = NoPayload;
    type Response = Vec<responses::EventResponse>;
}

pub struct TimelineUrl;

impl Url for TimelineUrl {
    const URL_SPEC: &'static str = "/me/timeline";

    fn url(&self) -> String {
        format!("/me/timeline")
    }
}

