use crate::tests::test_utils::*;
use crate::server;
use assert_json_diff::assert_json_include;
use serde_json::json;

#[async_std::test]
async fn get_other_profile() {
    let test_db = TestDb::new().await;
    let mut server = server(test_db.db()).await;

    let username = "tim";
    create_user_and_authenticate(&mut server, Some(username.to_string())).await.token;

    let (json, status, _) = get(&format!("/users/{}", username)).send(&mut server).await;
    assert_eq!(status, 200);

    assert_json_include!(actual: json, expected: json!({
        "data" : {
            "username" : "tim"
        }
    }))
}

#[async_std::test]
async fn get_nonexistent_profile() {
    let test_db = TestDb::new().await;
    let mut server = server(test_db.db()).await;

    let (json, status, _) = get(&format!("/users/jim")).send(&mut server).await;
    assert_eq!(status, 404);

    assert_json_include!(
        actual: json,
        expected: json!({
            "error": {
                "message": "User does not exist",
                "status_code": "404"
            }
        })
    );

}