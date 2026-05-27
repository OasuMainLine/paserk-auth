use std::sync::Arc;

use auth::{config::Config, routes, state::AppState};
use axum::Router;
use axum_cookie::CookieLayer;
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool};
use redis::Client;
use shared::env::load_env;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use validator::Validate;

#[tokio::main]
async fn main() {
    load_env().expect("Error loading env variables");
    let config = envy::from_env::<Config>().expect("Error loading configuration object");
    if let Err(errors) = config.validate() {
        panic!("Missing/Invalid configuration provided: {:?}", errors)
    }

    let db_config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(
        &config.auth_database_url,
    );
    let pool = Pool::builder(db_config).build().unwrap_or_else(|_| {
        panic!(
            "Could not create connection pool with {}",
            &config.auth_database_url
        )
    });
    let redis =
        Client::open(config.auth_redis_url.clone()).expect("Unable to open redis connection");

    tracing_subscriber::fmt::init();

    let app_state = Arc::new(AppState {
        db: pool,
        redis,
        config: config.clone(),
    });

    let services = ServiceBuilder::new().layer(TraceLayer::new_for_http());
    let app = Router::new()
        .merge(routes::root::router(app_state.clone()))
        .merge(routes::v1::router(app_state.clone()))
        .layer(CookieLayer::default())
        .layer(services)
        .with_state(app_state.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Error initializing tcp listener");

    println!("Starting server at {} 🚀🚀", "127.0.0.1:3000");
    axum::serve(listener, app).await.expect("Runtime error");
}
