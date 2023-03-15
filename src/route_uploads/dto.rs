use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::json;
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

pub enum GetUploadParamRejection {
    InvalidUuid,
}

impl IntoResponse for GetUploadParamRejection {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid uuid" })),
        )
            .into_response()
    }
}

#[derive(Serialize, ToSchema)]
pub struct GetUploadResponse {
    pub uuid: Uuid,
    pub uploaded_size: u64,
    pub created_at: NaiveDateTime,
}

#[derive(Deserialize, IntoParams)]
pub struct PutUploadParam {
    pub uuid: Uuid,
}

#[async_trait]
impl<S> FromRequestParts<S> for PutUploadParam
where
    S: Send + Sync,
{
    type Rejection = PutUploadParamRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let uuid = axum::extract::Path::<String>::from_request_parts(parts, state)
            .await
            .map_err(|_| Self::Rejection::InvalidUuid)?;
        let uuid = Uuid::parse_str(&uuid).map_err(|_| Self::Rejection::InvalidUuid)?;
        Ok(Self { uuid })
    }
}

pub enum PutUploadParamRejection {
    InvalidUuid,
}

impl IntoResponse for PutUploadParamRejection {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid uuid" })),
        )
            .into_response()
    }
}

#[derive(Serialize, ToSchema)]
pub struct PutUploadResponse {
    pub uuid: Uuid,
    pub file_name: String,
    pub uploaded_size: u64,
}
