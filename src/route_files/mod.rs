use crate::{app_state::AppState, errors::FileRouterError};
use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Multipart},
    http::StatusCode,
    routing::post,
    Json, Router,
};
use axum_extra::extract::WithRejection;
use futures::{Stream, TryStreamExt};
use serde::Serialize;
use std::path::Path;
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/files",
        post(upload_file).layer(DefaultBodyLimit::disable()),
    )
}

#[derive(Serialize)]
struct FileUpload {
    uuid: String,
}

async fn upload_file(
    WithRejection(mut body, _): WithRejection<Multipart, FileRouterError>,
) -> Result<(StatusCode, Json<Vec<FileUpload>>), FileRouterError> {
    let mut uuids = Vec::new();

    while let Some(field) = body.next_field().await? {
        let uuid = steam_to_file(field).await?;
        uuids.push(FileUpload {
            uuid: uuid.to_string(),
        });
    }

    Ok((StatusCode::CREATED, Json(uuids)))
}

async fn steam_to_file<E>(
    stream: impl Stream<Item = Result<Bytes, E>>,
) -> Result<Uuid, FileRouterError>
where
    FileRouterError: From<E>,
{
    let uuid = uuid::Uuid::new_v4();
    let path = Path::new("files").join(uuid.to_string());
    let mut file = BufWriter::new(File::create(path).await?);

    futures::pin_mut!(stream);
    while let Some(chunk) = stream.try_next().await? {
        file.write_all(&chunk).await?;
    }

    Ok(uuid)
}
