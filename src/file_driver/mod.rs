use axum::{body::Bytes, extract::multipart::MultipartError, http::StatusCode};
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
    pub stagings_path: PathBuf,
    pub files_path: PathBuf,
}

impl FileDriver {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            stagings_path: root.as_ref().join("stagings"),
            files_path: root.as_ref().join("files"),
        }
    }

    pub async fn create_dirs(&mut self) {
        let current_dir = std::env::current_dir().expect("failed to get current directory");
        self.stagings_path = {
            let mut path = current_dir.clone();
            path.push(&self.stagings_path);
            path
        };
        self.files_path = {
            let mut path = current_dir;
            path.push(&self.files_path);
            path
        };

        tracing::info!(
            "creating stagings directory at `{}`",
            self.stagings_path.display()
        );
        tokio::fs::create_dir_all(&self.stagings_path)
            .await
            .expect(&format!(
                "failed to create stagings directory at `{}`",
                self.stagings_path.display()
            ));

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

    pub async fn read_staging_size(&self, uuid: Uuid) -> Result<Option<u64>, ReadStagingSizeError> {
        let path = self.stagings_path.join(uuid.to_string());
        let metadata = tokio::fs::metadata(&path).await;
        match metadata {
            Ok(metadata) => Ok(Some(metadata.len())),
            Err(err) if err.kind() == tokio::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(ReadStagingSizeError::ReadFileMetadata(err)),
        }
    }

    pub async fn write_staging(
        &self,
        uuid: Uuid,
        offset: Option<u64>,
        stream: impl Stream<Item = Result<Bytes, MultipartError>>,
    ) -> Result<u64, WriteStagingError> {
        let path = self.stagings_path.join(uuid.to_string());
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await
            .map_err(WriteStagingError::CreateFile)?;
        let offset = offset.unwrap_or_default();

        if offset != 0 {
            let metadata = file
                .metadata()
                .await
                .map_err(WriteStagingError::ReadFileMetadata)?;
            let file_size = metadata.len();

            if file_size < offset {
                return Err(WriteStagingError::InvalidOffset { offset, file_size });
            }
        }

        file.seek(SeekFrom::Start(offset))
            .await
            .map_err(WriteStagingError::WriteToFile)?;
        file.set_len(offset)
            .await
            .map_err(WriteStagingError::WriteToFile)?;

        let mut writer = BufWriter::new(&mut file);

        futures::pin_mut!(stream);
        while let Some(chunk) = stream.try_next().await? {
            writer
                .write_all(&chunk)
                .await
                .map_err(WriteStagingError::WriteToFile)?;
        }

        let metadata = file
            .metadata()
            .await
            .map_err(WriteStagingError::ReadFileMetadata)?;
        Ok(metadata.len())
    }

    pub async fn read_staging_info(&self, uuid: Uuid) -> Result<StagingInfo, ReadStagingInfoError> {
        let path = self.stagings_path.join(uuid.to_string());
        let hash = compute_file_hash(&path);
        let mime = compute_file_mime(&path);
        let hash = hash.await?;
        let mime = mime.await?;
        Ok(StagingInfo { hash, mime })
    }

    pub async fn commit_staging_into_file(
        &self,
        staging_uuid: Uuid,
        file_uuid: Uuid,
    ) -> Result<(), CommitStagingIntoFileError> {
        // TODO: Rename won't work between different filesystems. Document this.
        Ok(tokio::fs::rename(
            self.stagings_path.join(staging_uuid.to_string()),
            self.files_path.join(file_uuid.to_string()),
        )
        .await?)
    }

    // pub async fn commit_staging_into_file(
    //     &self,
    //     staging_uuid: Uuid,
    //     staging_name: String,
    //     file_uuid: Uuid,
    // ) -> Result<FileInfo, CommitStagingIntoFileError> {
    //     let staging_path = self.stagings_path.join(staging_uuid.to_string());
    //     let staging_metadata = tokio::fs::metadata(&staging_path)
    //         .await
    //         .map_err(CommitStagingIntoFileError::ReadStagingFileMetadata)?;
    //     let staging_size = staging_metadata.len();

    //     let staging_file = tokio::fs::File::open(&staging_path)
    //         .await
    //         .map_err(CommitStagingIntoFileError::ReadStagingFile)?;

    //     let file_path = self.files_path.join(file_uuid.to_string());
    //     let file = tokio::fs::File::create(&file_path)
    //         .await
    //         .map_err(CommitStagingIntoFileError::CreateFile)?;

    //     let mut reader = tokio::io::BufReader::new(staging_file);
    //     let mut writer = tokio::io::BufWriter::new(file);

    //     tokio::io::copy(&mut reader, &mut writer)
    //         .await
    //         .map_err(CommitStagingIntoFileError::WriteToFile)?;

    //     let hash = {
    //         let mut hasher = md5::Md5::new();
    //         let mut reader = tokio::io::BufReader::new(staging_file);
    //         hasher.update(&mut reader);
    //         format!("{:x}", hasher.finalize())
    //     };

    //     Ok(FileInfo {
    //         uuid: file_uuid,
    //         name: staging_name,
    //         mime_type: "application/octet-stream".to_string(),
    //         hash,
    //         size: staging_size,
    //     })
    // }
}

#[derive(ErrorEnum, Error, Debug)]
pub enum ReadStagingSizeError {
    #[error("internal server error")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    ReadFileMetadata(tokio::io::Error),
}

#[derive(ErrorEnum, Error, Debug)]
pub enum WriteStagingError {
    #[error("internal server error")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    CreateFile(tokio::io::Error),
    #[error("internal server error")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    ReadFileMetadata(tokio::io::Error),
    #[error("invalid offset; offset is `{offset}`, but file size is `{file_size}`")]
    #[status(StatusCode::BAD_REQUEST)]
    InvalidOffset { offset: u64, file_size: u64 },
    #[error("{0}")]
    #[status("0")]
    ReadFromStream(#[from] MultipartError),
    #[error("internal server error")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    WriteToFile(tokio::io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StagingInfo {
    pub mime: &'static str,
    pub hash: u32,
}

#[derive(ErrorEnum, Error, Debug)]
pub enum ReadStagingInfoError {
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

#[derive(ErrorEnum, Error, Debug)]
pub enum CommitStagingIntoFileError {
    #[error("internal server error")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    IOError(#[from] tokio::io::Error),
}
