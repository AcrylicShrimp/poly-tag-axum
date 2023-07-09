use crate::{
    app_state::AppState,
    db::{model::Staging, DBPool},
    file_driver::FileDriver,
};
use axum::{debug_handler, extract::State, http::StatusCode, Json};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use dto::*;

/// Query for an existing staging area.
#[utoipa::path(
    get,
    operation_id = "staging-get",
    tag = "staging",
    path = "/stagings/{uuid}",
    params(
        ("uuid" = uuid::Uuid, Path, description = "UUID of the staging file")
    ),
    responses(
        (status = OK, description = "Query for an existing staging file", body = StagingGetRes),
        (status = BAD_REQUEST, description = "Given uuid is not valid", body = ErrorBody, example = json!({ "error": "invalid uuid" })),
        (status = NOT_FOUND, description = "Staging was not found", body = ErrorBody, example = json!({ "error": "staging `3fa85f64-5717-4562-b3fc-2c963f66afa6` was not found" })),
        (status = INTERNAL_SERVER_ERROR, description = "An unknown error has occurred during processing", body = ErrorBody)
    )
)]
#[debug_handler(state = AppState)]
pub async fn handle(
    State(db_pool): State<DBPool>,
    State(file_driver): State<FileDriver>,
    param: Param,
) -> Result<(StatusCode, Json<StagingGetRes>), ErrRes> {
    use crate::db::schema::stagings::dsl::*;

    let db_connection = &mut db_pool.get().await?;
    let staging = stagings
        .select((uuid, staged_at))
        .filter(uuid.eq(param.uuid))
        .first::<Staging>(db_connection)
        .await
        .optional()?;
    let staging = if let Some(staging) = staging {
        staging
    } else {
        return Err(ErrRes::NotFound { uuid: param.uuid });
    };

    Ok((
        StatusCode::OK,
        Json(StagingGetRes {
            uuid: staging.uuid,
            staged_size: file_driver
                .read_staging_size(staging.uuid)
                .await?
                .unwrap_or_default(),
            staged_at: staging.staged_at,
        }),
    ))
}

pub mod dto {
    use crate::file_driver::ReadStagingSizeError;
    use axum::{
        async_trait,
        extract::{FromRequestParts, Path},
        http::{request::Parts, StatusCode},
    };
    use chrono::NaiveDateTime;
    use codegen::ErrorEnum;
    use serde::{Deserialize, Serialize};
    use thiserror::Error;
    use utoipa::{IntoParams, ToSchema};
    use uuid::Uuid;

    #[derive(Deserialize, IntoParams)]
    pub struct Param {
        pub uuid: Uuid,
    }

    #[async_trait]
    impl<S> FromRequestParts<S> for Param
    where
        S: Send + Sync,
    {
        type Rejection = ParamRejection;

        async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
            let uuid = Path::<String>::from_request_parts(parts, state)
                .await
                .map_err(|_| Self::Rejection::InvalidUuid)?;
            let uuid = Uuid::parse_str(&uuid).map_err(|_| Self::Rejection::InvalidUuid)?;

            Ok(Self { uuid })
        }
    }

    #[derive(ErrorEnum, Error, Debug)]
    pub enum ParamRejection {
        #[error("invalid uuid")]
        #[status(StatusCode::BAD_REQUEST)]
        InvalidUuid,
    }

    #[derive(Serialize, ToSchema)]
    pub struct StagingGetRes {
        pub uuid: Uuid,
        pub staged_size: u64,
        pub staged_at: NaiveDateTime,
    }

    #[derive(ErrorEnum, Error, Debug)]
    pub enum ErrRes {
        #[error("internal server error")]
        #[status(StatusCode::INTERNAL_SERVER_ERROR)]
        PoolError(#[from] diesel_async::pooled_connection::deadpool::PoolError),
        #[error("internal server error")]
        #[status(StatusCode::INTERNAL_SERVER_ERROR)]
        DieselError(#[from] diesel::result::Error),
        #[error("staging `{uuid}` was not found")]
        #[status(StatusCode::NOT_FOUND)]
        NotFound { uuid: Uuid },
        #[error("{0}")]
        #[status("0")]
        ReadStagingSizeError(#[from] ReadStagingSizeError),
    }
}
