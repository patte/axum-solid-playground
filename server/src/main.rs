use axum::{
    extract::Extension, http::StatusCode, response::IntoResponse, routing::get, routing::post,
    Router,
};
use std::{net::SocketAddr, str::FromStr};
use tower_cookies::{CookieManagerLayer, Key};
use tower_sessions::{
    cookie::{time::Duration, SameSite},
    Expiry, MemoryStore, SessionManagerLayer,
};

mod error;

use crate::auth::{
    finish_authentication, finish_register, get_me, signout, start_authentication, start_register,
};
use crate::state::AppState;

// enables !info, !warn, etc.
#[macro_use]
extern crate tracing;

mod auth;
mod db;
mod queries;
mod state;

#[cfg(feature = "dev_proxy")]
mod proxy;

use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    // load env
    dotenv().ok();

    set_default_env_var("RUST_LOG", "INFO");
    set_default_env_var("LISTEN_HOST_PORT", "127.0.0.1:3000");
    set_default_env_var("DATABASE_URL", "sqlite://sqlite.db");

    // initialize tracing
    tracing_subscriber::fmt::init();

    // initialize app state
    let app_state = AppState::new();

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_name(&env::var("SESSION_NAME").unwrap_or("session".to_string()))
        .with_same_site(SameSite::Strict)
        .with_secure(env::var("SESSION_SECURE").unwrap_or("true".to_string()) != "false")
        .with_expiry(Expiry::OnInactivity(Duration::seconds(360)));

    auth::set_key(Key::from(
        env::var("SESSION_KEY")
            .expect("SESSION_KEY environment variable not set")
            .as_bytes(),
    ));

    // listen
    let addr = SocketAddr::from_str(&env::var("LISTEN_HOST_PORT").unwrap())
        .expect("Invalid LISTEN_HOST_PORT environment variable");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    let router = Router::new()
        .route("/register_start/:username", post(start_register))
        .route("/register_finish", post(finish_register))
        .route("/authenticate_start", post(start_authentication))
        .route("/authenticate_finish", post(finish_authentication))
        .route("/health", get(|| async { "OK" }))
        .route("/me", get(get_me))
        .route("/signout", post(signout))
        .layer(Extension(app_state))
        .layer(session_layer)
        .layer(CookieManagerLayer::new())
        .fallback(handler_404);

    #[cfg(not(feature = "dev_proxy"))]
    {
        info!("Starting server on {addr}");
        axum::serve(listener, router).await.unwrap();
    }

    #[cfg(feature = "dev_proxy")]
    {
        info!("Starting dev server on {addr}");
        let client = proxy::get_client();
        let router = Router::new()
            .route("/", get(proxy::proxy_handler))
            .route("/*key", get(proxy::proxy_handler))
            .merge(router)
            .with_state(client);
        axum::serve(listener, router).await.unwrap();
    }

    info!("listening on {addr}");
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 - Not Found")
}

fn set_default_env_var(key: &str, value: &str) {
    if env::var(key).is_err() {
        env::set_var(key, value);
    }
}
