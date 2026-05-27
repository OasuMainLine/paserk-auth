//! Responses file
//! Stores common api statuses and response related utilities

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response, Result},
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
pub struct ApiResponse {
    pub data: Option<Value>,
    pub code: StatusCode,
    pub status: APIStatus,
    pub message: Option<String>,
}

impl Default for ApiResponse {
    fn default() -> Self {
        Self {
            data: None,
            code: StatusCode::OK,
            status: APIStatus::SUCCESS,
            message: None,
        }
    }
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> axum::response::Response {
        let body = match self.status {
            APIStatus::SUCCESS | APIStatus::FAIL => Json(json!({
                "status": self.status.as_str(),
                "data": self.data
            })),
            APIStatus::ERROR => Json(json!({
                "status": self.status.as_str(),
                "message": self.message,
                "data": self.data,
            })),
        };

        (self.code, body).into_response()
    }
}
impl Into<Result<Response>> for ApiResponse {
    fn into(self) -> Result<Response> {
        match self.status {
            APIStatus::SUCCESS => Ok(self.into_response()),
            _ => Err(self.into_response().into()),
        }
    }
}

impl ApiResponse {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn success(mut self) -> Self {
        self.status = APIStatus::SUCCESS;

        self
    }

    pub fn fail(mut self) -> Self {
        self.status = APIStatus::FAIL;
        self
    }
    pub fn error(mut self, message: &str) -> Self {
        self.status = APIStatus::ERROR;
        self.message = Some(message.to_string());

        self
    }

    pub fn data(mut self, data: Value) -> Self {
        self.data = Some(data);

        self
    }

    pub fn status_code(mut self, code: StatusCode) -> Self {
        self.code = code;
        self
    }
}
