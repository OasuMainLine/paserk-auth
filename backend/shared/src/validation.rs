use axum::http::StatusCode;
use serde_json::json;
use validator::Validate;

use crate::responses::ApiResponse;

pub struct AxumValidator;

impl AxumValidator {
    pub fn validate(schema: &impl Validate) -> Result<(), ApiResponse> {
        schema.validate().map_err(|errors| {
            ApiResponse::new()
                .fail()
                .status_code(StatusCode::UNPROCESSABLE_ENTITY)
                .data(json!(errors))
        })
    }
}
