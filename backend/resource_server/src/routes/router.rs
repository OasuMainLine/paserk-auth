use std::sync::Arc;

use axum::Router;

use crate::{routes::root, state::AppState};

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .merge(root::router(state.clone()))
        .with_state(state)
}
