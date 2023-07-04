use crate::response::IntoStatus;
use axum::{
    extract::{
        multipart::{Field, MultipartError},
        Multipart,
    },
    http::StatusCode,
};
use codegen::ErrorEnum;
use thiserror::Error;

pub struct MultipartParser<'c, R, E1, E2> {
    max_field_count: Option<usize>,
    filter: Option<&'c (dyn Fn(usize, &Field) -> Result<bool, E1> + Send + Sync)>,
    writer: &'c (dyn Fn(usize, Field) -> Result<R, E2> + Send + Sync),
}

impl<'c, R, E1, E2> MultipartParser<'c, R, E1, E2>
where
    E1: std::fmt::Debug + IntoStatus,
    E2: std::fmt::Debug + IntoStatus,
{
    pub fn new(
        max_field_count: Option<usize>,
        filter: Option<&'c (dyn Fn(usize, &Field) -> Result<bool, E1> + Send + Sync)>,
        writer: &'c (dyn Fn(usize, Field) -> Result<R, E2> + Send + Sync),
    ) -> Self {
        Self {
            max_field_count,
            filter,
            writer,
        }
    }

    pub async fn parse(&self, mut body: Multipart) -> Result<Vec<R>, MultipartParserError<E1, E2>> {
        let mut fields = Vec::new();
        let mut field_count = 0;

        while let Some(field) = body.next_field().await? {
            field_count += 1;

            if let Some(max_field_count) = self.max_field_count {
                if max_field_count < field_count {
                    return Err(MultipartParserError::MaxFieldCountError(max_field_count));
                }
            }

            if let Some(filter) = &self.filter {
                if !filter(field_count, &field).map_err(MultipartParserError::FilterError)? {
                    continue;
                }
            }

            let field =
                (self.writer)(field_count, field).map_err(MultipartParserError::WriterError)?;

            fields.push(field);
        }

        Ok(fields)
    }
}

#[derive(ErrorEnum, Error, Debug)]
pub enum MultipartParserError<E1, E2>
where
    E1: std::fmt::Debug + IntoStatus,
    E2: std::fmt::Debug + IntoStatus,
{
    #[error("internal server error")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    MultipartError(#[from] MultipartError),
    #[error("{0}")]
    #[status("0")]
    FilterError(E1),
    #[error("{0}")]
    #[status("0")]
    WriterError(E2),
    #[error("fields cannot be more than {0}")]
    #[status(StatusCode::BAD_REQUEST)]
    MaxFieldCountError(usize),
}
