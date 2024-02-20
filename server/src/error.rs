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
    #[error("Deserialising Session failed: {0}")]
    InvalidSessionState(#[from] tower_sessions::session::Error),
    #[error("Username must be between 3 and 24 characters")]
    InvalidUsername,
    #[error("UserID and credentialID don't match")]
    UserAndCredentialDontMatch,
    #[error("Username already exists. Please login or choose a different username.")]
    UsernameAlreadyExists,
    #[error("Database error! Sorry! Please try again later.")]
    GenericDatabaseError,
    #[error("You can only register new credentials for yourself.")]
    RegisterForSelfOnly,
    #[error("You are already signed in.")]
    AlreadySignedIn,
}
impl IntoResponse for WebauthnError {
    fn into_response(self) -> Response {
        let body = match self {
            WebauthnError::CorruptSession => "Corrupt Session",
            WebauthnError::UserNotFound => "User Not Found",
            WebauthnError::Unknown => "Unknown Error",
            WebauthnError::InvalidSessionState(_) => "Deserialising Session failed",
            WebauthnError::InvalidUsername => "Username must be between 3 and 24 characters",
            WebauthnError::UserAndCredentialDontMatch => "UserID and credentialID don't match",
            WebauthnError::UsernameAlreadyExists => {
                "Username already exists. Please sign in or choose a different username."
            }
            WebauthnError::GenericDatabaseError => "Database error! Sorry! Please try again later.",
            WebauthnError::RegisterForSelfOnly => {
                "You can only register new credentials for yourself."
            }
            WebauthnError::AlreadySignedIn => "You are already signed in.",
        };

        // its often easiest to implement `IntoResponse` by calling other implementations
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
