use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable)]
pub struct Staging {
    pub uuid: Uuid,
    pub staged_at: NaiveDateTime,
}
