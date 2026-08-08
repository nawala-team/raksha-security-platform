//! Raksha Security Platform - Installation Wizard
//! 
//! Web-based installer for initial setup with SuperAdmin creation

use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

mod templates;
mod setup;

#[derive(Clone, Default)]
pub struct InstallerState {
    pub step: Arc<RwLock<u8>>,
    pub config: Arc<RwLock<InstallConfig>>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct InstallConfig {
    pub db_host: String,
    pub db_port: u16,
    pub db_name: String,
    pub db_user: String,
    pub db_password: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub admin_email: String,
    pub admin_password: String,
    pub admin_name: String,
    pub site_name: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    // Check if already installed
    if std::path::Path::new(".installed").exists() {
        eprintln!("⚠️  Raksha is already installed!");
        eprintln!("   Delete .installed file to run installer again.");
        std::process::exit(1);
    }

    let state = InstallerState::default();
    
    let app = Router::new()
        .route("/", get(index))
        .route("/install", get(index))
        .route("/install/requirements", get(step_requirements))
        .route("/install/database", get(step_database))
        .route("/install/database", post(save_database))
        .route("/install/admin", get(step_admin))
        .route("/install/admin", post(save_admin))
        .route("/install/finish", get(step_finish))
        .route("/install/run", post(run_install))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    println!("🚀 Raksha Installer running at http://{}", addr);
    println!("   Open in browser to start installation\n");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> Html<String> {
    Html(templates::welcome())
}

async fn step_requirements() -> Html<String> {
    let checks = setup::check_requirements().await;
    Html(templates::requirements(&checks))
}

async fn step_database(State(state): State<InstallerState>) -> Html<String> {
    let config = state.config.read().await;
    Html(templates::database(&config))
}

#[derive(Deserialize)]
pub struct DatabaseForm {
    db_host: String,
    db_port: u16,
    db_name: String,
    db_user: String,
    db_password: String,
    redis_url: String,
}

async fn save_database(
    State(state): State<InstallerState>,
    Form(form): Form<DatabaseForm>,
) -> impl IntoResponse {
    // Test database connection
    match setup::test_db_connection(&form.db_host, form.db_port, &form.db_name, &form.db_user, &form.db_password).await {
        Ok(_) => {
            let mut config = state.config.write().await;
            config.db_host = form.db_host;
            config.db_port = form.db_port;
            config.db_name = form.db_name;
            config.db_user = form.db_user;
            config.db_password = form.db_password;
            config.redis_url = form.redis_url;
            config.jwt_secret = setup::generate_secret();
            Redirect::to("/install/admin")
        }
        Err(e) => {
            // Return to database page with error
            Redirect::to(&format!("/install/database?error={}", urlencoding::encode(&e)))
        }
    }
}

async fn step_admin(State(state): State<InstallerState>) -> Html<String> {
    let config = state.config.read().await;
    Html(templates::admin(&config))
}

#[derive(Deserialize)]
pub struct AdminForm {
    admin_name: String,
    admin_email: String,
    admin_password: String,
    site_name: String,
}

async fn save_admin(
    State(state): State<InstallerState>,
    Form(form): Form<AdminForm>,
) -> Redirect {
    let mut config = state.config.write().await;
    config.admin_name = form.admin_name;
    config.admin_email = form.admin_email;
    config.admin_password = form.admin_password;
    config.site_name = form.site_name;
    Redirect::to("/install/finish")
}

async fn step_finish(State(state): State<InstallerState>) -> Html<String> {
    let config = state.config.read().await;
    Html(templates::finish(&config))
}

async fn run_install(State(state): State<InstallerState>) -> impl IntoResponse {
    let config = state.config.read().await;
    
    match setup::run_installation(&config).await {
        Ok(_) => {
            // Mark as installed
            std::fs::write(".installed", "1").ok();
            Html(templates::success())
        }
        Err(e) => {
            Html(templates::error(&e))
        }
    }
}
