use crate::tests::test_utils::*;
use crate::server;

#[async_std::test]
async fn logging_out() {
    let test_db = TestDb::new().await;
    let mut server = server(test_db.db()).await;

    let token = create_user_and_authenticate(&mut server, Some("tim".to_string()))
        .await
        .token;

    let (_, status, _) = get("/me")
        .header("Authorization", format!("Bearer {}", token))
        .send(&mut server)
        .await;
    assert_eq!(status, 200);

    let (_, status, _) = delete("/users/bob/session")
        .header("Authorization", format!("Bearer {}", token))
        .send(&mut server)
        .await;
    assert_eq!(status, 200);

    let (_, status, _) = get("/me")
        .header("Authorization", format!("Bearer {}", token))
        .send(&mut server)
        .await;
    assert_eq!(status, 401);
}