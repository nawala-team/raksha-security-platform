//! Authentication API integration tests.

use serde_json::json;

use crate::common::{TestServer, TestUser};

#[tokio::test]
async fn test_login_with_valid_credentials() {
    let server = TestServer::spawn().await;
    let user = TestUser::create(&server.db, "admin").await;

    let response = server
        .client
        .post(server.url("/auth/login"))
        .json(&json!({
            "email": user.email,
            "password": user.password,
        }))
        .send()
        .await
        .expect("Failed to send login request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body.get("tokens").is_some());
    assert!(body["tokens"]["access_token"].is_string());
    assert!(body["tokens"]["refresh_token"].is_string());
    assert_eq!(body["user"]["email"], user.email);

    server.cleanup().await;
}

#[tokio::test]
async fn test_login_with_invalid_password() {
    let server = TestServer::spawn().await;
    let user = TestUser::create(&server.db, "analyst").await;

    let response = server
        .client
        .post(server.url("/auth/login"))
        .json(&json!({
            "email": user.email,
            "password": "wrong_password_123",
        }))
        .send()
        .await
        .expect("Failed to send login request");

    assert_eq!(response.status(), 401);

    server.cleanup().await;
}

#[tokio::test]
async fn test_login_with_nonexistent_user() {
    let server = TestServer::spawn().await;

    let response = server
        .client
        .post(server.url("/auth/login"))
        .json(&json!({
            "email": "nonexistent@test.raksha.dev",
            "password": "SomePassword123!",
        }))
        .send()
        .await
        .expect("Failed to send login request");

    assert_eq!(response.status(), 401);

    server.cleanup().await;
}

#[tokio::test]
async fn test_login_with_invalid_email_format() {
    let server = TestServer::spawn().await;

    let response = server
        .client
        .post(server.url("/auth/login"))
        .json(&json!({
            "email": "not-an-email",
            "password": "SomePassword123!",
        }))
        .send()
        .await
        .expect("Failed to send login request");

    // Should fail validation
    assert_eq!(response.status(), 422);

    server.cleanup().await;
}

#[tokio::test]
async fn test_login_with_short_password_rejected() {
    let server = TestServer::spawn().await;

    let response = server
        .client
        .post(server.url("/auth/login"))
        .json(&json!({
            "email": "user@test.raksha.dev",
            "password": "short",
        }))
        .send()
        .await
        .expect("Failed to send login request");

    // Password min length is 8
    assert_eq!(response.status(), 422);

    server.cleanup().await;
}

#[tokio::test]
async fn test_refresh_token_flow() {
    let server = TestServer::spawn().await;
    let user = TestUser::create(&server.db, "admin").await;

    // Login first to get tokens
    let login_resp = server
        .client
        .post(server.url("/auth/login"))
        .json(&json!({
            "email": user.email,
            "password": user.password,
        }))
        .send()
        .await
        .unwrap();

    let login_body: serde_json::Value = login_resp.json().await.unwrap();
    let refresh_token = login_body["tokens"]["refresh_token"].as_str().unwrap();

    // Use refresh token to get new access token
    let refresh_resp = server
        .client
        .post(server.url("/auth/refresh"))
        .json(&json!({
            "refresh_token": refresh_token,
        }))
        .send()
        .await
        .expect("Failed to send refresh request");

    assert_eq!(refresh_resp.status(), 200);

    let refresh_body: serde_json::Value = refresh_resp.json().await.unwrap();
    assert!(refresh_body["tokens"]["access_token"].is_string());

    server.cleanup().await;
}

#[tokio::test]
async fn test_logout_invalidates_session() {
    let server = TestServer::spawn().await;
    let user = TestUser::create(&server.db, "admin").await;

    // Login
    let login_resp = server
        .client
        .post(server.url("/auth/login"))
        .json(&json!({
            "email": user.email,
            "password": user.password,
        }))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = login_resp.json().await.unwrap();
    let access_token = body["tokens"]["access_token"].as_str().unwrap();

    // Logout
    let logout_resp = server
        .client
        .post(server.url("/auth/logout"))
        .bearer_auth(access_token)
        .send()
        .await
        .expect("Failed to send logout request");

    assert_eq!(logout_resp.status(), 200);

    // Attempting to use the same token should fail
    let protected_resp = server
        .client
        .get(server.url("/users/me"))
        .bearer_auth(access_token)
        .send()
        .await
        .expect("Failed to send protected request");

    assert_eq!(protected_resp.status(), 401);

    server.cleanup().await;
}
