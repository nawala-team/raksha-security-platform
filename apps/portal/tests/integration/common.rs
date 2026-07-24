//! Test utilities and shared helpers for integration tests.

use std::net::SocketAddr;
use std::sync::Once;

use axum::Router;
use reqwest::Client;
use sqlx::PgPool;
use tokio::net::TcpListener;
use uuid::Uuid;

static INIT: Once = Once::new();

/// Test configuration loaded from environment or defaults.
pub struct TestConfig {
    pub database_url: String,
    pub redis_url: String,
}

impl TestConfig {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://raksha:test_secret@localhost:5432/raksha_test".to_string()),
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        }
    }
}

/// A running test server instance with its address and HTTP client.
pub struct TestServer {
    pub addr: SocketAddr,
    pub client: Client,
    pub db: PgPool,
}

impl TestServer {
    /// Spawn the full application on a random port for testing.
    pub async fn spawn() -> Self {
        init_tracing();

        let config = TestConfig::from_env();

        // Create isolated test database pool
        let db = PgPool::connect(&config.database_url)
            .await
            .expect("Failed to connect to test database");

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&db)
            .await
            .expect("Failed to run migrations");

        // Build the app router (simplified for tests without full AppState)
        let app = create_test_router(db.clone()).await;

        // Bind to port 0 to get a random available port
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind test server");
        let addr = listener.local_addr().unwrap();

        // Spawn the server in the background
        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        });

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap();

        Self { addr, client, db }
    }

    /// Base URL for this test server.
    pub fn url(&self, path: &str) -> String {
        format!("http://{}/api/v1{}", self.addr, path)
    }

    /// Clean up test data after tests complete.
    pub async fn cleanup(&self) {
        sqlx::query("DELETE FROM users WHERE email LIKE '%@test.raksha.dev'")
            .execute(&self.db)
            .await
            .ok();
    }
}

/// Create a test user and return credentials.
pub struct TestUser {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub name: String,
}

impl TestUser {
    /// Insert a test user into the database and return the credentials.
    pub async fn create(db: &PgPool, role: &str) -> Self {
        let id = Uuid::new_v4();
        let email = format!("test-{}@test.raksha.dev", Uuid::new_v4());
        let password = "TestPassword123!".to_string();
        let name = format!("Test User {}", &id.to_string()[..8]);

        // Hash password with argon2
        let password_hash = hash_password(&password);

        sqlx::query(
            r#"
            INSERT INTO users (id, email, name, password_hash, role, is_active, created_at)
            VALUES ($1, $2, $3, $4, $5::user_role, true, NOW())
            "#,
        )
        .bind(id)
        .bind(&email)
        .bind(&name)
        .bind(&password_hash)
        .bind(role)
        .execute(db)
        .await
        .expect("Failed to create test user");

        Self {
            id,
            email,
            password,
            name,
        }
    }
}

/// Hash a password using argon2 for test fixtures.
fn hash_password(password: &str) -> String {
    use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
    use rand::rngs::OsRng;

    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string()
}

/// Initialize tracing for tests (only once).
fn init_tracing() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter("raksha=debug,tower_http=debug")
            .with_test_writer()
            .try_init()
            .ok();
    });
}

/// Build the test router. This mirrors the production router with test-friendly state.
async fn create_test_router(db: PgPool) -> Router {
    // In a full integration test, this would build the real AppState.
    // For now, we construct a minimal version that exercises the HTTP layer.
    use axum::{routing::get, Json};
    use serde_json::json;

    // Placeholder: in production tests, wire up the full routes::build_router(state)
    Router::new().route(
        "/api/v1/health",
        get(|| async { Json(json!({"status": "healthy"})) }),
    )
}
