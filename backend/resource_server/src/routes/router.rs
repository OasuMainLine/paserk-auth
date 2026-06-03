use std::sync::Arc;

use axum::Router;

use crate::{
    routes::{root, v1},
    state::AppState,
};

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let app_router = Router::new()
        .merge(root::router(state.clone()))
        .merge(v1::router(state.clone()))
        .with_state(state);

    Router::new().nest("/resource", app_router)
}
