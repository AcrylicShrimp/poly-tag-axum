use crate::{
    file_driver::{
        CommitStagingIntoFileError, ReadStagingInfoError, ReadStagingSizeError, WriteStagingError,
    },
    response::IntoStatus,
};
use axum::{
    extract::multipart::{MultipartError, MultipartRejection},
    http::StatusCode,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileDriverError<E> {
    #[error("internal server error")]
    ReadStagingSizeError(#[from] ReadStagingSizeError),
    #[error("internal server error")]
    WriteStagingError(#[from] WriteStagingError<E>),
    #[error("internal server error")]
    ReadStagingInfoError(#[from] ReadStagingInfoError),
    #[error("internal server error")]
    CommitStagingIntoFileError(#[from] CommitStagingIntoFileError),
}

impl<E> IntoStatus for FileDriverError<E> {
    fn into_status(&self) -> StatusCode {
        match self {
            FileDriverError::ReadStagingSizeError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            FileDriverError::WriteStagingError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            FileDriverError::ReadStagingInfoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            FileDriverError::CommitStagingIntoFileError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Error, Debug)]
pub enum FileRouterError {
    #[error("invalid multipart request")]
    MultipartExtractorRejection(#[from] MultipartRejection),
    #[error("invalid multipart request")]
    MultipartError(#[from] MultipartError),
    #[error("internal server error")]
    IOError(#[from] std::io::Error),
}

impl IntoStatus for FileRouterError {
    fn into_status(&self) -> StatusCode {
        match self {
            FileRouterError::MultipartExtractorRejection(_) => StatusCode::BAD_REQUEST,
            FileRouterError::MultipartError(err) => err.status(),
            FileRouterError::IOError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
