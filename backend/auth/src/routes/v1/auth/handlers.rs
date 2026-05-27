use std::{borrow::Cow, collections::HashMap, sync::Arc};

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Result},
};
use axum_cookie::CookieManager;
use redis::AsyncTypedCommands;
use shared::{extractors::ValidatedJson, responses::ApiResponse};

use diesel::{
    ExpressionMethods, SelectableHelper,
    query_dsl::methods::{FilterDsl, SelectDsl},
};

use diesel_async::RunQueryDsl;
use serde::Deserialize;
use serde_json::json;
use validator::{Validate, ValidationError};

use crate::{
    errors::AuthServiceError,
    models::user::{NewUser, PartialUser, User},
    state::AppState,
    utils::{
        cookies::BaseCookie,
        extractors::{Database, Session},
        paseto::{get_signing_key_from_redis, sign_access_token_for, sign_refresh_token_for},
        passwords::{PasswordError, check_password},
    },
};

#[derive(Deserialize, Validate)]
pub struct SignUpUserSchema {
    #[validate(length(min = 1))]
    username: String,
    #[validate(email)]
    email: String,
    #[validate(length(min = 8), custom(function = "validate_password"))]
    password: String,
}
fn validate_password(password: &str) -> Result<(), ValidationError> {
    match check_password(password) {
        Ok(_) => Ok(()),
        Err(err) => match err {
            PasswordError::WeakPasswordError { feedback } => {
                let feedback = feedback.unwrap_or("Password too weak".to_string());

                Err(ValidationError {
                    code: "password/weak".into(),
                    message: Some(Cow::from(feedback)),
                    params: HashMap::new(),
                })
            }
        },
    }
}

pub async fn sign_up_user(
    cookie: CookieManager,
    Database(mut db): Database,
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<SignUpUserSchema>,
) -> Result<impl IntoResponse> {
    use crate::schema::users;
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .unwrap();

    let user = NewUser {
        username: payload.username,
        email: payload.email,
        password_hash: password_hash.to_string().as_bytes().to_vec(),
        created_at: None,
        id: None,
    };

    let user = diesel::insert_into(users::dsl::users)
        .values(&user)
        .returning(PartialUser::as_returning())
        .get_result(&mut db)
        .await
        .map_err(AuthServiceError::from)?;

    let mut delete_user = async || {
        diesel::delete(users::dsl::users)
            .filter(users::id.eq(&user.id))
            .execute(&mut db)
            .await
    };

    let secret_key = get_signing_key_from_redis(&state.redis).await;
    let access_exp =
        chrono::Utc::now() + chrono::Duration::minutes(state.config.auth_access_token_max_age);
    let refresh_exp =
        chrono::Utc::now() + chrono::Duration::minutes(state.config.auth_refresh_token_max_age);

    let access_token = sign_access_token_for(user.clone().into(), &secret_key, access_exp)
        .map_err(AuthServiceError::from);
    let access_token = match access_token {
        Ok(token) => token,
        Err(e) => {
            delete_user().await.map_err(AuthServiceError::from)?;
            return Err(e.into_response().into());
        }
    };

    let refresh_token = sign_refresh_token_for(user.clone().into(), &secret_key, refresh_exp)
        .map_err(AuthServiceError::from);
    let refresh_token = match refresh_token {
        Ok(token) => token,
        Err(e) => {
            delete_user().await.map_err(AuthServiceError::from)?;
            return Err(e.into_response().into());
        }
    };
    cookie.add(BaseCookie::new(
        "x-auth-access-token",
        &access_token,
        true,
        "*",
    ));
    cookie.add(BaseCookie::new(
        "x-auth-refresh-token",
        &refresh_token,
        true,
        "*",
    ));

    ApiResponse::new()
        .data(json!({
            "user": user,
        }))
        .into()
}

pub async fn logout_user(Session(_): Session, cookie: CookieManager) -> Result<impl IntoResponse> {
    cookie.remove("x-auth-access-token");
    cookie.remove("x-auth-refresh-token");
    Ok(StatusCode::NO_CONTENT)
}

pub async fn verify_user(Session(user): Session) -> Result<impl IntoResponse> {
    ApiResponse::new()
        .data(json!({
            "user": user
        }))
        .into()
}

#[derive(Deserialize, Validate, Debug)]
pub struct SignInUserSchema {
    #[validate(email)]
    email: String,
    #[validate(length(min = 8), custom(function = "validate_password"))]
    password: String,
}
pub async fn sign_in_user(
    State(state): State<Arc<AppState>>,
    Database(mut db): Database,
    cookie: CookieManager,
    ValidatedJson(payload): ValidatedJson<SignInUserSchema>,
) -> Result<impl IntoResponse> {
    use crate::schema::users;
    let user = users::table
        .select(users::all_columns)
        .filter(users::email.eq(payload.email))
        .get_result::<User>(&mut db)
        .await
        .map_err(AuthServiceError::from)
        .map_err(|_| {
            AuthServiceError::UnauthorizedError(Some(String::from("Invalid email or password")))
        })?;

    let hash = String::from_utf8(user.password_hash.clone()).map_err(|_| {
        ApiResponse::new()
            .error("Failed to parse request")
            .status_code(StatusCode::INTERNAL_SERVER_ERROR)
            .into_response()
    })?;
    let hash = PasswordHash::try_from(hash.as_str()).map_err(|_| {
        ApiResponse::new()
            .error("Failed to parse request")
            .status_code(StatusCode::INTERNAL_SERVER_ERROR)
            .into_response()
    })?;

    let argon2 = Argon2::default();
    if argon2
        .verify_password(payload.password.as_bytes(), &hash)
        .is_err()
    {
        return Err(AuthServiceError::UnauthorizedError(Some(String::from(
            "Invalid email or password",
        )))
        .into());
    }

    let user = PartialUser::from(user);
    let secret_key = get_signing_key_from_redis(&state.redis).await;
    let access_exp =
        chrono::Utc::now() + chrono::Duration::minutes(state.config.auth_access_token_max_age);
    let refresh_exp =
        chrono::Utc::now() + chrono::Duration::minutes(state.config.auth_refresh_token_max_age);

    let access_token = sign_access_token_for(user.clone().into(), &secret_key, access_exp)
        .map_err(AuthServiceError::from)?;
    let refresh_token = sign_refresh_token_for(user.clone().into(), &secret_key, refresh_exp)
        .map_err(AuthServiceError::from)?;

    cookie.add(BaseCookie::new(
        "x-auth-access-token",
        &access_token,
        true,
        "*",
    ));
    cookie.add(BaseCookie::new(
        "x-auth-refresh-token",
        &refresh_token,
        true,
        "*",
    ));
    ApiResponse::new()
        .data(json!({
            "user": user,
        }))
        .into()
}

pub async fn get_public_keys(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse> {
    let mut redis = state
        .redis
        .get_multiplexed_async_connection()
        .await
        .map_err(AuthServiceError::from)?;

    let mut keys_iter: redis::AsyncIter<String> = redis
        .scan_match("verifying_key:*")
        .await
        .map_err(AuthServiceError::from)?;
    let mut keys: Vec<String> = Vec::new();

    while let Some(key) = keys_iter.next_item().await {
        let key = key.map_err(AuthServiceError::RedisError)?;
        keys.push(key);
    }
    drop(keys_iter);
    let mut response: Vec<serde_json::Value> = Vec::new();
    for key in keys {
        let kid = key.split(":").last().ok_or(
            ApiResponse::new()
                .error("Could not find kid for existing verifying_key")
                .status_code(StatusCode::INTERNAL_SERVER_ERROR),
        )?;

        let paserk_key = redis
            .hget(&key, "key")
            .await
            .map_err(AuthServiceError::from)?
            .ok_or(
                ApiResponse::new()
                    .error("Could not find verifying_key value")
                    .status_code(StatusCode::INTERNAL_SERVER_ERROR),
            )?;

        response.push(json!({
            "kid": kid,
            "key": paserk_key
        }));
    }

    return Ok(Json(json!({
        "keys": response
    })));
}
