use crate::tests::test_utils::*;
use serde_json::json;
use assert_json_diff::assert_json_include;
use shared::payloads::{CreateUserPayload, LoginPayload};

use crate::server;

#[async_std::test]
async fn create_user_and_login() {
    let test_db = TestDb::new().await;
    let mut server = server(test_db.db()).await;

    let token = create_user_and_authenticate(&mut server, None).await.token;

    let (json, status, _) = get("/me")
        .header("Authorization", format!("Bearer {}", token))
        .send(&mut server).await;
    assert_eq!(status, 200);
    assert_json_include!(actual: json, expected: json!({
        "data" : {
            "username" : "Geoff"
        }
    }));


    let (json, status, _) = post("/users/Geoff/session", Some(LoginPayload { password : "123456".to_string()})).send(&mut server).await;
    assert_eq!(status, 201);
    assert_json_include!(actual: json, expected: json!({
        "data" : {
            "token" : token
        }
    }));
}

#[async_std::test]
async fn duplicate_username_not_allowed() {
    let test_db = TestDb::new().await;
    let mut server = server(test_db.db()).await;

    let username = "Geoff".to_string();

    let (_, status, _) = post("/users", 
        Some(CreateUserPayload {
            username: username.clone(),
            password: "123456".to_string(),
        })).send(&mut server).await;
    assert_eq!(status, 201);


    let (json, status, _) = post("/users", 
        Some(CreateUserPayload {
            username,
            password: "654321".to_string(),
        })).send(&mut server).await;
    assert_eq!(status, 409);
    assert_eq!(json, json!({
        "error" : {
            "status_code" : "409",
            "message" : "Submitted username already taken"
        }
    }));
}
