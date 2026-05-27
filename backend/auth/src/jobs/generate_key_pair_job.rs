use crate::{
    config::{SIGNING_REDIS_KEY, VERIFYING_REDIS_KEY},
    jobs::schedule_state::ScheduleState,
};
use anyhow::{Context, Error};
use ed25519_dalek::SigningKey;
use log::info;
use rand::rngs::StdRng;
use redis::{AsyncTypedCommands, pipe};
use rusty_paseto::core::Key;
use rusty_paseto::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct GenerateKeyPairJob;

pub async fn generate_key_pair(
    _job: GenerateKeyPairJob,
    state: ScheduleState,
) -> Result<(), Error> {
    info!("jey!");
    let mut csprng = rand::make_rng::<StdRng>();
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    let secret_key = Key::<64>::try_from(signing_key.to_keypair_bytes())?;
    let public_key = Key::try_from(verifying_key.as_bytes())?;
    let signing_key = PasetoAsymmetricPrivateKey::<V4, Public>::from(&secret_key);
    let verifying_key = PasetoAsymmetricPublicKey::<V4, Public>::from(&public_key);

    let verifying_key_exp = chrono::Utc::now()
        + chrono::Duration::minutes(state.config.auth_refresh_token_max_age)
        + chrono::Duration::days(1);

    let mut conn = state
        .redis
        .get_multiplexed_async_connection()
        .await
        .context("Unable to get redis connection")?;

    conn.set(SIGNING_REDIS_KEY, &signing_key.to_paserk_string())
        .await
        .context("Unable to add signing key")?;

    let verifying_redis_key = format!("{}:{}", VERIFYING_REDIS_KEY, verifying_key.paserk_id());

    pipe()
        .hset(
            &verifying_redis_key,
            "key",
            &verifying_key.to_paserk_string(),
        )
        .hset(&verifying_redis_key, "exp", &verifying_key_exp.timestamp())
        .query_async::<()>(&mut conn)
        .await
        .context("Error setting verifying key")?;

    Ok(())
}
