use axum::{Json, http::StatusCode, response::IntoResponse};
use log::{error, warn};
use serde_json::json;
use shared::responses::APIStatus;
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
}

impl AuthServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::DieselError(_)
            | Self::DieselPoolError(_)
            | Self::PasetoError(_)
            | Self::RedisError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UnauthorizedError(_) => StatusCode::UNAUTHORIZED,
            Self::ForbiddenError(_) => StatusCode::FORBIDDEN,
        }
    }
    fn api_status(&self) -> APIStatus {
        match self {
            Self::DieselError(_)
            | Self::DieselPoolError(_)
            | Self::PasetoError(_)
            | Self::RedisError(_) => APIStatus::ERROR,
            Self::ForbiddenError(_) | Self::UnauthorizedError(_) => APIStatus::FAIL,
        }
    }

    fn to_message(&self) -> String {
        match self {
            Self::DieselError(_)
            | Self::DieselPoolError(_)
            | Self::PasetoError(_)
            | Self::RedisError(_) => self.to_string(),
            Self::ForbiddenError(msg) | Self::UnauthorizedError(msg) => {
                if let Some(msg) = msg {
                    return msg.clone();
                }
                return self.to_string();
            }
        }
    }
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
        }

        if self.api_status() == APIStatus::ERROR {
            (
                self.status_code(),
                Json(json!({
                    "status": self.api_status().as_str(),
                    "message": self.to_message(),
                })),
            )
                .into_response()
        } else {
            (
                self.status_code(),
                Json(json!({
                    "status": self.api_status().as_str(),
                    "data": {
                        "message": self.to_message()
                    }
                })),
            )
                .into_response()
        }
    }
}
