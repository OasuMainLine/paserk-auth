use axum::response::{IntoResponse, Result};
use serde_json::json;
use shared::responses::ApiSuccess;

pub async fn get_todos() -> Result<impl IntoResponse> {
    return Ok(ApiSuccess::ok(json!({
        "todos": vec![json!({
            "name": "Clean garage",
            "completed": false
        })]
    })));
}
