//! User management API integration tests.

use serde_json::json;

use crate::common::{TestServer, TestUser};

/// Helper: login and return an access token.
async fn login_as(server: &TestServer, user: &TestUser) -> String {
    let resp = server
        .client
        .post(server.url("/auth/login"))
        .json(&json!({
            "email": user.email,
            "password": user.password,
        }))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    body["tokens"]["access_token"]
        .as_str()
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn test_get_current_user() {
    let server = TestServer::spawn().await;
    let user = TestUser::create(&server.db, "admin").await;
    let token = login_as(&server, &user).await;

    let response = server
        .client
        .get(server.url("/users/me"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to get current user");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["email"], user.email);
    assert_eq!(body["name"], user.name);

    server.cleanup().await;
}

#[tokio::test]
async fn test_list_users_as_admin() {
    let server = TestServer::spawn().await;
    let admin = TestUser::create(&server.db, "admin").await;
    let _analyst = TestUser::create(&server.db, "analyst").await;
    let token = login_as(&server, &admin).await;

    let response = server
        .client
        .get(server.url("/users?page=1&per_page=10"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to list users");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["data"].is_array());
    assert!(body["meta"]["total"].as_i64().unwrap() >= 2);

    server.cleanup().await;
}

#[tokio::test]
async fn test_list_users_as_analyst_forbidden() {
    let server = TestServer::spawn().await;
    let analyst = TestUser::create(&server.db, "analyst").await;
    let token = login_as(&server, &analyst).await;

    let response = server
        .client
        .get(server.url("/users"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to send request");

    // Analysts cannot list all users
    assert_eq!(response.status(), 403);

    server.cleanup().await;
}

#[tokio::test]
async fn test_get_user_by_id() {
    let server = TestServer::spawn().await;
    let admin = TestUser::create(&server.db, "admin").await;
    let target = TestUser::create(&server.db, "analyst").await;
    let token = login_as(&server, &admin).await;

    let response = server
        .client
        .get(server.url(&format!("/users/{}", target.id)))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to get user by id");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["id"], target.id.to_string());
    assert_eq!(body["email"], target.email);

    server.cleanup().await;
}

#[tokio::test]
async fn test_update_user_role() {
    let server = TestServer::spawn().await;
    let admin = TestUser::create(&server.db, "admin").await;
    let user = TestUser::create(&server.db, "analyst").await;
    let token = login_as(&server, &admin).await;

    let response = server
        .client
        .put(server.url(&format!("/users/{}/role", user.id)))
        .bearer_auth(&token)
        .json(&json!({ "role": "admin" }))
        .send()
        .await
        .expect("Failed to update role");

    assert_eq!(response.status(), 200);

    // Verify role was updated
    let get_resp = server
        .client
        .get(server.url(&format!("/users/{}", user.id)))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = get_resp.json().await.unwrap();
    assert_eq!(body["role"], "admin");

    server.cleanup().await;
}

#[tokio::test]
async fn test_unauthenticated_request_rejected() {
    let server = TestServer::spawn().await;

    let response = server
        .client
        .get(server.url("/users/me"))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 401);

    server.cleanup().await;
}

#[tokio::test]
async fn test_invalid_token_rejected() {
    let server = TestServer::spawn().await;

    let response = server
        .client
        .get(server.url("/users/me"))
        .bearer_auth("invalid.jwt.token")
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 401);

    server.cleanup().await;
}
