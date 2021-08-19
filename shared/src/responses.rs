use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T
}

impl<T> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String
}

impl TokenResponse {
    pub fn new(token: &str) -> Self {
        Self { token : token.to_string() }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventResponse {
    pub id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostEventResponse {
    pub id: Option<Uuid>,
    pub content: Option<String>,
}