use std::sync::Arc;

use crate::state::AppState;
use axum::{
    Extension,
    extract::{FromRequestParts, Request, State},
    middleware::Next,
    response::{IntoResponse, Result},
};
use axum_cookie::CookieManager;
use log::{error, warn};
use serde::Deserialize;
use serde_json::json;
use shared::{
    paserk::PaserkClientError,
    responses::{ApiError, ApiFail},
    serde_customs::FlexibleNumber,
};

#[derive(Deserialize, Clone, Debug)]
pub struct UserSession {
    #[serde(alias = "sub")]
    pub id: FlexibleNumber,
    pub username: String,
    pub email: String,
}

pub async fn session_middleware(
    State(mut state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse> {
    let (mut parts, body) = request.into_parts();
    let cookie = CookieManager::from_request_parts(&mut parts, &state).await?;
    let auth_cookie = cookie
        .get("x-auth-access-token")
        .ok_or(ApiFail::bad_request(json!({
            "message": "Authentication cookie not found"
        })))?;

    let token = auth_cookie.value();
    let state = Arc::make_mut(&mut state);

    let paserk_client = state
        .paserk_client
        .get_mut()
        .expect("Paserk clietn not initialized");

    let user = paserk_client.validate_token::<UserSession>(token).await;
    if let Err(error) = user {
        match error {
            PaserkClientError::MissingFooter | PaserkClientError::InvalidToken => {
                warn!("Unable to authenticate user {}", error);
                return Err(ApiFail::bad_request(json!({
                    "message": "Invalid token"
                }))
                .into());
            }
            PaserkClientError::ExpiredToken => {
                return Err(ApiFail::unauthorized(json!({
                    "message": "User session expired"
                }))
                .into());
            }
            error => {
                error!("Fatal error authenticating user {}", error);
                return Err(ApiError::server_error("Error authenticating").into());
            }
        }
    }
    let user = user.unwrap();

    let mut request = Request::from_parts(parts, body);
    request.extensions_mut().insert(user);

    let response = next.run(request).await;
    Ok(response)
}

pub type Session = Extension<UserSession>;
