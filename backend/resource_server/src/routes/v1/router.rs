use std::sync::Arc;

use axum::Router;

use crate::state::AppState;

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new().with_state(state)
}
