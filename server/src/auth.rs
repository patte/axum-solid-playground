use crate::models::User;
use crate::session::ExtractMe;
use crate::state::AppState;
use crate::{error::WebauthnError, ua::user_agent::get_user_agent_string_short};
use crate::{queries, session};
use axum::{
    extract::{Extension, Json, Path},
    response::IntoResponse,
};
use chrono::Utc;
use tower_cookies::Cookies;
use tower_sessions::Session;

use webauthn_rs::prelude::*;

use crate::ua::user_agent::ExtractUserAgent;

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
    // error early if user_agent is missing or invalid
    ExtractUserAgent(_user_agent): ExtractUserAgent,
    ExtractMe(me): ExtractMe,
) -> Result<impl IntoResponse, WebauthnError> {
    info!("Start register");

    // check username
    if username.len() < 3 || username.len() > 24 {
        return Err(WebauthnError::InvalidUsername);
    }

    let (user, user_is_new) = match me {
        Some(me) => {
            if me.username != username {
                return Err(WebauthnError::RegisterForSelfOnly);
            }
            (me, false)
        }
        None => (User::new(username.clone()), true),
    };

    if user_is_new {
        // check if username exists
        if app_state
            .db
            .conn
            .call({
                let username = user.username.clone();
                move |conn| queries::check_username_exists(conn, &username).map_err(|e| e.into())
            })
            .await
            .map_err(|e| {
                error!("check_username_exists: {:?}", e);
                WebauthnError::GenericDatabaseError
            })?
        {
            return Err(WebauthnError::UsernameAlreadyExists);
        }
    }

    // load excluded credentials
    let exclude_credentials: Option<Vec<CredentialID>> = if user_is_new {
        None
    } else {
        let authenticators = app_state
            .db
            .conn
            .call(move |conn| {
                queries::get_authenticators_for_user_id(conn, user.id).map_err(|e| e.into())
            })
            .await
            .map_err(|e| {
                error!("get_authenticators_for_user: {:?}", e);
                WebauthnError::GenericDatabaseError
            })?;
        Some(
            authenticators
                .iter()
                .map(|a| a.passkey.cred_id().clone())
                .collect(),
        )
    };

    // Remove any previous registrations that may have occured from the session.
    session.remove_value("reg_state").await.map_err(|e| {
        error!("Failed to remove reg_state from session: {:?}", e);
        WebauthnError::CorruptSession
    })?;

    let res = match app_state.webauthn.start_passkey_registration(
        user.id,
        &user.username,
        &user.username,
        exclude_credentials,
    ) {
        Ok((ccr, reg_state)) => {
            // Store auth state in session. This is only save because session
            // store is server side. A cookie store would enable replay attacks.
            session
                .insert("reg_state", (user, user_is_new, reg_state))
                .await
                .map_err(|e| {
                    error!("Failed to insert reg_state into session: {:?}", e);
                    WebauthnError::CorruptSession
                })?;
            info!("Start register successful!");
            Json(ccr)
        }
        Err(e) => {
            info!("start_passkey_registration: {:?}", e);
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
    cookies: Cookies,
    ExtractUserAgent(user_agent): ExtractUserAgent,
    ExtractMe(me): ExtractMe,
    Json(reg): Json<RegisterPublicKeyCredential>,
) -> Result<impl IntoResponse, WebauthnError> {
    let ua_short = get_user_agent_string_short(&user_agent, &app_state.ua_parser);

    let (user, user_is_new, reg_state): (User, bool, PasskeyRegistration) = session
        .get("reg_state")
        .await
        .map_err(|e| {
            error!("Failed to get reg_state from session: {:?}", e);
            WebauthnError::CorruptSession
        })?
        .ok_or_else(|| {
            error!("Failed to get session");
            WebauthnError::CorruptSession
        })?;

    session.remove_value("reg_state").await.map_err(|e| {
        error!("Failed to remove reg_state from session: {:?}", e);
        WebauthnError::CorruptSession
    })?;

    let res = match app_state
        .webauthn
        .finish_passkey_registration(&reg, &reg_state)
    {
        Ok(sk) => {
            // save user and passkey to db
            app_state
                .db
                .conn
                .call({
                    let user = user.clone();
                    move |conn| {
                        if user_is_new {
                            queries::insert_user_and_passkey(conn, user, sk.clone(), &ua_short)
                                .map_err(|e| e.into())
                        } else {
                            queries::insert_authenticator(
                                conn,
                                user.id,
                                sk.clone(),
                                Utc::now(),
                                &ua_short,
                            )
                            .map_err(|e| e.into())
                            .map(|_| ())
                        }
                    }
                })
                .await
                .map_err(|e| {
                    error!("insert_user_and_passkey: {:?}", e);
                    WebauthnError::GenericDatabaseError
                })?;

            info!("finish register successful!");

            // set session authenticated
            if me.is_none() {
                session::set_me_authenticated(user.clone(), session, cookies).await?;
            }

            Json(user)
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
    ExtractMe(me): ExtractMe,
) -> Result<impl IntoResponse, WebauthnError> {
    info!("Start Authentication");

    if me.is_some() {
        return Err(WebauthnError::AlreadySignedIn);
    }

    // Remove any previous authentication that may have occured from the session.
    session.remove_value("auth_state").await.map_err(|e| {
        error!("Failed to remove auth_state from session: {:?}", e);
        WebauthnError::CorruptSession
    })?;

    let res = match app_state.webauthn.start_discoverable_authentication() {
        Ok((rcr, auth_state)) => {
            // Store auth state in session. This is only save because session
            // store is server side. A cookie store would enable replay attacks.
            session
                .insert("auth_state", auth_state)
                .await
                .map_err(|e| {
                    error!("Failed to insert auth_state into session: {:?}", e);
                    WebauthnError::CorruptSession
                })?;
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
    cookies: Cookies,
    ExtractMe(me): ExtractMe,
    Json(auth_input): Json<PublicKeyCredential>,
) -> Result<impl IntoResponse, WebauthnError> {
    if me.is_some() {
        return Err(WebauthnError::AlreadySignedIn);
    }

    let auth_state: DiscoverableAuthentication = session
        .get("auth_state")
        .await
        .map_err(|e| {
            error!("Failed to get auth_state from session: {:?}", e);
            WebauthnError::CorruptSession
        })?
        .ok_or_else(|| {
            error!("Failed to get session");
            WebauthnError::CorruptSession
        })?;

    session.remove_value("auth_state").await.map_err(|e| {
        error!("Failed to remove auth_state from session: {:?}", e);
        WebauthnError::CorruptSession
    })?;

    let (user_id, cred_id) = match app_state
        .webauthn
        .identify_discoverable_authentication(&auth_input)
    {
        Ok(creds) => creds,
        Err(e) => {
            info!("Error in finish_authentication: {:?}", e);
            return Err(WebauthnError::UserAndCredentialDontMatch);
        }
    };

    // make string from &[u8] to be able to copy it and not have lifetime
    // dependency on auth_input.
    let passkey_id = Base64UrlSafeData::from(cred_id).to_string();

    // try to find the used passkey for the claimed user_id
    let passkey = app_state
        .db
        .conn
        .call({
            let passkey_id = passkey_id.clone();
            move |conn| {
                queries::get_passkey_for_user_and_passkey_id(conn, user_id, passkey_id)
                    .map_err(|e| e.into())
            }
        })
        .await
        .map_err(|e| {
            error!("get_passkey_for_user_and_passkey_id: {:?}", e);
            WebauthnError::GenericDatabaseError
        })?
        .ok_or_else(|| {
            error!("Failed to get passkey for claimed user_id.");
            WebauthnError::UserNotFound
        })?;

    let res = match app_state.webauthn.finish_discoverable_authentication(
        &auth_input,
        auth_state,
        &[DiscoverableKey::from(passkey)],
    ) {
        Ok(auth_result) => {
            // Update the credential counter if needed.
            if auth_result.needs_update() {
                app_state
                    .db
                    .conn
                    .call({
                        let passkey_id = passkey_id.clone();
                        move |conn| {
                            queries::update_passkey_for_user_and_passkey_id(
                                conn,
                                user_id,
                                passkey_id,
                                auth_result.counter(),
                                auth_result.backup_state(),
                                auth_result.backup_eligible(),
                            )
                            .map_err(|e| e.into())
                        }
                    })
                    .await
                    .map_err(|e| {
                        error!("update_passkey_for_user_and_passkey_id: {:?}", e);
                        WebauthnError::GenericDatabaseError
                    })?;
            }

            // load user
            let user = app_state
                .db
                .conn
                .call(move |conn| queries::get_user_by_id(conn, user_id).map_err(|e| e.into()))
                .await
                .map_err(|e| {
                    error!("get_user_by_id: {:?}", e);
                    WebauthnError::GenericDatabaseError
                })?;

            // set session authenticated
            session::set_me_authenticated(user.clone(), session, cookies).await?;

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
