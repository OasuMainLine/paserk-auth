use std::sync::Arc;

use axum::Router;
use axum_cookie::CookieLayer;
use resource_server::{config::Config, routes, state::AppState};
use shared::{env::load_env, paserk::PaserkClient};
use tokio::sync::OnceCell;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    load_env().expect("Error loading env variables");

    let config = envy::from_env::<Config>().expect("Error loading configuration");
    let paserk_client = PaserkClient::new(&config.resource_paserk_url);
    tracing_subscriber::fmt::init();
    let app_state = Arc::new(AppState {
        config: config.clone(),
        paserk_client: OnceCell::const_new_with(paserk_client),
    });

    let services = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CookieLayer::default());

    let app = Router::new()
        .merge(routes::router(app_state.clone()))
        .layer(services)
        .with_state(app_state.clone());

    let address = format!("0.0.0.0:{}", &config.resource_service_port);

    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .expect("Error initializing tcp listener");

    println!("Starting server at {} 🚀🚀", &address);
    axum::serve(listener, app).await.expect("Runtime error");
}
