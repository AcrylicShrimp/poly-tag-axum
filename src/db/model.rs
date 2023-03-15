use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable)]
pub struct Upload {
    pub uuid: Uuid,
    pub created_at: NaiveDateTime,
}
