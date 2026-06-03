use axum::response::{IntoResponse, Result};
use serde_json::json;
use shared::responses::ApiSuccess;

use crate::middlewares::session::Session;

pub async fn get_todos(session: Session) -> Result<impl IntoResponse> {
    println!("{:#?}", session);
    return Ok(ApiSuccess::ok(json!({
        "todos": vec![json!({
            "name": "Clean garage",
            "completed": false
        })]
    })));
}
