use redis::{AsyncCommands, Client};
use rusty_paseto::{
    paserk::{K4, PaserkPublic, PaserkSecret},
    prelude::*,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use shared::serde_customs::FlexibleNumber;

use crate::config::{AUDIENCE, ISSUER, SIGNING_REDIS_KEY, VERIFYING_REDIS_KEY};

fn build_base_token<'a>(
    sub: &'a str,
    exp: chrono::DateTime<chrono::Utc>,
) -> PasetoBuilder<'a, V4, Public> {
    let mut builder = PasetoBuilder::<V4, Public>::default();
    builder
        .set_claim(SubjectClaim::from(sub))
        .set_claim(IssuerClaim::from(ISSUER))
        .set_claim(AudienceClaim::from(AUDIENCE))
        .set_claim(ExpirationClaim::try_from(exp.to_rfc3339().as_str()).unwrap());
    builder
}

fn get_paserk_id(secret_key: &Key<64>) -> String {
    let public_bytes: [u8; 32] = secret_key.as_slice()[32..].try_into().unwrap();
    let public_key = Key::<32>::from(public_bytes);
    let paseto_public = PasetoAsymmetricPublicKey::<V4, Public>::from(&public_key);
    paseto_public.paserk_id()
}

fn sign_token<'a>(
    secret_key: &Key<64>,
    mut builder: PasetoBuilder<'a, V4, Public>,
    pid: &'a str,
) -> Result<String, rusty_paseto::Error> {
    let paseto_secret = PasetoAsymmetricPrivateKey::<V4, Public>::from(secret_key);

    let footer = Footer::from(pid);

    builder
        .set_footer(footer)
        .build(&paseto_secret)
        .map_err(rusty_paseto::Error::from)
}

#[derive(Serialize, Deserialize)]
pub struct UserClaims {
    #[serde(alias = "sub")]
    pub id: FlexibleNumber,
    pub username: String,
    pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct RefreshUserClaims {
    #[serde(alias = "sub")]
    pub id: FlexibleNumber,
}

pub fn sign_access_token_for(
    claims: UserClaims,
    secret_key: &Key<64>,
    exp: chrono::DateTime<chrono::Utc>,
) -> Result<String, rusty_paseto::Error> {
    let sub = claims.id.to_string();
    let mut token = build_base_token(&sub, exp);
    token
        .set_claim(CustomClaim::try_from(("email", claims.email.as_str())).unwrap())
        .set_claim(CustomClaim::try_from(("username", claims.username.as_str())).unwrap());
    let pid = get_paserk_id(secret_key);
    sign_token(&secret_key, token, &pid)
}

pub fn sign_refresh_token_for(
    claims: UserClaims,
    secret_key: &Key<64>,
    exp: chrono::DateTime<chrono::Utc>,
) -> Result<String, rusty_paseto::Error> {
    let sub = claims.id.to_string();
    let token = build_base_token(&sub, exp);
    let pid = get_paserk_id(&secret_key);
    sign_token(secret_key, token, &pid)
}

fn verify_token<T: DeserializeOwned>(
    token: &str,
    public_key: &Key<32>,
) -> Result<T, rusty_paseto::Error> {
    let verifier_key = PasetoAsymmetricPublicKey::<V4, Public>::from(public_key);
    let pid = verifier_key.paserk_id();
    let claims: T = PasetoParser::<V4, Public>::default()
        .validate_claim(ExpirationClaim::default(), &|key, value| {
            if key != "exp" {
                return Err(PasetoClaimError::Unexpected(key.to_string()));
            }
            let val = value
                .as_str()
                .ok_or(PasetoClaimError::Unexpected(key.to_string()))?;

            let datetime = chrono::DateTime::parse_from_rfc3339(val).map_err(|_| {
                PasetoClaimError::CustomValidation("Expiration claim is invalid".to_string())
            })?;
            let now = chrono::Utc::now();

            if datetime < now {
                return Err(PasetoClaimError::CustomValidation(
                    "Token expired".to_string(),
                ));
            }
            Ok(())
        })
        .set_footer(Footer::from(pid.as_str()))
        .parse_into(token, &verifier_key)
        .map_err(rusty_paseto::Error::from)?;
    Ok(claims)
}
pub fn verify_access_token(
    token: &str,
    public_key: &Key<32>,
) -> Result<UserClaims, rusty_paseto::Error> {
    verify_token(token, public_key)
}
pub fn verify_refresh_token(
    token: &str,
    public_key: &Key<32>,
) -> Result<RefreshUserClaims, rusty_paseto::Error> {
    verify_token(token, public_key)
}
pub async fn get_signing_key_from_redis(redis: &Client) -> Key<64> {
    let mut conn = redis.get_multiplexed_async_connection().await.unwrap();
    let signing_key: String = conn.get(&SIGNING_REDIS_KEY).await.unwrap();
    let secret_key = PaserkSecret::<K4>::try_from(signing_key).unwrap();
    let secret_key: Result<Key<64>, PasetoError> = Key::try_from(secret_key.as_bytes());
    secret_key.unwrap()
}

pub async fn get_verifier_key_for(kid: &str, redis: &Client) -> Option<Key<32>> {
    let mut conn = redis.get_multiplexed_async_connection().await.unwrap();
    let redis_key = format!("{}:{}", &VERIFYING_REDIS_KEY, kid);
    let verifier_key: Option<String> = conn.hget(&redis_key, "key").await.unwrap();
    if verifier_key.is_none() {
        return None;
    }
    let public_key = PaserkPublic::<K4>::try_from(verifier_key.unwrap()).unwrap();
    let public_key = Key::<32>::try_from(public_key.as_bytes());
    Some(public_key.unwrap())
}

pub fn extract_kid_from_token(token: &str) -> Option<String> {
    let untrusted = UntrustedToken::try_parse(token).ok()?;
    let footer = untrusted.footer_str().ok()?;
    footer
}
