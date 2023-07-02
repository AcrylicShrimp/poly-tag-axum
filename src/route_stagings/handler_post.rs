use crate::{app_state::AppState, db::DBPool};
use axum::{debug_handler, extract::State, http::StatusCode, Json};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use dto::*;
use uuid::Uuid;

/// Allocate a new staging area for a file.
#[utoipa::path(
    post,
    operation_id = "staging-post",
    tag = "staging",
    path = "/stagings",
    responses(
        (status = CREATED, description = "Allocate a new staging file", body = StagingPostRes),
        (status = INTERNAL_SERVER_ERROR, description = "An unknown error has occurred during processing", body = ErrorBody, example = json!({ "error": "internal server error" })),
    ),
)]
#[debug_handler(state = AppState)]
pub async fn handle(
    State(db_pool): State<DBPool>,
) -> Result<(StatusCode, Json<StagingPostRes>), ErrRes> {
    use crate::db::schema::stagings::dsl::*;

    let db_connection = &mut db_pool.get().await?;
    let staging_uuid = Uuid::new_v4();
    diesel::insert_into(stagings)
        .values(uuid.eq(staging_uuid))
        .execute(db_connection)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(StagingPostRes { uuid: staging_uuid }),
    ))
}

pub mod dto {
    use axum::http::StatusCode;
    use codegen::ErrorEnum;
    use serde::Serialize;
    use thiserror::Error;
    use utoipa::ToSchema;
    use uuid::Uuid;

    #[derive(Serialize, ToSchema)]
    pub struct StagingPostRes {
        pub uuid: Uuid,
    }

    #[derive(ErrorEnum, Error, Debug)]
    pub enum ErrRes {
        #[error("internal server error")]
        #[status(StatusCode::INTERNAL_SERVER_ERROR)]
        PoolError(#[from] diesel_async::pooled_connection::deadpool::PoolError),
        #[error("internal server error")]
        #[status(StatusCode::INTERNAL_SERVER_ERROR)]
        DieselError(#[from] diesel::result::Error),
    }
}
