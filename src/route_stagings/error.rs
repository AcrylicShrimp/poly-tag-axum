use crate::file_driver::{ReadStagingSizeError, WriteStagingError};
use axum::{extract::multipart::MultipartError, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum NewStagingError {
    #[error("database error")]
    R2d2Error(#[from] diesel::r2d2::PoolError),
    #[error("database error")]
    DieselError(#[from] diesel::result::Error),
}

impl IntoResponse for NewStagingError {
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

#[derive(Debug, Error)]
pub enum GetStagingError {
    #[error("database error")]
    R2d2Error(#[from] diesel::r2d2::PoolError),
    #[error("database error")]
    DieselError(#[from] diesel::result::Error),
    #[error("staging was not found with uuid `{uuid}`")]
    NotFound { uuid: Uuid },
    #[error("internal error")]
    FileDriverError(#[from] ReadStagingSizeError),
}

impl IntoResponse for GetStagingError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match &self {
            Self::R2d2Error(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::DieselError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::FileDriverError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        #[cfg(debug_assertions)]
        let error_message = format!("{:?}", self);
        #[cfg(not(debug_assertions))]
        let error_message = self.to_string();

        (status_code, Json(json!({ "error": error_message }))).into_response()
    }
}

#[derive(Debug, Error)]
pub enum PutStagingError {
    #[error("database error")]
    R2d2Error(#[from] diesel::r2d2::PoolError),
    #[error("database error")]
    DieselError(#[from] diesel::result::Error),
    #[error("staging was not found with uuid `{uuid}`")]
    NotFound { uuid: Uuid },
    #[error("invalid multipart request")]
    MultipartError(#[from] MultipartError),
    #[error("multiple fields were found; only one field is allowed")]
    MultipleFieldFound,
    #[error("invalid filename; it must be a valid filename")]
    InvalidFileName,
    #[error("field was no found; a field is required")]
    NoFieldFound,
    #[error("internal error")]
    FileDriverError(#[from] WriteStagingError<MultipartError>),
}

impl IntoResponse for PutStagingError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match &self {
            Self::R2d2Error(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::DieselError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::MultipartError(_) => StatusCode::BAD_REQUEST, // TODO: is this the right status code?
            Self::MultipleFieldFound => StatusCode::BAD_REQUEST,
            Self::InvalidFileName => StatusCode::BAD_REQUEST,
            Self::NoFieldFound => StatusCode::BAD_REQUEST,
            Self::FileDriverError(err) => match err {
                WriteStagingError::CreateFile(..) => StatusCode::INTERNAL_SERVER_ERROR,
                WriteStagingError::ReadFileMetadata(..) => StatusCode::INTERNAL_SERVER_ERROR,
                WriteStagingError::InvalidOffset { offset, file_size } => {
                    return (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(json!({
                            "error": format!("invalid offset; expected offset to be less than or equal to file size; offset: {}, file size: {}", offset, file_size),
                        }))
                    )
                        .into_response();
                }
                WriteStagingError::ReadFromStream(..) => StatusCode::INTERNAL_SERVER_ERROR,
                WriteStagingError::WriteToFile(..) => StatusCode::INTERNAL_SERVER_ERROR,
            },
        };

        #[cfg(debug_assertions)]
        let error_message = format!("{:?}", self);
        #[cfg(not(debug_assertions))]
        let error_message = self.to_string();

        (status_code, Json(json!({ "error": error_message }))).into_response()
    }
}
