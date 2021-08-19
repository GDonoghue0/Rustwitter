use crate::tests::test_utils::*;
use crate::{State, Server};
use serde_json::json;
use assert_json_diff::{assert_json_include};
use crate::server;
use shared::payloads::CreateEventPayload;

#[async_std::test]
async fn sees_own_events() {
    let test_db = TestDb::new().await;
    let mut server = server(test_db.db()).await;

    let token = create_user_and_authenticate(&mut server, None).await.token;

    post_event("oldest", &token, &mut server).await;
    post_event("middle", &token, &mut server).await;
    post_event("newest", &token, &mut server).await;

    let (json, status, _) = get("/me/timeline")
    .header("Authorization", format!("Bearer {}", token))
    .send(&mut server).await;

    assert_eq!(status, 200);
    assert_json_include!(actual: json, expected: json!({
        "data" : [
            {"content" : "newest"},
            {"content" : "middle"},
            {"content" : "oldest"},
        ]
    }));
}


#[async_std::test]
async fn sees_events_from_following() {
    let test_db = TestDb::new().await;
    let mut server = server(test_db.db()).await;

    let tim_token = create_user_and_authenticate(&mut server, Some("tim".to_string())).await.token;
    let jim_token = create_user_and_authenticate(&mut server, Some("jim".to_string())).await.token;

    post_event("oldest", &jim_token, &mut server).await;
    post_event("middle", &jim_token, &mut server).await;
    post_event("newest", &jim_token, &mut server).await;

    let (_, status, _) = post("/users/jim/follow", 
        None::<()>,)
        .header("Authorization", format!("Bearer {}", tim_token))
        .send(&mut server).await;
    assert_eq!(status, 201);

    let (json, status, _) = get("/me/timeline")
    .header("Authorization", format!("Bearer {}", jim_token))
    .send(&mut server).await;

    assert_eq!(status, 200);
    assert_json_include!(actual: json, expected: json!({
        "data" : [
            {"content" : "newest"},
            {"content" : "middle"},
            {"content" : "oldest"},
        ]
    }));
}

#[async_std::test]
async fn pagination() {
    let test_db = TestDb::new().await;
    let mut server = server(test_db.db()).await;

    let token = create_user_and_authenticate(&mut server, Some("tim".to_string())).await.token;

    post_event("5", &token, &mut server).await;
    post_event("4", &token, &mut server).await;
    post_event("3", &token, &mut server).await;
    post_event("2", &token, &mut server).await;
    post_event("1", &token, &mut server).await;


    let (json, status, _) = get("/me/timeline?page=1&page_size=2")
    .header("Authorization", format!("Bearer {}", token))
    .send(&mut server).await;

    assert_eq!(status, 200);
    assert_json_include!(actual: &json, expected: json!({
        "data" : [
            {"content" : "1"},
            {"content" : "2"},
        ]
    }));

    assert_eq!(json["data"].as_array().unwrap().len(), 2);

    let (json, status, _) = get("/me/timeline?page=2&page_size=2")
        .header("Authorization", format!("Bearer {}", token))
        .send(&mut server)
        .await;

    assert_eq!(status, 200);
    assert_json_include!(
        actual: &json,
        expected: json!({
            "data": [
                { "content": "3" },
                { "content": "4" },
            ]
        })
    );
    assert_eq!(json["data"].as_array().unwrap().len(), 2);

    let (json, status, _) = get("/me/timeline?page=3&page_size=2")
        .header("Authorization", format!("Bearer {}", token))
        .send(&mut server)
        .await;

    assert_eq!(status, 200);
    assert_json_include!(
        actual: &json,
        expected: json!({
            "data": [
                { "content": "5" },
            ]
        })
    );
    assert_eq!(json["data"].as_array().unwrap().len(), 1);

}

#[async_std::test]
async fn max_page_size() {
    let test_db = TestDb::new().await;
    let mut server = server(test_db.db()).await;

    let token = create_user_and_authenticate(&mut server, None).await.token;

    for _ in 0..21 {
        post_event("Hello", &token, &mut server).await;
    }

    let (json, status, _) = get("/me/timeline?page=1&page_size=100")
        .header("Authorization", format!("Bearer {}", token))
        .send(&mut server)
        .await;

    assert_eq!(status, 200);
    assert_eq!(json["data"].as_array().unwrap().len(), 20);
}


async fn post_event(text: &str, token: &str, server: &Server<State>) {
    post("/events", 
        Some(CreateEventPayload {
            content: text.to_string(),
        }))
    .header("Authorization", format!("Bearer {}", token))
    .send(server).await;
}

#[async_std::test]
async fn response_includes_user_who_posted_event() {
    let test_db = TestDb::new().await;
    let mut server = server(test_db.db()).await;

    let token = create_user_and_authenticate(&mut server, Some("bob".to_string()))
        .await
        .token;


    post_event("hello", &token, &server).await;
  

    let (json, status, _) = get("/me/timeline")
        .header("Authorization", format!("Bearer {}", token))
        .send(&mut server)
        .await;

    dbg!(&json);

    assert_eq!(status, 200);
    assert_json_include!(
        actual: json,
        expected: json!({
            "data": [
                {
                    "content": "hello",
                    "user": {
                        "username": "bob"
                    }
                },
            ]
        })
    );
}