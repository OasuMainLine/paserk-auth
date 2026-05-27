use std::sync::Arc;

use axum::Router;

use crate::{routes::v1::auth, state::AppState};

pub fn router(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new().nest("/auth", auth::router(app_state.clone()))
}
