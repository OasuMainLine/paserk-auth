use axum::response::IntoResponse;
use log::{error, warn};
use serde_json::json;
use shared::responses::{ApiError, ApiFail};
#[derive(thiserror::Error, Debug)]
pub enum AuthServiceError {
    // Library Specific Errors
    #[error("Error while performing database operation")]
    DieselError(#[from] diesel::result::Error),
    #[error("Unable to acquire pool session")]
    DieselPoolError(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("Unable to parse token")]
    PasetoError(#[from] rusty_paseto::Error),
    #[error("Unable to acquire redis connection")]
    RedisError(#[from] redis::RedisError),

    // Domain Errors
    #[error("Forbidden")]
    ForbiddenError(Option<String>),

    #[error("Unauthorized")]
    UnauthorizedError(Option<String>),

    #[error("User with given email already exists")]
    ExistingUserError,
}

impl IntoResponse for AuthServiceError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::DieselError(ref e) => {
                error!("Database error: {}", &e);
            }
            Self::DieselPoolError(ref e) => {
                error!("Database pool error {}", &e);
            }
            Self::PasetoError(ref e) => {
                error!("Paseto error {}", &e)
            }
            Self::RedisError(ref e) => {
                error!("Redis error {}", &e)
            }
            ref e => {
                warn!("Error on response {}", &e)
            }
        };
        match self {
            Self::DieselError(_)
            | Self::DieselPoolError(_)
            | Self::PasetoError(_)
            | Self::RedisError(_) => ApiError::server_error(&self.to_string()).into_response(),
            Self::ExistingUserError => ApiFail::unprocessable_entity(json!({
                "message": &self.to_string()
            }))
            .into_response(),
            Self::ForbiddenError(ref msg) => ApiFail::forbidden(json!({
                "message": msg.clone().unwrap_or(self.to_string())
            }))
            .into_response(),
            Self::UnauthorizedError(ref msg) => ApiFail::unauthorized(json!({
                "message": msg.clone().unwrap_or(self.to_string())
            }))
            .into_response(),
        }
    }
}
