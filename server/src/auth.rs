use std::env;

use crate::state::AppState;
use crate::{error::WebauthnError, extractors::get_user_agent_string_short};
use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use cookie::{Cookie, SameSite};
use tower_cookies::Cookies;
use tower_sessions::Session;

use webauthn_rs::prelude::*;

use crate::queries::User;

use crate::extractors::ExtractUserAgent;

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
) -> Result<impl IntoResponse, WebauthnError> {
    info!("Start register");

    // check username
    if username.len() < 3 || username.len() > 24 {
        return Err(WebauthnError::InvalidUsername);
    }

    // check if username exists
    if app_state
        .db
        .conn
        .call({
            let username = username.clone();
            move |conn| crate::queries::check_username_exists(conn, &username).map_err(|e| e.into())
        })
        .await
        .map_err(|e| {
            error!("check_username_exists: {:?}", e);
            WebauthnError::GenericDatabaseError
        })?
    {
        return Err(WebauthnError::UsernameAlreadyExists);
    }

    let new_user = User::new(username);

    // Remove any previous registrations that may have occured from the session.
    session.remove_value("reg_state").await.map_err(|e| {
        error!("Failed to remove reg_state from session: {:?}", e);
        WebauthnError::CorruptSession
    })?;

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
    Json(reg): Json<RegisterPublicKeyCredential>,
) -> Result<impl IntoResponse, WebauthnError> {
    let ua_short = get_user_agent_string_short(&user_agent, &app_state.ua_parser);
    println!("short ua: {}", ua_short);

    let (new_user, reg_state): (User, PasskeyRegistration) = session
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
                    let new_user = new_user.clone();
                    move |conn| {
                        crate::queries::insert_user_and_passkey(
                            conn,
                            new_user,
                            sk.clone(),
                            &ua_short,
                        )
                        .map_err(|e| e.into())
                    }
                })
                .await
                .map_err(|e| {
                    error!("insert_user_and_passkey: {:?}", e);
                    WebauthnError::GenericDatabaseError
                })?;

            info!("finish register successful!");

            // set session authenticated
            set_me_authenticated(new_user.clone(), session, cookies).await?;

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
    Json(auth_input): Json<PublicKeyCredential>,
) -> Result<impl IntoResponse, WebauthnError> {
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
                crate::queries::get_passkey_for_user_and_passkey_id(conn, user_id, passkey_id)
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
                            crate::queries::update_passkey_for_user_and_passkey_id(
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
                .call(move |conn| {
                    crate::queries::get_user_by_id(conn, user_id).map_err(|e| e.into())
                })
                .await
                .map_err(|e| {
                    error!("get_user_by_id: {:?}", e);
                    WebauthnError::GenericDatabaseError
                })?;

            // set session authenticated
            set_me_authenticated(user.clone(), session, cookies).await?;

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

const COOKIE_NAME_JS: &str = "authenticated_user_js";

// remembers the user in the server side session and a cookie for the client
// the session is used server side
// the cookie for the client side js app (plaintext, http only = false)
//   used to hydrate session state on first render
//   only informative to client
pub async fn set_me_authenticated(
    user: User,
    session: Session,
    cookies: Cookies,
) -> Result<(), WebauthnError> {
    session
        .insert("authenticated_user", user.clone())
        .await
        .map_err(|e| {
            error!("Failed to insert authenticated_user into session: {:?}", e);
            WebauthnError::CorruptSession
        })?;

    // informative cookie for the client app
    // readable by the javascript app
    let user_stringified = serde_json::to_string(&user).map_err(|_| WebauthnError::Unknown)?;
    cookies.add(
        Cookie::build((COOKIE_NAME_JS, user_stringified))
            .expires(session.expiry_date())
            .http_only(false)
            .same_site(SameSite::Strict)
            .secure(env::var("COOKIES_SECURE").unwrap_or("true".to_string()) != "false")
            .build(),
    );
    Ok(())
}

pub async fn signout(session: Session, cookies: Cookies) -> Result<(), StatusCode> {
    session.flush().await.map_err(|e| {
        error!("Failed to remove authenticated_user from session: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    cookies.remove(Cookie::new(COOKIE_NAME_JS, ""));
    Ok(())
}

pub async fn get_me(session: Session) -> Option<User> {
    session
        .get::<User>("authenticated_user")
        .await
        .unwrap_or_else(|e| {
            error!("Failed to get authenticated_user from session: {:?}", e);
            None
        })
}

pub async fn get_me_handler(session: Session) -> Result<impl IntoResponse, StatusCode> {
    let user = get_me(session).await;
    match user {
        Some(user) => Ok(Json(user)),
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

// code to get cookie in client side js
// ```js
/*
document.cookie
    .split(';')
    .map(v => v.trim())
    .find(v => v.startsWith('authenticated_user'))
    .split('=')[2]
*/
// ```
