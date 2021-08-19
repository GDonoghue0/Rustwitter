use crate::tests::test_utils::*;
use serde_json::json;
use assert_json_diff::assert_json_include;
use crate::server;
use shared::payloads::CreateEventPayload;

#[async_std::test]
async fn make_valid_post() {
    let test_db = TestDb::new().await;
    let mut server = server(test_db.db()).await;

    let token = create_user_and_authenticate(&mut server, None).await.token;

    let (json, status, _) = post("/events",
        Some(CreateEventPayload {
            content: "Hello".to_string(),
        })).header("Authorization", format!("Bearer {}", token))
        .send(&mut server).await;

    assert_eq!(status, 201);
    assert_json_include!(actual: json, expected: json!({
        "data" : {
            "content" : "Hello"
        }
    }));
}

#[async_std::test]
async fn make_invalid_post() {
    let test_db = TestDb::new().await;
    let mut server = server(test_db.db()).await;

    let token = create_user_and_authenticate(&mut server, None).await.token;

    let text = std::iter::repeat('a').take(1000).collect::<String>();
    let (json, status, _) = post("/events", 
        Some(CreateEventPayload {
            content: text,
        })).header("Authorization", format!("Bearer {}", token))
        .send(&mut server).await;

    assert_eq!(status, 409);
    assert_json_include!(actual: json, expected: json!({
        "error" : {
            "message" : "content too long"
        }
    }));
}

#[async_std::test]
async fn invalid_data_gets_mapped_to_a_422() {
    let test_db = TestDb::new().await;
    let mut server = server(test_db.db()).await;

    let token = create_user_and_authenticate(&mut server, None).await.token;

    let (json, status, _) = post("/events", Some(json!({ "foo": "bar" })))
        .header("Authorization", format!("Bearer {}", token))
        .send(&mut server)
        .await;
    assert_eq!(status, 422);

    assert_json_include!(
        actual: json,
        expected: json!({
            "error": {
                "status_code": "422",
                "message": "missing field `content` at line 1 column 13"
            }
        })
    );
}