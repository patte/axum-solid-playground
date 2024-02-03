use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WebauthnError {
    #[error("unknown webauthn error")]
    Unknown,
    #[error("Corrupt Session")]
    CorruptSession,
    #[error("User Not Found")]
    UserNotFound,
    #[error("User Has No Credentials")]
    UserHasNoCredentials,
    #[error("Deserialising Session failed: {0}")]
    InvalidSessionState(#[from] tower_sessions::session::Error),
    #[error("Username must be between 3 and 24 characters")]
    InvalidUsername,
    #[error("Invalid UserID supplied during authentication")]
    InvalidUserUniqueId,
    #[error("Username already exists. Please login or choose a different username.")]
    UsernameAlreadyExists,
}
impl IntoResponse for WebauthnError {
    fn into_response(self) -> Response {
        let body = match self {
            WebauthnError::CorruptSession => "Corrupt Session",
            WebauthnError::UserNotFound => "User Not Found",
            WebauthnError::Unknown => "Unknown Error",
            WebauthnError::UserHasNoCredentials => "User Has No Credentials",
            WebauthnError::InvalidSessionState(_) => "Deserialising Session failed",
            WebauthnError::InvalidUsername => "Username must be between 3 and 24 characters",
            WebauthnError::InvalidUserUniqueId => "Invalid UserID supplied during authentication",
            WebauthnError::UsernameAlreadyExists => {
                "Username already exists. Please sign in or choose a different username."
            }
        };

        // its often easiest to implement `IntoResponse` by calling other implementations
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
