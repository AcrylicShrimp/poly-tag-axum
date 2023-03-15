pub mod dto;
pub mod error;

use self::{
    dto::{GetUploadParam, GetUploadResponse},
    error::GetUploadError,
};
use crate::{
    app_state::AppState,
    db::{model::*, DbPool},
};
use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use diesel::prelude::*;

pub fn router() -> Router<AppState> {
    Router::new().route("/:uuid", get(get_upload))
}

#[utoipa::path(
    tag = "uploading",
    get,
    path = "/uploads/{uuid}",
    params(
        ("uuid" = uuid::Uuid, Path, description = "UUID of the uploading file")
    ),
    responses(
        (status = OK, description = "Get an uploading file", body = GetUploadResponse),
        (status = BAD_REQUEST, description = "Invalid uuid", body = ErrorBody, example = json!({ "error": "invalid uuid" })),
        (status = NOT_FOUND, description = "Uploading file was not found", body = ErrorBody, example = json!({ "error": "no upload was found with uuid `3fa85f64-5717-4562-b3fc-2c963f66afa6`" })),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error", body = ErrorBody, example = json!({ "error": "database error" }))
    )
)]
async fn get_upload(
    State(db_pool): State<DbPool>,
    param: GetUploadParam,
) -> Result<(StatusCode, Json<GetUploadResponse>), GetUploadError> {
    use crate::db::schema::uploads::dsl::*;

    let db_connection = &mut db_pool.get()?;
    let upload = uploads
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
            file_name: upload.file_name,
            uploaded_size: upload.uploaded_size,
            uploaded_at: upload.uploaded_at,
        }),
    ))
}
