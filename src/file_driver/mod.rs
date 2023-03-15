use axum::body::Bytes;
use futures::{Stream, TryStreamExt};
use std::{io::SeekFrom, path::PathBuf};
use thiserror::Error;
use tokio::{
    fs::OpenOptions,
    io::{AsyncSeekExt, AsyncWriteExt, BufWriter},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FileDriver {
    pub root: PathBuf,
}

impl FileDriver {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub async fn create_root_dir(&mut self) {
        self.root = tokio::fs::canonicalize(&self.root).await.expect(&format!(
            "failed to canonicalize path `{}`",
            self.root.display()
        ));

        tracing::info!("creating root directory at `{}`", self.root.display());

        tokio::fs::create_dir_all(&self.root).await.expect(&format!(
            "failed to create root directory at `{}`",
            self.root.display()
        ));
    }

    pub async fn read_file_size(&self, uuid: Uuid) -> Result<Option<u64>, ReadFileSizeError> {
        let path = self.root.join(uuid.to_string());
        let metadata = tokio::fs::metadata(&path).await;
        match metadata {
            Ok(metadata) => Ok(Some(metadata.len())),
            Err(err) if err.kind() == tokio::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(ReadFileSizeError::ReadFileMetadata(err)),
        }
    }

    pub async fn write_file<E>(
        &self,
        uuid: Uuid,
        offset: Option<u64>,
        stream: impl Stream<Item = Result<Bytes, E>>,
    ) -> Result<u64, WriteFileError<E>> {
        let path = self.root.join(uuid.to_string());
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
}

#[derive(Debug, Error)]
pub enum ReadFileSizeError {
    #[error("failed to read file metadata")]
    ReadFileMetadata(tokio::io::Error),
}

#[derive(Debug, Error)]
pub enum WriteFileError<E> {
    #[error("failed to create file")]
    CreateFile(tokio::io::Error),
    #[error("failed to read file metadata")]
    ReadFileMetadata(tokio::io::Error),
    #[error("invalid offset; offset is `{offset}`, but file size is `{file_size}`")]
    InvalidOffset { offset: u64, file_size: u64 },
    #[error("failed to read from stream")]
    ReadFromStream(#[from] E),
    #[error("failed to write to file")]
    WriteToFile(tokio::io::Error),
}
