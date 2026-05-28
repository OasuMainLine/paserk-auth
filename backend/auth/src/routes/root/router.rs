use std::sync::Arc;

use axum::{Router, routing::get};

use crate::{routes::root::handlers, state::AppState};

pub fn router(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(handlers::get_root))
        .route("/health", get(handlers::get_health))
        .route("/health/db", get(handlers::get_health_db))
        .fallback(handlers::get_not_found_fallback)
        .with_state(app_state)
}
