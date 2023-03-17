use axum::body::Bytes;
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
        self.stagings_path = tokio::fs::canonicalize(&self.stagings_path)
            .await
            .expect(&format!(
                "failed to canonicalize path for stagings: `{}`",
                self.stagings_path.display()
            ));
        self.files_path = tokio::fs::canonicalize(&self.files_path)
            .await
            .expect(&format!(
                "failed to canonicalize path for files: `{}`",
                self.files_path.display()
            ));

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

    pub async fn write_staging<E>(
        &self,
        uuid: Uuid,
        offset: Option<u64>,
        stream: impl Stream<Item = Result<Bytes, E>>,
    ) -> Result<u64, WriteStagingError<E>> {
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
}

#[derive(Debug, Error)]
pub enum ReadStagingSizeError {
    #[error("failed to read file metadata")]
    ReadFileMetadata(tokio::io::Error),
}

#[derive(Debug, Error)]
pub enum WriteStagingError<E> {
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
