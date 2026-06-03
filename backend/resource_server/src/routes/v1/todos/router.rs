use std::sync::Arc;

use axum::{Router, middleware, routing::get};

use crate::{
    middlewares::session::session_middleware, routes::v1::todos::handlers, state::AppState,
};

pub fn router(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    let auth_router = Router::new()
        .route("/todos", get(handlers::get_todos))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            session_middleware,
        ))
        .with_state(app_state.clone());

    auth_router
}
