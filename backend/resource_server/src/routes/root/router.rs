use std::sync::Arc;

use axum::{Router, routing::get};

use crate::{routes::root::handlers, state::AppState};

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(handlers::get_health))
        .with_state(state)
}
