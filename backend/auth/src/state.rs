use diesel_async::{AsyncPgConnection, pooled_connection::deadpool::Pool};
use redis::Client;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<AsyncPgConnection>,
    pub redis: Client,
    pub config: Config,
}
