use crate::error::WebauthnError;
use crate::state::AppState;
use axum::{
    extract::{Extension, Json, Path},
    response::IntoResponse,
};
use tower_sessions::Session;

use webauthn_rs::prelude::*;

use crate::store::User;

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
    if app_state
        .db
        .store
        .check_username_exists(username.clone())
        .await
        .unwrap()
    {
        return Err(WebauthnError::UsernameAlreadyExists);
    }

    let new_user = User::new(username);

    // Remove any previous registrations that may have occured from the session.
    session
        .remove_value("reg_state")
        .await
        .expect("Failed to remove reg_state from session");

    let res = match app_state.webauthn.start_passkey_registration(
        new_user.id,
        &new_user.username,
        &new_user.username,
        // This would only be needed if we allow users to register multiple credentials.
        // atm register is only allowed for new users.
        None,
    ) {
        Ok((ccr, reg_state)) => {
            // Store auth state in session. This is only save because session
            // store is server side. A cookie store would enable replay attacks.
            session
                .insert("reg_state", (new_user, reg_state))
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
    let (new_user, reg_state): (User, PasskeyRegistration) = match session.get("reg_state").await? {
        Some((new_user, reg_state)) => (new_user, reg_state),
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
            app_state
                .db
                .store
                .insert_user_and_passkey(new_user.clone(), sk.clone())
                .await
                .unwrap();

            info!("finish register successful!");
            Json(new_user)
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
            // Store auth state in session. This is only save because session
            // store is server side. A cookie store would enable replay attacks.
            session
                .insert("auth_state", auth_state)
                .await
                .expect("Failed to insert");
            Json(rcr)
        }
        Err(e) => {
            info!("Error in start_authentication: {:?}", e);
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

    // parse user_id and cred_id from the client supplied credential
    let (user_id, cred_id) = match app_state
        .webauthn
        .identify_discoverable_authentication(&auth)
    {
        Ok(creds) => creds,
        Err(e) => {
            info!("Error in finish_authentication: {:?}", e);
            return Err(WebauthnError::InvalidUserUniqueId);
        }
    };

    // find key for user that matches cred_id
    let passkey = match app_state
        .db
        .store
        .get_passkey_for_user_and_passkey_id(user_id, Base64UrlSafeData::from(cred_id).to_string())
        .await
        .unwrap()
    {
        Some(passkey) => passkey,
        None => {
            return Err(WebauthnError::UserNotFound);
        }
    };

    let res = match app_state.webauthn.finish_discoverable_authentication(
        &auth,
        auth_state,
        &[DiscoverableKey::from(passkey)],
    ) {
        Ok(auth_result) => {
            // Update the credential counter if needed.
            if auth_result.needs_update() {
                app_state
                    .db
                    .store
                    .update_passkey_for_user_and_passkey_id(
                        user_id.clone(),
                        Base64UrlSafeData::from(cred_id).to_string(),
                        auth_result.counter(),
                        auth_result.backup_state(),
                        auth_result.backup_eligible(),
                    )
                    .await
                    .unwrap();
            }

            // load user
            let user = app_state
                .db
                .store
                .get_user_by_id(user_id.clone())
                .await
                .unwrap();

            // TODO: set session authenticated

            Json(user)
        }
        Err(e) => {
            info!("Error in finish_authentication: {:?}", e);
            return Err(WebauthnError::Unknown);
        }
    };
    info!("Authentication Successful!");
    Ok(res)
}
