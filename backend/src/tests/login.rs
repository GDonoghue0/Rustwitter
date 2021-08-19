use crate::tests::test_utils::*;
use serde_json::json;
use assert_json_diff::assert_json_include;
use shared::payloads::LoginPayload;

use crate::server;


#[async_std::test]
async fn authenticating_without_auth_header() {
    let test_db = TestDb::new().await;
    let mut server = server(test_db.db()).await;

    create_user_and_authenticate(&mut server, None).await;

    let (json, status, headers) = get("/me").send(&mut server).await;
    let content_type = &headers["content-type"];
    assert_eq!(content_type, "application/json");
    assert_eq!(status, 400);
    assert_json_include!(actual: json, expected: json!({
        "error" : {
            "message" : "Missing value for 'Authorization' header"
        }
    }));
}

#[async_std::test]
async fn authenticating_with_invalid_auth_header() {
    let test_db = TestDb::new().await;
    let mut server = server(test_db.db()).await;

    let token = create_user_and_authenticate(&mut server, None).await.token;

    let (_, status, _) = get("/me")
        .header("Authorization", format!("NotBearer {}", token))
        .send(&mut server).await;
    assert_eq!(status, 400);
}

#[async_std::test]
async fn login_unknown_user() {
    let test_db = TestDb::new().await;
    let mut server = server(test_db.db()).await;

    let (_, status, _) = post("/users/Geoff/session", Some(LoginPayload { password : "123456".to_string()})).send(&mut server).await;
    assert_eq!(status, 404);
}

#[async_std::test]
async fn login_invalid_password() {
    let test_db = TestDb::new().await;
    let mut server = server(test_db.db()).await;

    let username = "Geoff";
    create_user_and_authenticate(&mut server, Some(username.to_string())).await;

    let (json, status, _) = post(
        &format!("/users/{}/session", username),
        Some(LoginPayload {
            password: "654321".to_string(),
        }),
    ).send(&mut server).await;
    assert_eq!(status, 403);
    assert_json_include!(
        actual: json,
        expected: json!({
            "error": {
                "status_code": "403",
                "message": "Something went wrong",
            }
        }),
    );
}