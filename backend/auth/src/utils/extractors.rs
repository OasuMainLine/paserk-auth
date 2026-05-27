use std::sync::Arc;

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use axum_cookie::CookieManager;
use diesel_async::{AsyncPgConnection, pooled_connection::deadpool::Object};
use log::error;

use crate::{
    errors::AuthServiceError,
    state::AppState,
    utils::paseto::{UserClaims, extract_kid_from_token, get_verifier_key_for, verify_token},
};

pub struct Database(pub Object<AsyncPgConnection>);

impl FromRequestParts<Arc<AppState>> for Database {
    type Rejection = Response;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let conn = state
            .db
            .get()
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR).into_response())?;

        Ok(Self(conn))
    }
}

pub struct Session(pub UserClaims);

impl FromRequestParts<Arc<AppState>> for Session {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let cookie = CookieManager::from_request_parts(parts, state)
            .await
            .expect("Error accessing cookies");

        let auth_cookie = cookie.get("x-auth-access-token").ok_or(
            AuthServiceError::UnauthorizedError(Some(String::from(
                "Authorization cookie not present",
            )))
            .into_response(),
        )?;

        let kid = extract_kid_from_token(auth_cookie.value()).ok_or(
            AuthServiceError::UnauthorizedError(Some(String::from("Invalid or expired token")))
                .into_response(),
        )?;

        let public_key = get_verifier_key_for(&kid, &state.redis).await.ok_or(
            AuthServiceError::UnauthorizedError(Some(String::from("Invalid or expired token")))
                .into_response(),
        )?;

        let claims = verify_token(auth_cookie.value(), &public_key).map_err(|e| {
            error!("Error validating token: {}", e);
            AuthServiceError::UnauthorizedError(Some(String::from("Invalid or expired token")))
                .into_response()
        })?;

        Ok(Self(claims))
    }
}
