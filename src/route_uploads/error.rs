use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum NewUploadError {
    #[error("database error")]
    R2d2Error(#[from] diesel::r2d2::PoolError),
    #[error("database error")]
    DieselError(#[from] diesel::result::Error),
}

impl IntoResponse for NewUploadError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match &self {
            Self::R2d2Error(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::DieselError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        #[cfg(debug_assertions)]
        let error_message = format!("{:?}", self);
        #[cfg(not(debug_assertions))]
        let error_message = self.to_string();

        (status_code, Json(json!({ "error": error_message }))).into_response()
    }
}

pub enum GetUploadParamRejection {
    InvalidUuid,
}

impl IntoResponse for GetUploadParamRejection {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid uuid" })),
        )
            .into_response()
    }
}

#[derive(Debug, Error)]
pub enum GetUploadError {
    #[error("database error")]
    R2d2Error(#[from] diesel::r2d2::PoolError),
    #[error("database error")]
    DieselError(#[from] diesel::result::Error),
    #[error("no upload was found with uuid `{uuid}`")]
    NotFound { uuid: Uuid },
}

impl IntoResponse for GetUploadError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match &self {
            Self::R2d2Error(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::DieselError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
        };

        #[cfg(debug_assertions)]
        let error_message = format!("{:?}", self);
        #[cfg(not(debug_assertions))]
        let error_message = self.to_string();

        (status_code, Json(json!({ "error": error_message }))).into_response()
    }
}
