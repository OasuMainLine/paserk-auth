use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate, Clone, Debug)]
pub struct Config {
    #[validate(url)]
    pub resource_paserk_url: String,
    #[validate(url)]
    pub resource_database_url: String,
    pub resource_service_port: i64,
}
