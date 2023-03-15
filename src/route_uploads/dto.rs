use super::error::GetUploadParamRejection;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct NewUploadResponse {
    pub uuid: Uuid,
}

#[derive(Deserialize, IntoParams)]
pub struct GetUploadParam {
    pub uuid: Uuid,
}

#[async_trait]
impl<S> FromRequestParts<S> for GetUploadParam
where
    S: Send + Sync,
{
    type Rejection = GetUploadParamRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let uuid = axum::extract::Path::<String>::from_request_parts(parts, state)
            .await
            .map_err(|_| Self::Rejection::InvalidUuid)?;
        let uuid = Uuid::parse_str(&uuid).map_err(|_| Self::Rejection::InvalidUuid)?;
        Ok(Self { uuid })
    }
}

#[derive(Serialize, ToSchema)]
pub struct GetUploadResponse {
    pub uuid: Uuid,
    pub file_name: Option<String>,
    pub uploaded_size: i64,
    pub uploaded_at: NaiveDateTime,
}
