use std::{
    path::Path,
    pin::Pin,
    task::{Context, Poll},
};
use thiserror::Error;
use tokio::io::{AsyncWrite, Error as IOError};

#[derive(Debug, Error)]
pub enum ComputeFileHashError {
    #[error("failed to open file: {0}")]
    OpenFileError(IOError),
    #[error("failed to read file: {0}")]
    ReadFileError(IOError),
}

pub async fn compute_file_hash(path: impl AsRef<Path>) -> Result<u32, ComputeFileHashError> {
    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(ComputeFileHashError::OpenFileError)?;
    let mut hasher = AsyncCrc32Hasher::new();
    tokio::io::copy(&mut file, &mut hasher)
        .await
        .map_err(ComputeFileHashError::ReadFileError)?;
    Ok(hasher.into_inner().finalize())
}

struct AsyncCrc32Hasher {
    inner: crc32fast::Hasher,
}

impl AsyncCrc32Hasher {
    pub fn new() -> Self {
        Self {
            inner: crc32fast::Hasher::new(),
        }
    }

    pub fn into_inner(self) -> crc32fast::Hasher {
        self.inner
    }
}

impl AsyncWrite for AsyncCrc32Hasher {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, IOError>> {
        self.inner.update(buf);
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), IOError>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), IOError>> {
        Poll::Ready(Ok(()))
    }
}
