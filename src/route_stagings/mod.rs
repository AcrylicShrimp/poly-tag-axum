pub mod dto;
pub mod error;

use self::{dto::*, error::*};
use crate::{
    app_state::AppState,
    db::{model::*, DbPool},
    file_driver::FileDriver,
};
use axum::{
    extract::{DefaultBodyLimit, Multipart, State},
    headers::ContentRange,
    http::StatusCode,
    routing::{get, post, put},
    Json, Router, TypedHeader,
};
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use smartstring::alias::String;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/stagings", post(new_staging))
        .route("/stagings/:uuid", get(get_staging))
        .route(
            "/stagings/:uuid",
            // put(put_staging).layer(DefaultBodyLimit::disable()),
            put(put_staging),
        )
}

#[utoipa::path(
    tag = "staging",
    post,
    path = "/stagings",
    responses(
        (status = CREATED, description = "Allocate a new staging file", body = NewStagingResponse),
        (status = INTERNAL_SERVER_ERROR, description = "An unknown error has occurred during processing", body = ErrorBody, example = json!({ "error": "database error" }))
    )
)]
async fn new_staging(
    State(db_pool): State<DbPool>,
) -> Result<(StatusCode, Json<NewStagingResponse>), NewStagingError> {
    use crate::db::schema::stagings::dsl::*;

    let db_connection = &mut db_pool.get().await?;
    db_connection
        .transaction::<_, NewStagingError, _>(|db_connection| {
            async move {
                let staging_uuid = Uuid::new_v4();
                diesel::insert_into(stagings)
                    .values(uuid.eq(staging_uuid))
                    .execute(db_connection)
                    .await?;

                Ok((
                    StatusCode::CREATED,
                    Json(NewStagingResponse { uuid: staging_uuid }),
                ))
            }
            .scope_boxed()
        })
        .await
}

#[utoipa::path(
    tag = "staging",
    get,
    path = "/stagings/{uuid}",
    params(
        ("uuid" = uuid::Uuid, Path, description = "UUID of the staging file")
    ),
    responses(
        (status = OK, description = "Query for an existing staging file", body = GetStagingResponse),
        (status = BAD_REQUEST, description = "Given uuid is not valid", body = ErrorBody, example = json!({ "error": "invalid uuid" })),
        (status = NOT_FOUND, description = "Staging was not found", body = ErrorBody, example = json!({ "error": "staging was not found with uuid `3fa85f64-5717-4562-b3fc-2c963f66afa6`" })),
        (status = INTERNAL_SERVER_ERROR, description = "An unknown error has occurred during processing", body = ErrorBody, example = json!({ "error": "database error" }))
    )
)]
async fn get_staging(
    State(db_pool): State<DbPool>,
    State(file_driver): State<FileDriver>,
    param: GetStagingParam,
) -> Result<(StatusCode, Json<GetStagingResponse>), GetStagingError> {
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
        return Err(GetStagingError::NotFound { uuid: param.uuid });
    };

    Ok((
        StatusCode::OK,
        Json(GetStagingResponse {
            uuid: staging.uuid,
            staged_size: file_driver
                .read_staging_size(staging.uuid)
                .await?
                .unwrap_or_default(),
            staged_at: staging.staged_at,
        }),
    ))
}

async fn put_staging(
    State(db_pool): State<DbPool>,
    State(file_driver): State<FileDriver>,
    content_range: Option<TypedHeader<ContentRange>>,
    param: PutStagingParam,
    mut body: Multipart,
) -> Result<(StatusCode, Json<PutStagingResponse>), PutStagingError> {
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
                    return Err(PutStagingError::NotFound { uuid: param.uuid });
                }

                let offset = content_range
                    .and_then(|content_range| content_range.bytes_range())
                    .map(|range| range.0);

                let mut field_found = false;
                let mut file_name = None;
                let mut file_size = None;

                while let Some(field) = body.next_field().await? {
                    if field_found {
                        return Err(PutStagingError::MultipleFieldFound);
                    }

                    field_found = true;
                    file_name = if let Some(file_name) = field.file_name() {
                        Some(String::from(file_name))
                    } else {
                        return Err(PutStagingError::InvalidFileName);
                    };
                    file_size = Some(file_driver.write_staging(param.uuid, offset, field).await?);
                }

                if !field_found {
                    return Err(PutStagingError::NoFieldFound);
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
                    Json(PutStagingResponse {
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
