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
        .route("/", post(new_upload))
        .route("/:uuid", get(get_upload))
        .route("/:uuid", put(put_upload).layer(DefaultBodyLimit::disable()))
}

#[utoipa::path(
    tag = "uploading",
    post,
    path = "/uploads",
    responses(
        (status = CREATED, description = "Allocate a new uploading file", body = NewUploadResponse),
        (status = INTERNAL_SERVER_ERROR, description = "An unknown error has occurred during processing", body = ErrorBody, example = json!({ "error": "database error" }))
    )
)]
async fn new_upload(
    State(db_pool): State<DbPool>,
) -> Result<(StatusCode, Json<NewUploadResponse>), NewUploadError> {
    use crate::db::schema::uploads::dsl::*;

    let db_connection = &mut db_pool.get()?;
    let upload_uuid = Uuid::new_v4();
    insert_into(uploads)
        .values(uuid.eq(upload_uuid))
        .execute(db_connection)?;

    Ok((
        StatusCode::CREATED,
        Json(NewUploadResponse { uuid: upload_uuid }),
    ))
}

#[utoipa::path(
    tag = "uploading",
    get,
    path = "/uploads/{uuid}",
    params(
        ("uuid" = uuid::Uuid, Path, description = "UUID of the uploading file")
    ),
    responses(
        (status = OK, description = "Query for an existing uploading file", body = GetUploadResponse),
        (status = BAD_REQUEST, description = "Given uuid is not valid", body = ErrorBody, example = json!({ "error": "invalid uuid" })),
        (status = NOT_FOUND, description = "Uploading file was not found", body = ErrorBody, example = json!({ "error": "no upload was found with uuid `3fa85f64-5717-4562-b3fc-2c963f66afa6`" })),
        (status = INTERNAL_SERVER_ERROR, description = "An unknown error has occurred during processing", body = ErrorBody, example = json!({ "error": "database error" }))
    )
)]
async fn get_upload(
    State(db_pool): State<DbPool>,
    State(file_driver): State<FileDriver>,
    param: GetUploadParam,
) -> Result<(StatusCode, Json<GetUploadResponse>), GetUploadError> {
    use crate::db::schema::uploads::dsl::*;

    let db_connection = &mut db_pool.get()?;
    let upload = uploads
        .select((uuid, created_at))
        .filter(uuid.eq(param.uuid))
        .first::<Upload>(db_connection)
        .optional()?;
    let upload = if let Some(upload) = upload {
        upload
    } else {
        return Err(GetUploadError::NotFound { uuid: param.uuid });
    };

    Ok((
        StatusCode::OK,
        Json(GetUploadResponse {
            uuid: upload.uuid,
            uploaded_size: file_driver
                .read_file_size(upload.uuid)
                .await?
                .unwrap_or_default(),
            created_at: upload.created_at,
        }),
    ))
}

async fn put_upload(
    State(db_pool): State<DbPool>,
    State(file_driver): State<FileDriver>,
    content_range: Option<TypedHeader<ContentRange>>,
    param: PutUploadParam,
    mut body: Multipart,
) -> Result<(StatusCode, Json<PutUploadResponse>), PutUploadError> {
    use crate::db::schema::uploads::dsl::*;

    let db_connection = &mut db_pool.get()?;
    let upload = uploads
        .select((uuid, created_at))
        .filter(uuid.eq(param.uuid))
        .first::<Upload>(db_connection)
        .optional()?;
    let upload = if let Some(upload) = upload {
        upload
    } else {
        return Err(PutUploadError::NotFound { uuid: param.uuid });
    };

    let offset = content_range
        .and_then(|content_range| content_range.bytes_range())
        .map(|range| range.0);

    let mut field_found = false;
    let mut file_name = None;
    let mut file_size = None;

    while let Some(field) = body.next_field().await? {
        if field_found {
            return Err(PutUploadError::MultipleFieldFound);
        }

        field_found = true;
        file_name = if let Some(upload_file_name) = field.file_name() {
            Some(upload_file_name.to_owned())
        } else {
            return Err(PutUploadError::InvalidFileName);
        };
        file_size = Some(file_driver.write_file(param.uuid, offset, field).await?);
    }

    if !field_found {
        return Err(PutUploadError::NoFieldFound);
    }

    Ok((
        StatusCode::OK,
        Json(PutUploadResponse {
            uuid: upload.uuid,
            file_name: file_name.unwrap(),
            uploaded_size: file_size.unwrap(),
        }),
    ))
}
