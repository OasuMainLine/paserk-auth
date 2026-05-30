use rusty_paseto::{
    paserk::{K4, PaserkPublic},
    prelude::*,
};
use serde::{Deserialize, de::DeserializeOwned};
#[derive(thiserror::Error, Debug)]
pub enum PaserkClientError {
    #[error("Could not reach server")]
    NetworkError(#[from] reqwest::Error),
    #[error("Could not parse key response. Check that {0} is a paserk uri")]
    ParseError(String),

    #[error("Could not parse claims into given type")]
    ClaimParseError,
    #[error("Could not assess reason for the error")]
    UnknownError,

    #[error("The provided token was invalid")]
    InvalidToken,

    #[error("The provided key doesn't have a footer. Unable to verify")]
    MissingFooter,
    #[error("The key has a valid format, but its origin is unknown")]
    UnknownKey,
}

#[derive(Deserialize)]
struct KeyItem {
    pub key: String,
    pub kid: String,
}
#[derive(Deserialize)]
struct KeysResponse {
    pub keys: Vec<KeyItem>,
}
pub struct PaserkClient {
    paserk_keys_endpoint: String,
}

impl PaserkClient {
    pub fn new(paserk_keys_endpoint: &str) -> Self {
        Self {
            paserk_keys_endpoint: paserk_keys_endpoint.to_string(),
        }
    }

    pub async fn validate_token<T>(&self, token: &str) -> Result<T, PaserkClientError>
    where
        T: DeserializeOwned,
    {
        let client = reqwest::Client::new();
        let unverified_token =
            UntrustedToken::try_parse(token).map_err(|_| PaserkClientError::InvalidToken)?;
        let footer = unverified_token
            .footer_str()
            .map_err(|_| PaserkClientError::InvalidToken)?
            .ok_or(PaserkClientError::MissingFooter)?;

        let keys_response = client
            .get(&self.paserk_keys_endpoint)
            .send()
            .await
            .map_err(PaserkClientError::from)?;

        let keys_response: KeysResponse = keys_response.json().await.map_err(|e| {
            if e.is_decode() {
                return PaserkClientError::ParseError(self.paserk_keys_endpoint.to_string());
            }
            PaserkClientError::UnknownError
        })?;

        let verifier_key = keys_response
            .keys
            .into_iter()
            .find(|key| key.kid == footer)
            .ok_or(PaserkClientError::UnknownKey)?;

        let verifier_key = PaserkPublic::<K4>::try_from(verifier_key.key)
            .map_err(|_| PaserkClientError::UnknownError)?;
        let public_key = Key::<32>::try_from(verifier_key.as_bytes())
            .map_err(|_| PaserkClientError::UnknownError)?;

        let verifier_key = PasetoAsymmetricPublicKey::<V4, Public>::from(&public_key);
        let kid = verifier_key.paserk_id();
        let claims: T = PasetoParser::<V4, Public>::default()
            .set_footer(Footer::from(kid.as_str()))
            .parse_into(token, &verifier_key)
            .map_err(|_| PaserkClientError::ClaimParseError)?;

        Ok(claims)
    }
}
