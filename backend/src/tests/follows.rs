use crate::tests::test_utils::*;
use serde_json::json;
use assert_json_diff::{assert_json_include, assert_json_eq};
use crate::server;


#[async_std::test]
async fn following_another_user() {
    let test_db = TestDb::new().await;
    let mut server = server(test_db.db()).await;

    let tim_token = create_user_and_authenticate(&mut server, Some("tim".to_string())).await.token;
    create_user_and_authenticate(&mut server, Some("jim".to_string())).await.token;

    for username in &["tim", "jim"] {
        let (json, status, _) = get(&format!("/users/{}/followers", username)).send(&mut server).await;
        assert_eq!(status, 200);
        assert_json_eq!(json, json!({"data" : []}));

        let (json, status, _) = get(&format!("/users/{}/followers", username)).send(&mut server).await;
        assert_eq!(status, 200);
        assert_json_eq!(json, json!({"data" : []}));
    }

    let (json, status, _) = post("/users/jim/follow", 
        None::<()>,)
        .header("Authorization", format!("Bearer {}", tim_token))
        .send(&mut server).await;
    assert_eq!(status, 201);

    assert_json_include!(actual: json, expected: json!({"data" : null}));

    let (json, status, _) = get("/users/tim/following").send(&mut server).await;
    assert_eq!(status, 200);
    assert_json_include!(actual: json, expected: json!({"data" : [
            {
                "username" : "jim"
            }
        ]
    }));

    let (json, status, _) = get("/users/jim/following").send(&mut server).await;
    assert_eq!(status, 200);
    assert_json_include!(actual: json, expected: json!({"data" : []}));

    let (json, status, _) = get("/users/tim/followers").send(&mut server).await;
    assert_eq!(status, 200);
    assert_json_include!(actual: json, expected: json!({"data" : []}));

    let (json, status, _) = get("/users/jim/followers").send(&mut server).await;
    assert_eq!(status, 200);
    assert_json_include!(actual: json, expected: json!({"data" : [
            {
                "username" : "tim"
            }
        ]
    }));
}



#[async_std::test]
async fn follow_same_user_twice() {
    let test_db = TestDb::new().await;
    let mut server = server(test_db.db()).await;

    let tim_token = create_user_and_authenticate(&mut server, Some("tim".to_string()))
        .await
        .token;
    create_user_and_authenticate(&mut server, Some("jim".to_string())).await;

    let (_, status, _) = post("/users/jim/follow", None::<()>)
        .header("Authorization", format!("Bearer {}", tim_token))
        .send(&mut server)
        .await;
    assert_eq!(status, 201);

    let (json, status, _) = post("/users/jim/follow", None::<()>)
        .header("Authorization", format!("Bearer {}", tim_token))
        .send(&mut server)
        .await;
    assert_eq!(status, 409);
    assert_json_include!(
        actual: json,
        expected: json!({
            "error": {
                "message": "You cannot follow the same user twice",
            }
        })
    );
}

#[async_std::test]
async fn cannot_follow_self() {
    let test_db = TestDb::new().await;
    let mut server = server(test_db.db()).await;

    let bobs_token = create_user_and_authenticate(&mut server, Some("bob".to_string()))
        .await
        .token;

    let (json, status, _) = post("/users/bob/follow", None::<()>)
        .header("Authorization", format!("Bearer {}", bobs_token))
        .send(&mut server)
        .await;
    assert_eq!(status, 409);
    assert_json_include!(
        actual: json,
        expected: json!({
            "error": {
                "message": "You cannot follow yourself",
            }
        })
    );
}


