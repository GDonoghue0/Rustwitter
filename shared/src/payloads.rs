use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserPayload {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginPayload {
    pub password: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateEventPayload {
    pub content: String,
}