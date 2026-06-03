use std::sync::Arc;

use axum::Router;

use crate::{routes::v1::todos, state::AppState};

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new().nest("/v1", todos::router(state.clone()))
}
