use shared::{Timeline,Me};
use shared::responses::UserResponse;
use shared::NoPayload;
use shared::ApiEndpoint;
use tide::Request;
use tide::http::StatusCode;
use crate::State;
use crate::endpoints::authenticate;
use crate::BackendApiEndpoint;
use shared::responses::EventResponse;
use sqlx::query;
use serde::Deserialize;
use async_trait::async_trait;

// pub(crate) async fn get(req: Request<State>) -> tide::Result {
//     let user = authenticate(&req).await?;
//     Ok(user.to_response(StatusCode::Ok))
// }

#[async_trait]
impl BackendApiEndpoint for Me {
    async fn handler(req: Request<State>, _: NoPayload) -> tide::Result<(<Self as ApiEndpoint>::Response, StatusCode)> {
        let user = authenticate(&req).await?;
        Ok((user, StatusCode::Ok))
    }
}

#[async_trait]
impl BackendApiEndpoint for Timeline {
    async fn handler(req: Request<State>, _: NoPayload) -> tide::Result<(<Self as ApiEndpoint>::Response, StatusCode)> {
        let db_pool = &req.state().db_pool;
        let current_user = authenticate(&req).await?;

        let pagination = req.query::<Pagination>()?;
        let page_size = pagination.page_size.unwrap_or(20).min(20) as i64;

        let page = pagination.page.unwrap_or(1) as i64;
        let offset = (page - 1) * page_size;

        let events = query!(
            r#"
            select
                events.id as event_id
                , events.content as event_content
                , events.created_at as event_created_at
                , users.id as user_id
                , users.username as user_username
            from (
                select id, content, created_at, user_id
                from events
                where user_id = $1
                union all
                select events.id, events.content, events.created_at, events.user_id
                from users
                inner join follows on
                    follows.follower_id = $1
                    and follows.followed_id = users.id
                inner join events on
                    events.user_id = users.id
            ) events
            inner join users on users.id = events.user_id
            order by events.created_at desc
            limit $2
            offset $3
        "#,
            current_user.id,
            page_size,
            offset,
        )
        .fetch_all(db_pool)
        .await?;

        let event_responses = events
            .into_iter()
            .map(|event| EventResponse {
                id: event.event_id.unwrap(),
                content: event.event_content.unwrap(),
                created_at: event.event_created_at.unwrap(),
                user: UserResponse {
                    id: event.user_id,
                    username: event.user_username,
                },
            })
            .collect::<Vec<_>>();

        Ok((event_responses, StatusCode::Ok))
    }
}

#[derive(Debug, Deserialize)]
struct Pagination {
    page: Option<usize>,
    page_size: Option<usize>,
}