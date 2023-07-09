use axum::{body::Bytes, http::StatusCode, Error};
use codegen::ErrorEnum;
use compute_file_hash::*;
use compute_file_mime::*;
use futures::{Stream, TryStreamExt};
use std::{
    io::SeekFrom,
    path::{Path, PathBuf},
};
use thiserror::Error;
use tokio::{
    fs::OpenOptions,
    io::{AsyncSeekExt, AsyncWriteExt, BufWriter},
};
use uuid::Uuid;

mod compute_file_hash;
mod compute_file_mime;

#[derive(Debug, Clone)]
pub struct FileDriver {
    pub files_path: PathBuf,
}

impl FileDriver {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            files_path: root.as_ref().join("files"),
        }
    }

    pub async fn create_dirs(&mut self) {
        let current_dir = std::env::current_dir().expect("failed to get current directory");

        self.files_path = {
            let mut path = current_dir;
            path.push(&self.files_path);
            path
        };

        tracing::info!(
            "creating files directory at `{}`",
            self.files_path.display()
        );
        tokio::fs::create_dir_all(&self.files_path)
            .await
            .expect(&format!(
                "failed to create files directory at `{}`",
                self.files_path.display()
            ));
    }

    pub async fn read_file_size(&self, uuid: Uuid) -> Result<Option<u64>, ReadFileSizeError> {
        let path = self.files_path.join(uuid.to_string());
        let metadata = tokio::fs::metadata(&path).await;
        match metadata {
            Ok(metadata) => Ok(Some(metadata.len())),
            Err(err) if err.kind() == tokio::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(ReadFileSizeError::ReadFileMetadata(err)),
        }
    }

    pub async fn write_file(
        &self,
        uuid: Uuid,
        offset: Option<u64>,
        stream: impl Stream<Item = Result<Bytes, Error>>,
    ) -> Result<u64, WriteFileError> {
        let path = self.files_path.join(uuid.to_string());
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await
            .map_err(WriteFileError::CreateFile)?;
        let offset = offset.unwrap_or_default();

        if offset != 0 {
            let metadata = file
                .metadata()
                .await
                .map_err(WriteFileError::ReadFileMetadata)?;
            let file_size = metadata.len();

            if file_size < offset {
                return Err(WriteFileError::InvalidOffset { offset, file_size });
            }
        }

        file.seek(SeekFrom::Start(offset))
            .await
            .map_err(WriteFileError::WriteToFile)?;
        file.set_len(offset)
            .await
            .map_err(WriteFileError::WriteToFile)?;

        let mut writer = BufWriter::new(&mut file);

        futures::pin_mut!(stream);
        while let Some(chunk) = stream.try_next().await? {
            writer
                .write_all(&chunk)
                .await
                .map_err(WriteFileError::WriteToFile)?;
        }

        let metadata = file
            .metadata()
            .await
            .map_err(WriteFileError::ReadFileMetadata)?;
        Ok(metadata.len())
    }

    pub async fn read_file_info(&self, uuid: Uuid) -> Result<FileInfo, ReadFileInfoError> {
        let path = self.files_path.join(uuid.to_string());
        let hash = compute_file_hash(&path);
        let mime = compute_file_mime(&path);
        let hash = hash.await?;
        let mime = mime.await?;
        Ok(FileInfo { hash, mime })
    }
}

#[derive(ErrorEnum, Error, Debug)]
pub enum ReadFileSizeError {
    #[error("internal server error")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    ReadFileMetadata(tokio::io::Error),
}

#[derive(ErrorEnum, Error, Debug)]
pub enum WriteFileError {
    #[error("internal server error")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    CreateFile(tokio::io::Error),
    #[error("internal server error")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    ReadFileMetadata(tokio::io::Error),
    #[error("invalid offset; offset is `{offset}`, but file size is `{file_size}`")]
    #[status(StatusCode::UNPROCESSABLE_ENTITY)]
    InvalidOffset { offset: u64, file_size: u64 },
    #[error("internal server error")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    ReadFromStream(#[from] Error),
    #[error("internal server error")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    WriteToFile(tokio::io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileInfo {
    pub mime: &'static str,
    pub hash: u32,
}

#[derive(ErrorEnum, Error, Debug)]
pub enum ReadFileInfoError {
    #[error("internal server error")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    ReadFileMetadata(std::io::Error),
    #[error("internal server error")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    ComputeFileHashError(#[from] ComputeFileHashError),
    #[error("internal server error")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    ComputeFileMimeError(#[from] ComputeFileMimeError),
}
