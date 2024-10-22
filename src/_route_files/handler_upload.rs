use crate::{
    app_state::AppState,
    db::{model::FileHeader, DBPool},
    file_driver::FileDriver,
};
use axum::{
    body::Body,
    debug_handler,
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use dto::*;
use meilisearch_sdk::Client;
use std::sync::Arc;

/// Upload a file.
#[utoipa::path(
    put,
    operation_id = "file-upload",
    tag = "file",
    path = "/files/{uuid}",
    params(
        PathParam,
        QueryParam,
    ),
    responses(
        (status = OK, description = "Upload was successful", body = FileUploadRes),
        (status = INTERNAL_SERVER_ERROR, description = "An unknown error has occurred during processing", body = ErrorBody),
    ),
)]
#[debug_handler(state = AppState)]
pub async fn handle(
    State(db_pool): State<DBPool>,
    State(file_driver): State<FileDriver>,
    State(meilisearch_client): State<Arc<Client>>,
    path_param: Path<PathParam>,
    query_param: Query<QueryParam>,
    body: Body,
) -> Result<(StatusCode, Json<FileUploadRes>), ErrRes> {
    use crate::db::schema::files::dsl as files;

    let db_connection = &mut db_pool.get().await?;
    let file = files::files
        .select((files::uuid, files::name))
        .filter(files::uuid.eq(path_param.uuid))
        .first::<FileHeader>(db_connection)
        .await
        .optional()?;
    let file = if let Some(file) = file {
        file
    } else {
        return Err(ErrRes::FileNotFound(path_param.uuid));
    };

    let file_size = file_driver
        .write_file(file.uuid, query_param.offset, body.into_data_stream())
        .await?;
    let file_info = file_driver.read_file_info(file.uuid).await?;
    let now = Utc::now().naive_utc();

    diesel::update(files::files.filter(files::uuid.eq(file.uuid)))
        .set((
            files::mime.eq(file_info.mime),
            files::size.eq(file_size as i64),
            files::hash.eq(file_info.hash as i64),
            files::uploaded_at.eq(now),
        ))
        .execute(db_connection)
        .await?;
    meilisearch_client
        .index("files")
        .add_documents(&[&file], Some("uuid"))
        .await?;

    Ok((
        StatusCode::OK,
        Json(FileUploadRes {
            uuid: file.uuid,
            name: file.name,
            mime: file_info.mime,
            size: file_size,
            hash: file_info.hash,
            uploaded_at: now,
        }),
    ))
}

pub mod dto {
    use crate::file_driver::{ReadFileInfoError, WriteFileError};
    use axum::http::StatusCode;
    use chrono::NaiveDateTime;
    use codegen::ErrorEnum;
    use serde::{Deserialize, Serialize};
    use thiserror::Error;
    use utoipa::{IntoParams, ToSchema};
    use uuid::Uuid;

    #[derive(Deserialize, IntoParams)]
    #[serde(rename_all = "camelCase")]
    #[into_params(parameter_in = Path)]
    pub struct PathParam {
        pub uuid: Uuid,
    }

    #[derive(Deserialize, IntoParams)]
    #[serde(rename_all = "camelCase")]
    #[into_params(parameter_in = Query)]
    pub struct QueryParam {
        pub offset: Option<u64>,
    }

    #[derive(Serialize, ToSchema)]
    #[serde(rename_all = "camelCase")]
    pub struct FileUploadRes {
        pub uuid: Uuid,
        #[schema(example = "file.txt")]
        pub name: String,
        #[schema(example = "text/plain")]
        pub mime: &'static str,
        #[schema(example = "1024")]
        pub size: u64,
        #[schema(example = "1234")]
        pub hash: u32,
        pub uploaded_at: NaiveDateTime,
    }

    #[derive(ErrorEnum, Error, Debug)]
    pub enum ErrRes {
        #[error("internal server error")]
        #[status(StatusCode::INTERNAL_SERVER_ERROR)]
        PoolError(#[from] diesel_async::pooled_connection::deadpool::PoolError),
        #[error("internal server error")]
        #[status(StatusCode::INTERNAL_SERVER_ERROR)]
        DieselError(#[from] diesel::result::Error),
        #[error("internal server error")]
        #[status(StatusCode::INTERNAL_SERVER_ERROR)]
        MeilisearchError(#[from] meilisearch_sdk::errors::Error),
        #[error("file `{0}` is not found")]
        #[status(StatusCode::NOT_FOUND)]
        FileNotFound(Uuid),
        #[error("{0}")]
        #[status("0")]
        WriteFileError(#[from] WriteFileError),
        #[error("{0}")]
        #[status("0")]
        ReadFileInfoError(#[from] ReadFileInfoError),
    }
}
