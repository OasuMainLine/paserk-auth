//! Responses file
//! Stores common api statuses and response related utilities
//! For this project we use JSend: https://github.com/omniti-labs/jsend
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};

#[derive(Debug, Clone, PartialEq)]
pub enum APIStatus {
    SUCCESS,
    FAIL,
    ERROR,
}
impl APIStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            APIStatus::SUCCESS => "success",
            APIStatus::FAIL => "fail",
            APIStatus::ERROR => "error",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApiSuccess {
    pub code: StatusCode,
    pub data: Value,
}

#[derive(Debug, Clone)]
pub struct ApiFail {
    pub code: StatusCode,
    pub data: Value,
}

#[derive(Debug, Clone)]
pub struct ApiError {
    pub code: StatusCode,
    pub message: String,
    pub data: Option<Value>,
}

impl ApiSuccess {
    pub fn ok(data: Value) -> Self {
        Self {
            data,
            code: StatusCode::OK,
        }
    }
    /// Alias for `ApiSuccess::ok(json!({}))`
    pub fn empty() -> Self {
        Self::ok(json!({}))
    }

    pub fn created(data: Value) -> Self {
        Self {
            data,
            code: StatusCode::CREATED,
        }
    }
}

impl ApiFail {
    pub fn bad_request(data: Value) -> Self {
        Self {
            data,
            code: StatusCode::BAD_REQUEST,
        }
    }
    pub fn unprocessable_entity(data: Value) -> Self {
        Self {
            data,
            code: StatusCode::UNPROCESSABLE_ENTITY,
        }
    }

    pub fn unauthorized(data: Value) -> Self {
        Self {
            data,
            code: StatusCode::UNAUTHORIZED,
        }
    }
    pub fn forbidden(data: Value) -> Self {
        Self {
            data,
            code: StatusCode::FORBIDDEN,
        }
    }
}

impl ApiError {
    pub fn server_error(message: &str) -> Self {
        Self {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            data: None,
            message: message.to_string(),
        }
    }
    pub fn server_error_with_data(data: Value, message: &str) -> Self {
        Self {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            data: Some(data),
            message: message.to_string(),
        }
    }
}

impl IntoResponse for ApiSuccess {
    fn into_response(self) -> Response {
        return (
            self.code,
            Json(json!({
                "status": APIStatus::SUCCESS.as_str(),
                "data": self.data
            })),
        )
            .into_response();
    }
}

impl IntoResponse for ApiFail {
    fn into_response(self) -> Response {
        return (
            self.code,
            Json(json!({
                "status": APIStatus::FAIL.as_str(),
                "data": self.data
            })),
        )
            .into_response();
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        return (
            self.code,
            Json(json!({
                "status": APIStatus::ERROR.as_str(),
                "data": self.data
            })),
        )
            .into_response();
    }
}
