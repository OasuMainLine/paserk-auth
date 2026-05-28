use axum::{
    RequestPartsExt,
    extract::{FromRequest, MatchedPath, Request, rejection::JsonRejection},
};
use serde_json::json;
use validator::Validate;

use crate::{responses::ApiFail, validation::AxumValidator};

pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    T: Validate,
    axum::Json<T>: FromRequest<S, Rejection = JsonRejection>,
    S: Send + Sync,
{
    type Rejection = ApiFail;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();

        let path = parts
            .extract::<MatchedPath>()
            .await
            .map(|path| path.as_str().to_owned())
            .ok();

        let req = Request::from_parts(parts, body);

        match axum::Json::<T>::from_request(req, state).await {
            Ok(value) => AxumValidator::validate(&value.0).map(|_| Self(value.0)),
            Err(rejection) => {
                let payload = json!({
                    "message": rejection.body_text(),
                    "origin": "validated_json_extractor",
                    "path": path,
                });
                Err(ApiFail::unprocessable_entity(payload))
            }
        }
    }
}
