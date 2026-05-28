use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::{routes::v1::handlers, state::AppState};

pub fn router(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/sign-up", post(handlers::sign_up_user))
        .route("/sign-in", post(handlers::sign_in_user))
        .route("/me", get(handlers::verify_user))
        .route("/logout", post(handlers::logout_user))
        .route("/.well-known/paserk.json", get(handlers::get_public_keys))
        .with_state(app_state)
}
