use crate::BackendApiEndpoint;
use tide::Request;
use shared::payloads::CreateEventPayload;
use shared::responses::PostEventResponse;
use shared::{ApiEndpoint, PostEvent};


use crate::State;
use crate::endpoints::authenticate;
use tide::http::StatusCode;
use chrono::Utc;
use uuid::Uuid;
use sqlx::query;
use async_trait::async_trait;


#[async_trait]
impl BackendApiEndpoint for PostEvent {
    async fn handler(req: Request<State>, create_event: CreateEventPayload) -> tide::Result<(<Self as ApiEndpoint>::Response, StatusCode)> {
        let db_pool = &req.state().db_pool;

        if create_event.content.len() > 200 {
            return Err(tide::Error::from_str(StatusCode::Conflict, "content too long"));
        }

        let user = authenticate(&req).await?;

        let now = Utc::now();
        let row = query!(
            r#"
                insert into events (id, user_id, content, created_at, updated_at)
                values ($1, $2, $3, $4, $5) returning id, content 
            "#,
            Uuid::new_v4(),
            user.id,
            create_event.content,
            now,
            now,
        ).fetch_one(db_pool).await?;

        Ok((PostEventResponse{
            id: Some(row.id),
            content: Some(row.content)
        }, StatusCode::Created))
    }
}