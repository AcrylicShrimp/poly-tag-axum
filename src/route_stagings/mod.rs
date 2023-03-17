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
use diesel::{insert_into, prelude::*};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(new_staging))
        .route("/:uuid", get(get_staging))
        .route(
            "/:uuid",
            put(put_staging).layer(DefaultBodyLimit::disable()),
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

    let db_connection = &mut db_pool.get()?;
    let staging_uuid = Uuid::new_v4();
    insert_into(stagings)
        .values(uuid.eq(staging_uuid))
        .execute(db_connection)?;

    Ok((
        StatusCode::CREATED,
        Json(NewStagingResponse { uuid: staging_uuid }),
    ))
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

    let db_connection = &mut db_pool.get()?;
    let staging = stagings
        .select((uuid, staged_at))
        .filter(uuid.eq(param.uuid))
        .first::<Staging>(db_connection)
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
    use crate::db::schema::stagings::dsl::*;

    let db_connection = &mut db_pool.get()?;
    let staging = stagings
        .select((uuid, staged_at))
        .filter(uuid.eq(param.uuid))
        .first::<Staging>(db_connection)
        .optional()?;
    let staging = if let Some(staging) = staging {
        staging
    } else {
        return Err(PutStagingError::NotFound { uuid: param.uuid });
    };

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
        file_name = if let Some(staging_file_name) = field.file_name() {
            Some(staging_file_name.to_owned())
        } else {
            return Err(PutStagingError::InvalidFileName);
        };
        file_size = Some(file_driver.write_staging(param.uuid, offset, field).await?);
    }

    if !field_found {
        return Err(PutStagingError::NoFieldFound);
    }

    Ok((
        StatusCode::OK,
        Json(PutStagingResponse {
            uuid: staging.uuid,
            file_name: file_name.unwrap(),
            staged_size: file_size.unwrap(),
        }),
    ))
}
