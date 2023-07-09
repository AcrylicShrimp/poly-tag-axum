use crate::{
    app_state::AppState,
    db::{model::Staging, DBPool},
    file_driver::FileDriver,
};
use axum::{
    debug_handler,
    extract::{Multipart, State},
    headers::ContentRange,
    http::StatusCode,
    Json, TypedHeader,
};
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use dto::*;
use uuid::Uuid;

/// Upload a file to the staging area.
#[utoipa::path(
    put,
    tag = "staging",
    path = "/stagings/{uuid}",
    params(
        ("uuid" = uuid::Uuid, Path, description = "UUID of the staging file")
    ),
    responses(
        (status = OK, description = "Query for an existing staging file", body = StagingPutRes),
        (status = BAD_REQUEST, description = "Given uuid is not valid", body = ErrorBody, example = json!({ "error": "invalid uuid" })),
        (status = NOT_FOUND, description = "Staging was not found", body = ErrorBody, example = json!({ "error": "staging was not found with uuid `3fa85f64-5717-4562-b3fc-2c963f66afa6`" })),
        (status = INTERNAL_SERVER_ERROR, description = "An unknown error has occurred during processing", body = ErrorBody)
    )
)]
#[debug_handler(state = AppState)]
pub async fn handle(
    State(db_pool): State<DBPool>,
    State(file_driver): State<FileDriver>,
    content_range: Option<TypedHeader<ContentRange>>,
    param: Param,
    mut body: Multipart,
) -> Result<(StatusCode, Json<StagingPutRes>), ErrRes> {
    use crate::db::schema::files::dsl as files;
    use crate::db::schema::stagings::dsl as stagings;

    let db_connection = &mut db_pool.get().await?;
    db_connection
        .transaction(|db_connection| {
            async move {
                let staging = stagings::stagings
                    .for_update()
                    .select((stagings::uuid, stagings::staged_at))
                    .filter(stagings::uuid.eq(param.uuid))
                    .first::<Staging>(db_connection)
                    .await
                    .optional()?;

                if staging.is_none() {
                    return Err(ErrRes::NotFound { uuid: param.uuid });
                }

                let offset = content_range
                    .and_then(|content_range| content_range.bytes_range())
                    .map(|range| range.0);

                // MultipartParser::new(
                //     Some(1),
                //     Some(&|_index, field| -> Result<bool, ()> { Ok(field.file_name().is_some()) }),
                //     &|_index, field| -> Result<(), ()> { Ok(()) },
                // )
                // .parse(body)
                // .await?;

                let mut field_found = false;
                let mut file_name = None;
                let mut file_size = None;

                while let Some(field) = body.next_field().await? {
                    if field_found {
                        return Err(ErrRes::MultipleFieldFound);
                    }

                    field_found = true;
                    file_name = if let Some(file_name) = field.file_name() {
                        Some(String::from(file_name))
                    } else {
                        return Err(ErrRes::InvalidFileName);
                    };
                    file_size = Some(file_driver.write_staging(param.uuid, offset, field).await?);
                }

                if !field_found {
                    return Err(ErrRes::NoFieldFound);
                }

                let uuid = Uuid::new_v4();
                let file_name = file_name.unwrap();
                let file_size = file_size.unwrap();
                let info = file_driver.read_staging_info(param.uuid).await?;
                let now = Utc::now().naive_utc();

                diesel::insert_into(files::files)
                    .values((
                        files::uuid.eq(uuid),
                        files::name.eq(file_name.as_str()),
                        files::mime.eq(&info.mime),
                        files::size.eq(file_size as i64),
                        files::hash.eq(info.hash as i64),
                        files::uploaded_at.eq(now),
                    ))
                    .execute(db_connection)
                    .await?;

                // TODO: Transfer file from staging to storage.
                // TODO: Delete staging database record.

                Ok((
                    StatusCode::OK,
                    Json(StagingPutRes {
                        uuid,
                        name: file_name.into(),
                        mime: info.mime,
                        size: file_size,
                        hash: info.hash,
                        uploaded_at: now,
                    }),
                ))
            }
            .scope_boxed()
        })
        .await
}

pub mod dto {
    use crate::file_driver::{ReadStagingInfoError, WriteStagingError};
    use axum::{
        async_trait,
        extract::{multipart::MultipartError, FromRequestParts, Path},
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
                .map_err(|_| ParamRejection::InvalidUuid)?;
            let uuid = Uuid::parse_str(&uuid).map_err(|_| ParamRejection::InvalidUuid)?;

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
    pub struct StagingPutRes {
        pub uuid: Uuid,
        pub name: String,
        pub mime: &'static str,
        pub size: u64,
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
        #[error("staging `{uuid}` was not found")]
        #[status(StatusCode::NOT_FOUND)]
        NotFound { uuid: Uuid },
        #[error("{0}")]
        #[status("0")]
        MultipartError(#[from] MultipartError),
        #[error("multiple fields were found; only one field is allowed")]
        #[status(StatusCode::BAD_REQUEST)]
        MultipleFieldFound,
        #[error("invalid filename; it must be a valid filename")]
        #[status(StatusCode::BAD_REQUEST)]
        InvalidFileName,
        #[error("field was no found; a field is required")]
        #[status(StatusCode::BAD_REQUEST)]
        NoFieldFound,
        #[error("{0}")]
        #[status("0")]
        WriteStagingError(#[from] WriteStagingError),
        #[error("{0}")]
        #[status("0")]
        ReadStagingInfoError(#[from] ReadStagingInfoError),
    }
}
