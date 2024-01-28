use axum::http::StatusCode;
use codegen::ErrorEnum;
use thiserror::Error;

#[derive(ErrorEnum, Error, Debug)]
pub enum CreateCollectionErrorRes {
    #[error("internal server error")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    PoolError(#[from] diesel_async::pooled_connection::deadpool::PoolError),
    #[error("internal server error")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    DieselError(#[from] diesel::result::Error),
}
