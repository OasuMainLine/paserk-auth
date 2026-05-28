use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Result};

use diesel::sql_types::Integer;
use diesel::{IntoSql, select};
use diesel_async::RunQueryDsl;
use log::info;
use serde_json::json;
use shared::responses::{ApiFail, ApiSuccess};

use crate::errors::AuthServiceError;
use crate::utils::extractors::Database;

pub async fn get_health() -> Result<impl IntoResponse> {
    info!("get_health request");
    Ok(ApiSuccess::empty())
}

pub async fn get_health_db(Database(mut db): Database) -> Result<impl IntoResponse> {
    info!("get_health_db request");

    select(1.into_sql::<Integer>())
        .get_result::<i32>(&mut db)
        .await
        .map_err(AuthServiceError::from)
        .map_err(AuthServiceError::from)?;

    Ok(ApiSuccess::ok(json!({
        "message": "Database operational"
    })))
}

pub async fn get_root() -> (StatusCode, ()) {
    info!("get_root request");
    (StatusCode::NO_CONTENT, ())
}

pub async fn get_not_found_fallback(uri: Uri) -> impl IntoResponse {
    ApiFail::not_found(json!({
        "reason": "route not found",
        "uri": uri.to_string()
    }))
}
