use std::{collections::HashMap, sync::Arc};

use rusty_paseto::{
    paserk::{K4, PaserkPublic},
    prelude::*,
};
use serde::{Deserialize, de::DeserializeOwned};
use tokio::sync::RwLock;
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

#[derive(Clone)]
pub struct PaserkClient {
    paserk_keys_endpoint: String,

    cached_paserk_keys: Arc<RwLock<HashMap<String, String>>>,
}

impl PaserkClient {
    pub fn new(paserk_keys_endpoint: &str) -> Self {
        Self {
            paserk_keys_endpoint: paserk_keys_endpoint.to_string(),
            cached_paserk_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn validate_token<T>(&mut self, token: &str) -> Result<T, PaserkClientError>
    where
        T: DeserializeOwned,
    {
        let unverified_token =
            UntrustedToken::try_parse(token).map_err(|_| PaserkClientError::InvalidToken)?;
        let kid = unverified_token
            .footer_str()
            .map_err(|_| PaserkClientError::InvalidToken)?
            .ok_or(PaserkClientError::MissingFooter)?;
        let verifier_key = self.get_key_from_cache(&kid).await?;
        let verifier_key = PaserkPublic::<K4>::try_from(verifier_key)
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

    async fn get_key_from_cache(&mut self, kid: &str) -> Result<String, PaserkClientError> {
        let cache = self.cached_paserk_keys.read().await;
        let key = cache.get(&kid.to_string());

        if let Some(key) = key {
            return Ok(key.to_string());
        }
        drop(cache);
        let mut cache = self.cached_paserk_keys.write().await;

        let client = reqwest::Client::new();

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

        cache.clear();
        keys_response.keys.into_iter().for_each(|k| {
            cache.insert(k.kid, k.key);
        });

        Ok(cache
            .get(&kid.to_string())
            .ok_or(PaserkClientError::UnknownKey)?
            .clone())
    }
}
