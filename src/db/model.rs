use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable)]
pub struct Upload {
    pub uuid: Uuid,
    pub file_name: Option<String>,
    pub uploaded_size: i64,
    pub uploaded_at: NaiveDateTime,
}
