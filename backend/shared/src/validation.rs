use serde_json::json;
use validator::Validate;

use crate::responses::ApiFail;

pub struct AxumValidator;

impl AxumValidator {
    pub fn validate(schema: &impl Validate) -> Result<(), ApiFail> {
        schema
            .validate()
            .map_err(|errors| ApiFail::unprocessable_entity(json!(errors)))
    }
}
