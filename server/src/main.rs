use axum::{
    extract::Extension,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};

#[cfg(not(feature = "dev_proxy"))]
use axum_embed::ServeEmbed;
#[cfg(not(feature = "dev_proxy"))]
use rust_embed::RustEmbed;

use std::{net::SocketAddr, str::FromStr};
use tower_cookies::CookieManagerLayer;
use tower_sessions::{
    cookie::{time::Duration, SameSite},
    session_store::ExpiredDeletion,
    Expiry, SessionManagerLayer,
};

mod error;

use crate::auth::{
    finish_authentication, finish_register, get_me_handler, signout, start_authentication,
    start_register,
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

mod chat;
mod rusqlite_session_store;

#[cfg(not(feature = "dev_proxy"))]
#[derive(RustEmbed, Clone)]
#[folder = "../client/dist/"]
struct ClientDist;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // load env
    dotenv().ok();

    set_default_env_var("RUST_LOG", "INFO");
    set_default_env_var("LISTEN_HOST_PORT", "0.0.0.0:3000");
    set_default_env_var("DATABASE_URL", "sqlite://sqlite.db");

    // initialize tracing
    tracing_subscriber::fmt::init();

    // initialize app state
    let app_state = AppState::new().await;

    let session_store = rusqlite_session_store::RusqliteStore::new(app_state.db.conn.clone());
    session_store.migrate().await.unwrap();

    let deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(50)),
    );

    let session_layer = SessionManagerLayer::new(session_store)
        .with_name(&env::var("SESSION_NAME").unwrap_or("session".to_string()))
        .with_same_site(SameSite::Strict)
        .with_secure(env::var("COOKIES_SECURE").unwrap_or("true".to_string()) != "false")
        .with_expiry(Expiry::OnInactivity(Duration::hours(24)));

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
        .route("/me", get(get_me_handler))
        .route("/debug", get(get_debug))
        .route("/signout", post(signout))
        .route("/chat", get(chat::websocket_handler))
        .layer(Extension(app_state))
        .layer(session_layer)
        .layer(CookieManagerLayer::new())
        .fallback(handler_404);

    #[cfg(not(feature = "dev_proxy"))]
    {
        let serve_client = ServeEmbed::<ClientDist>::new();
        let router = Router::new().nest_service("/", serve_client).merge(router);
        info!("Starting server on {addr}");
        axum::serve(listener, router).await.unwrap();
    }

    #[cfg(feature = "dev_proxy")]
    {
        let client = proxy::get_client();
        let router = Router::new()
            .route("/", get(proxy::proxy_handler))
            .route("/*key", get(proxy::proxy_handler))
            .merge(router)
            .with_state(client);
        info!("Starting dev server on {addr}");
        axum::serve(listener, router).await.unwrap();
    }

    info!("listening on {addr}");

    deletion_task.await??;

    Ok(())
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 - Not Found")
}

fn set_default_env_var(key: &str, value: &str) {
    if env::var(key).is_err() {
        env::set_var(key, value);
    }
}

async fn get_debug(headers: axum::http::HeaderMap) -> impl IntoResponse {
    let env_primary_region = std::env::var("PRIMARY_REGION").unwrap_or("".to_string());
    let env_region = std::env::var("FLY_REGION").unwrap_or("".to_string());
    let machine_is_in_primary_region = env_primary_region == env_region && env_region != "";
    let req_region = headers
        .get("Fly-Region")
        .map(|v| v.to_str().unwrap_or(""))
        .unwrap_or("");
    let req_via = headers
        .get("Via")
        .map(|v| v.to_str().unwrap_or(""))
        .unwrap_or("");
    axum::Json(serde_json::json!({
        "FLY_MACHINE_ID": std::env::var("FLY_MACHINE_ID").unwrap_or("".to_string()),
        "PRIMARY_REGION": env_primary_region,
        "FLY_REGION": env_region,
        "machine_is_in_primary_region": machine_is_in_primary_region,
        "req_region": req_region,
        "req_via": req_via,
    }))
}
