use axum::{extract::Extension, http::StatusCode, response::IntoResponse, routing::post, Router};
use std::{net::SocketAddr, str::FromStr};
use tower_sessions::{
    cookie::{time::Duration, SameSite},
    Expiry, MemoryStore, SessionManagerLayer,
};

mod error;

use crate::auth::{finish_authentication, finish_register, start_authentication, start_register};
use crate::startup::AppState;

// enables !info, !warn, etc.
#[macro_use]
extern crate tracing;

mod auth;
mod startup;

use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    // load env
    dotenv().ok();

    set_default_env_var("RUST_LOG", "INFO");
    set_default_env_var("LISTEN_HOST_PORT", "127.0.0.1:3000");

    // initialize tracing
    tracing_subscriber::fmt::init();

    let app_state = AppState::new();

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_name(&env::var("SESSION_NAME").unwrap_or("session".to_string()))
        .with_same_site(SameSite::Strict)
        .with_secure(env::var("SESSION_SECURE").unwrap_or("true".to_string()) != "false")
        .with_expiry(Expiry::OnInactivity(Duration::seconds(360)));

    // build our application with a route
    let app = Router::new()
        .route("/register_start/:username", post(start_register))
        .route("/register_finish", post(finish_register))
        .route("/login_start/:username", post(start_authentication))
        .route("/login_finish", post(finish_authentication))
        .layer(Extension(app_state))
        .layer(session_layer)
        .fallback(handler_404);

    let app = Router::new().merge(app).nest_service(
        "/",
        tower_http::services::ServeDir::new("../client/release"),
    );

    //run
    let addr = SocketAddr::from_str(&env::var("LISTEN_HOST_PORT").unwrap())
        .expect("Invalid LISTEN_HOST_PORT environment variable");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 - Not Found")
}

fn set_default_env_var(key: &str, value: &str) {
    if env::var(key).is_err() {
        env::set_var(key, value);
    }
}
