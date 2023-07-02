use crate::response::IntoStatus;
use axum::{
    extract::{
        multipart::{Field, MultipartError},
        Multipart,
    },
    http::StatusCode,
};
use thiserror::Error;

pub struct MultipartParser<'c, R, E1, E2> {
    max_field_count: Option<usize>,
    filter: Option<&'c (dyn Fn(usize, &Field) -> Result<bool, E1> + Send + Sync)>,
    writer: &'c (dyn Fn(usize, Field) -> Result<R, E2> + Send + Sync),
}

impl<'c, R, E1, E2> MultipartParser<'c, R, E1, E2> {
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

#[derive(Error, Debug)]
pub enum MultipartParserError<E1, E2> {
    #[error("internal server error")]
    MultipartError(#[from] MultipartError),
    #[error("internal server error")]
    FilterError(E1),
    #[error("internal server error")]
    WriterError(E2),
    #[error("fields cannot be more than {0}")]
    MaxFieldCountError(usize),
}

impl<E1, E2> IntoStatus for MultipartParserError<E1, E2>
where
    E1: IntoStatus,
    E2: IntoStatus,
{
    fn into_status(&self) -> StatusCode {
        match self {
            MultipartParserError::MultipartError(err) => err.status(),
            MultipartParserError::FilterError(err) => err.into_status(),
            MultipartParserError::WriterError(err) => err.into_status(),
            MultipartParserError::MaxFieldCountError(_) => StatusCode::BAD_REQUEST,
        }
    }
}
