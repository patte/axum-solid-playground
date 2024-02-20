use axum::async_trait;
use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    response::IntoResponse,
};
use std::env;
use tower_cookies::Cookies;
use tower_sessions::Session;

use chrono::{DateTime, Utc};
use cookie::time::{Duration, OffsetDateTime};
use cookie::{Cookie, SameSite};

use crate::error::WebauthnError;
use crate::models::User;
use crate::queries;
use crate::state::AppState;

const COOKIE_NAME_JS: &str = "authenticated_user_js";

// remembers the user in the server side session and a cookie for the client
// the session is used server side
// the cookie to inform the client app
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

    cookies.add(create_informative_cookie(user, session.expiry_date()));
    Ok(())
}

// post signout handler
// remove session and informative cookie
pub async fn signout(session: Session, cookies: Cookies) -> Result<(), StatusCode> {
    session.flush().await.map_err(|e| {
        error!("Failed to remove authenticated_user from session: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    cookies.remove(Cookie::new(COOKIE_NAME_JS, ""));
    Ok(())
}

// informative cookie for the client app
// readable by the js app (plaintext, http only = false)
//   used to hydrate session state on first render
//   used to know when to refresh the session
// informative: only used to render the ui, not used for authentication
// see AuthContext.tsx for the client side code
fn create_informative_cookie(user: User, expiry_date: OffsetDateTime) -> Cookie<'static> {
    let expiry_date = expiry_date - Duration::seconds(1);

    #[derive(serde::Serialize)]
    struct CookiePayload {
        user: User,
        #[serde(with = "time::serde::rfc3339")]
        expiry_date: OffsetDateTime,
    }

    let payload = serde_json::to_string(&CookiePayload { user, expiry_date }).unwrap();

    Cookie::build((COOKIE_NAME_JS, payload))
        .path("/")
        .expires(expiry_date)
        .http_only(false)
        .same_site(SameSite::Strict)
        .secure(env::var("COOKIES_SECURE").unwrap_or("true".to_string()) != "false")
        .build()
}

// not called for auth routes ⬆️
// but only for api routes ⬇️
// roll the session and cookie expiry date
const ROLL_SESSION_EVERY_SECONDS: i64 = 60;
pub async fn roll_expiry_mw(
    cookies: Cookies,
    session: Session,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let response = next.run(request).await;

    let me = get_me_from_session(session.clone()).await;

    if me.is_some() {
        let now = chrono::Utc::now();
        let last_activity: Option<DateTime<Utc>> = session.get("last_activity").await.unwrap();
        let do_roll = match last_activity {
            Some(last_activity) => (now - last_activity).num_seconds() > ROLL_SESSION_EVERY_SECONDS,
            None => true,
        };
        if do_roll {
            // don't touch authenticated_user!
            // the expiry for the complete session (including authenticated_user)
            // is extended when last_activity is updated
            session.insert("last_activity", now).await.unwrap();
            // sync informative cookie
            cookies.add(create_informative_cookie(
                me.unwrap(),
                session.expiry_date(),
            ));
        }
    } else if cookies.get(COOKIE_NAME_JS).is_some() {
        info!("cookie found, but no user in session");
        cookies.remove(Cookie::new(COOKIE_NAME_JS, ""));
    }

    response
}

// get me from session
async fn get_me_from_session(session: Session) -> Option<User> {
    session
        .get::<User>("authenticated_user")
        .await
        .unwrap_or(None)
}

pub struct ExtractMe(pub Option<User>);

#[async_trait]
impl<S> axum::extract::FromRequestParts<S> for ExtractMe
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let session = parts.extensions.get::<tower_sessions::Session>().unwrap();
        let me = get_me_from_session(session.clone()).await;
        Ok(ExtractMe(me))
    }
}

pub struct ExtractMeEnsure(pub User);

#[async_trait]
impl<S> axum::extract::FromRequestParts<S> for ExtractMeEnsure
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let session = parts.extensions.get::<tower_sessions::Session>().unwrap();
        let me = get_me_from_session(session.clone()).await;
        match me {
            Some(me) => Ok(ExtractMeEnsure(me)),
            None => Err((StatusCode::UNAUTHORIZED, "Unauthorized")),
        }
    }
}

// rest handlers

pub async fn get_me(
    ExtractMeEnsure(user): ExtractMeEnsure,
) -> Result<impl IntoResponse, StatusCode> {
    Ok(Json(user))
}

pub async fn get_my_authenticators(
    Extension(app_state): Extension<AppState>,
    ExtractMeEnsure(user): ExtractMeEnsure,
) -> Result<impl IntoResponse, StatusCode> {
    let authenticators = app_state
        .db
        .conn
        .call(move |conn| {
            queries::get_authenticators_for_user_id(conn, user.id).map_err(|e| e.into())
        })
        .await
        .map_err(|e| {
            error!("get_authenticators_for_user: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(authenticators))
}

// for graphql handlers see graphql.rs
