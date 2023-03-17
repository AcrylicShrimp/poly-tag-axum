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
use smartstring::alias::String;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct NewStagingResponse {
    pub uuid: Uuid,
}

#[derive(Deserialize, IntoParams)]
pub struct GetStagingParam {
    pub uuid: Uuid,
}

#[async_trait]
impl<S> FromRequestParts<S> for GetStagingParam
where
    S: Send + Sync,
{
    type Rejection = GetStagingParamRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let uuid = axum::extract::Path::<String>::from_request_parts(parts, state)
            .await
            .map_err(|_| Self::Rejection::InvalidUuid)?;
        let uuid = Uuid::parse_str(&uuid).map_err(|_| Self::Rejection::InvalidUuid)?;
        Ok(Self { uuid })
    }
}

pub enum GetStagingParamRejection {
    InvalidUuid,
}

impl IntoResponse for GetStagingParamRejection {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid uuid" })),
        )
            .into_response()
    }
}

#[derive(Serialize, ToSchema)]
pub struct GetStagingResponse {
    pub uuid: Uuid,
    pub staged_size: u64,
    pub staged_at: NaiveDateTime,
}

#[derive(Deserialize, IntoParams)]
pub struct PutStagingParam {
    pub uuid: Uuid,
}

#[async_trait]
impl<S> FromRequestParts<S> for PutStagingParam
where
    S: Send + Sync,
{
    type Rejection = PutStagingParamRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let uuid = axum::extract::Path::<String>::from_request_parts(parts, state)
            .await
            .map_err(|_| Self::Rejection::InvalidUuid)?;
        let uuid = Uuid::parse_str(&uuid).map_err(|_| Self::Rejection::InvalidUuid)?;
        Ok(Self { uuid })
    }
}

pub enum PutStagingParamRejection {
    InvalidUuid,
}

impl IntoResponse for PutStagingParamRejection {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid uuid" })),
        )
            .into_response()
    }
}

#[derive(Serialize, ToSchema)]
pub struct PutStagingResponse {
    pub uuid: Uuid,
    pub name: String,
    pub mime: &'static str,
    pub size: u64,
    pub hash: u32,
    pub uploaded_at: NaiveDateTime,
}
