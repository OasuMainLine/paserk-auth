use axum::response::{IntoResponse, Result};
use log::info;
use shared::responses::ApiSuccess;

pub async fn get_health() -> Result<impl IntoResponse> {
    info!("get_health request");
    Ok(ApiSuccess::empty())
}
