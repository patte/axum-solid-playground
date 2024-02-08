//! Example chat application.
// source: https://github.com/tokio-rs/axum/blob/main/examples/chat/src/main.rs

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    Extension,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tower_sessions_core::Session;

use crate::{queries::User, state::AppState};

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<AppState>,
    session: Session,
) -> Result<impl IntoResponse, StatusCode> {
    let me = crate::auth::get_me(session)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;
    info!(
        "{} {} connected",
        std::env::var("FLY_REGION").unwrap_or("".to_string()),
        me.username
    );
    Ok(ws.on_upgrade(|socket| websocket(socket, state, me)))
}

// This function deals with a single websocket connection, i.e., a single
// connected client / user, for which we will spawn two independent tasks (for
// receiving / sending chat messages).
async fn websocket(stream: WebSocket, state: AppState, me: User) {
    // By splitting, we can send and receive at the same time.
    let (mut sender, mut receiver) = stream.split();

    let mut username = me.username.clone();

    // find a unique username (multiple tabs, devices) and insert it into active_usernames
    for i in 1..100 {
        if insert_username_if_unique(&state, &username) {
            break;
        } else if i == 99 {
            error!("Could not find a unique username for {}", me.username);
            return;
        }
        username = format!("{} ({})", me.username, i);
    }

    // send recent message to our client
    let recent_messages = get_recent_messages(&state);
    for msg in recent_messages.iter() {
        if sender.send(Message::Text(msg.clone())).await.is_err() {
            break;
        }
    }

    // We subscribe *before* sending the "joined" message, so that we will also
    // display it to our client.
    let mut rx = state.tx.subscribe();

    // Now send the "joined" message to all subscribers.
    let msg = format!("👋{username} joined.");
    tracing::debug!("{msg}");
    remember_message(&state, &msg);
    let _ = state.tx.send(msg);

    // update number of connected users
    broadcast_connected_usernames_count(&state);

    // Spawn the first task that will receive broadcast messages and send text
    // messages over the websocket to our client.
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any websocket error, break loop.
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Clone things we want to pass (move) to the receiving task.
    let tx = state.tx.clone();
    let name = username.clone();

    // Spawn a task that takes messages from the websocket, prepends the user
    // name, and sends them to all broadcast subscribers.
    let mut recv_task = tokio::spawn({
        let state = state.clone();
        async move {
            while let Some(Ok(Message::Text(text))) = receiver.next().await {
                let msg = format!("💬{name}: {text}");
                remember_message(&state, &msg);
                let _ = tx.send(msg);
            }
        }
    });

    // If any one of the tasks run to completion, we abort the other.
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    // Send "user left" message (similar to "joined" above).
    let msg = format!("👋{username} left.");
    tracing::debug!("{msg}");
    remember_message(&state, &msg);
    let _ = state.tx.send(msg);

    // Remove username from map
    state.connected_usernames.lock().unwrap().remove(&username);

    // update number of connected users
    broadcast_connected_usernames_count(&state);
}

fn insert_username_if_unique(state: &AppState, username: &str) -> bool {
    let mut user_set = state.connected_usernames.lock().unwrap();

    if !user_set.contains(username) {
        user_set.insert(username.to_owned());
        true
    } else {
        false
    }
}

fn broadcast_connected_usernames_count(state: &AppState) {
    let msg = format!("🧮{}", state.connected_usernames.lock().unwrap().len());
    let _ = state.tx.send(msg);
}

fn remember_message(state: &AppState, msg: &str) {
    let mut recent_messages = state.recent_messages.lock().unwrap();
    recent_messages.push(msg.to_owned());
    if recent_messages.len() > 7 {
        recent_messages.remove(0);
    }
}

fn get_recent_messages(state: &AppState) -> Vec<String> {
    state.recent_messages.lock().unwrap().clone()
}
