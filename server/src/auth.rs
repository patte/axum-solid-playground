use crate::error::WebauthnError;
use crate::startup::AppState;
use axum::{
    extract::{Extension, Json, Path},
    response::IntoResponse,
};
use tower_sessions::Session;

use webauthn_rs::prelude::*;

// Webauthn RS auth handlers.
// adapted for "conditional-ui" (no username required for authentication) based on the example here:
// source: https://github.com/kanidm/webauthn-rs/blob/master/tutorial/server/axum/src/auth.rs

// The registration flow:
//
//          ┌───────────────┐     ┌───────────────┐      ┌───────────────┐
//          │ Authenticator │     │    Browser    │      │     Site      │
//          └───────────────┘     └───────────────┘      └───────────────┘
//                  │                     │                      │
//                  │                     │     1. Start Reg     │
//                  │                     │    (with username)   │
//                  │                     │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶│
//                  │                     │                      │
//                  │                     │     2. Challenge     │
//                  │                     │◀ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┤
//                  │                     │                      │
//                  │  3. Select Token    │                      │
//             ─ ─ ─│◀ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│                      │
//  4. Verify │     │                     │                      │
//                  │  4. Yield PubKey    │                      │
//            └ ─ ─▶│─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶                      │
//                  │                     │                      │
//                  │                     │  5. Send Reg Opts    │
//                  │                     │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶│─ ─ ─
//                  │                     │                      │     │ 5. Verify
//                  │                     │                      │         PubKey
//                  │                     │                      │◀─ ─ ┘
//                  │                     │                      │─ ─ ─
//                  │                     │                      │     │ 6. Persist
//                  │                     │                      │       Credential
//                  │                     │                      │◀─ ─ ┘
//                  │                     │                      │
//                  │                     │                      │
//

// respond to the start registration request, provide the challenge to the browser.
pub async fn start_register(
    Extension(app_state): Extension<AppState>,
    session: Session,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, WebauthnError> {
    info!("Start register");

    // check username
    if username.len() < 3 || username.len() > 24 {
        return Err(WebauthnError::InvalidUsername);
    }

    // check if username is already registered
    {
        let users_guard = app_state.users.lock().await;
        if users_guard.name_to_id.contains_key(&username) {
            return Err(WebauthnError::UsernameAlreadyExists);
        }
    }

    // Since a user's username could change at anytime, we need to bind to a unique id.
    let user_unique_id = {
        let users_guard = app_state.users.lock().await;
        users_guard
            .name_to_id
            .get(&username)
            .copied()
            .unwrap_or_else(Uuid::new_v4)
    };

    info!("reg user_unique_id: {:?}", user_unique_id);

    // Remove any previous registrations that may have occured from the session.
    session
        .remove_value("reg_state")
        .await
        .expect("Failed to remove reg_state from session");

    // If the user has any other credentials, we exclude these here so they can't be duplicate registered.
    let exclude_credentials = {
        let users_guard = app_state.users.lock().await;
        users_guard
            .keys
            .get(&user_unique_id)
            .map(|keys| keys.iter().map(|sk| sk.cred_id().clone()).collect())
    };

    let res = match app_state.webauthn.start_passkey_registration(
        user_unique_id,
        &username,
        &username,
        exclude_credentials,
    ) {
        Ok((ccr, reg_state)) => {
            // Note that due to the session store in use being a server side memory store, this is
            // safe to store the reg_state into the session since it is not client controlled and
            // not open to replay attacks. If this was a cookie store, this would be UNSAFE.
            session
                .insert("reg_state", (username, user_unique_id, reg_state))
                .await
                .expect("Failed to insert");
            info!("Start register successful!");
            Json(ccr)
        }
        Err(e) => {
            info!("challenge_register -> {:?}", e);
            return Err(WebauthnError::Unknown);
        }
    };
    Ok(res)
}

// The browser has completed navigator.credentials.create and created a public key
// on their device. Verify the registration options and persist them.
pub async fn finish_register(
    Extension(app_state): Extension<AppState>,
    session: Session,
    Json(reg): Json<RegisterPublicKeyCredential>,
) -> Result<impl IntoResponse, WebauthnError> {
    let (username, user_unique_id, reg_state): (String, Uuid, PasskeyRegistration) =
        match session.get("reg_state").await? {
            Some((username, user_unique_id, reg_state)) => (username, user_unique_id, reg_state),
            None => {
                error!("Failed to get session");
                return Err(WebauthnError::CorruptSession);
            }
        };

    session
        .remove_value("reg_state")
        .await
        .expect("Failed to remove reg_state from session");

    let res = match app_state
        .webauthn
        .finish_passkey_registration(&reg, &reg_state)
    {
        Ok(sk) => {
            let mut users_guard = app_state.users.lock().await;

            //TODO: This is where we would store the credential in a db, or persist them in some other way.
            users_guard
                .keys
                .entry(user_unique_id)
                .and_modify(|keys| keys.push(sk.clone()))
                .or_insert_with(|| vec![sk.clone()]);

            users_guard
                .name_to_id
                .insert(username.clone(), user_unique_id);

            info!("finish register successful!");
            Json(User {
                id: user_unique_id,
                username,
            })
        }
        Err(e) => {
            error!("finish_passkey_registration: {:?}", e);
            return Err(WebauthnError::Unknown);
        }
    };

    Ok(res)
}

// The authentication flow:
//
//          ┌───────────────┐     ┌───────────────┐      ┌───────────────┐
//          │ Authenticator │     │    Browser    │      │     Site      │
//          └───────────────┘     └───────────────┘      └───────────────┘
//                  │                     │                      │
//                  │                     │     1. Start Auth    │
//                  │                     │     (no username)    │
//                  │                     │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶│
//                  │                     │                      │
//                  │                     │     2. Challenge     │
//                  │                     │◀ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┤
//                  │                     │                      │
//                  │  3. Select Token    │                      │
//             ─ ─ ─│◀ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│                      │
//  4. Verify │     │                     │                      │
//                  │    4. Yield Sig     │                      │
//            └ ─ ─▶│─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶                      │
//                  │                     │    5. Send Auth      │
//                  │                     │        Opts          │
//                  │                     │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶│─ ─ ─
//                  │                     │                      │     │ 5. Verify
//                  │                     │                      │          Sig
//                  │                     │                      │◀─ ─ ┘
//                  │                     │                      │
//                  │                     │                      │

// The user indicates the wish to start authentication and we need to provide a challenge.
// we use start_discoverable_authentication instead of start_passkey_authentication to allow
// the user to select a key to authenticate with.
pub async fn start_authentication(
    Extension(app_state): Extension<AppState>,
    session: Session,
) -> Result<impl IntoResponse, WebauthnError> {
    info!("Start Authentication");

    // Remove any previous authentication that may have occured from the session.
    session
        .remove_value("auth_state")
        .await
        .expect("Failed to remove auth_state from session");

    let res = match app_state.webauthn.start_discoverable_authentication() {
        Ok((rcr, auth_state)) => {
            // Note that due to the session store in use being a server side memory store, this is
            // safe to store the auth_state into the session since it is not client controlled and
            // not open to replay attacks. If this was a cookie store, this would be UNSAFE.
            session
                .insert("auth_state", auth_state)
                .await
                .expect("Failed to insert");
            Json(rcr)
        }
        Err(e) => {
            info!("challenge_authenticate -> {:?}", e);
            return Err(WebauthnError::Unknown);
        }
    };
    Ok(res)
}

// The browser and user have completed navigator.credentials.get.
// We need to check if a user exists for the claimed uuid, check if
// the used credential belongs to the user, and verify the signature.
pub async fn finish_authentication(
    Extension(app_state): Extension<AppState>,
    session: Session,
    Json(auth): Json<PublicKeyCredential>,
) -> Result<impl IntoResponse, WebauthnError> {
    let auth_state: DiscoverableAuthentication = session
        .get("auth_state")
        .await?
        .ok_or(WebauthnError::CorruptSession)?;

    session
        .remove_value("auth_state")
        .await
        .expect("Failed to remove auth_state from session");

    let creds = match app_state
        .webauthn
        .identify_discoverable_authentication(&auth)
    {
        Ok(creds) => creds,
        Err(e) => {
            info!("challenge_register -> {:?}", e);
            return Err(WebauthnError::InvalidUserUniqueId);
        }
    };

    let user_unique_id = creds.0;

    // find key for user that matches creds.1
    let mut users_guard = app_state.users.lock().await;

    let passkey = users_guard
        .keys
        .get(&user_unique_id)
        .and_then(|keys| {
            keys.iter()
                .find(|sk| sk.cred_id() == creds.1)
                .map(|sk| sk.clone())
        })
        .ok_or(WebauthnError::UserNotFound)?;

    let res = match app_state.webauthn.finish_discoverable_authentication(
        &auth,
        auth_state,
        &[DiscoverableKey::from(passkey)],
    ) {
        Ok(auth_result) => {
            info!("auth_result: {:?}", auth_result);

            // Update the credential counter, if possible.
            users_guard
                .keys
                .get_mut(&user_unique_id)
                .map(|keys| {
                    keys.iter_mut().for_each(|sk| {
                        // This will update the credential if it's the matching
                        // one. Otherwise it's ignored. That is why it is safe to
                        // iterate this over the full list.
                        sk.update_credential(&auth_result);
                    })
                })
                .ok_or(WebauthnError::UserHasNoCredentials)?;

            // get username for user_unique_id
            let username = users_guard
                .name_to_id
                .iter()
                .find(|(_, id)| **id == user_unique_id)
                .map(|(name, _)| name)
                .ok_or(WebauthnError::UserNotFound)?;

            Json(User {
                id: user_unique_id,
                username: username.to_string(),
            })
        }
        Err(e) => {
            info!("challenge_register -> {:?}", e);
            return Err(WebauthnError::Unknown);
        }
    };
    info!("Authentication Successful!");
    Ok(res)
}

// TODO: cleanup
use serde::Serialize;
#[derive(Serialize)]
struct User {
    id: Uuid,
    username: String,
}
