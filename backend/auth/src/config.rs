use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub struct Config {
    #[validate(url)]
    pub auth_database_url: String,
    #[validate(url)]
    pub auth_redis_url: String,

    pub auth_refresh_token_max_age: i64,
    pub auth_access_token_max_age: i64,
}

pub const SIGNING_REDIS_KEY: &'static str = "signing_key";
pub const VERIFYING_REDIS_KEY: &'static str = "verifying_key";
pub const ISSUER: &'static str = "auth_service";
pub const AUDIENCE: &'static str = "example";
