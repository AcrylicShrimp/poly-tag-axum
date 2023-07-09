use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Queryable, Serialize, Deserialize, ToSchema, Debug)]
pub struct Staging {
    pub uuid: Uuid,
    pub staged_at: NaiveDateTime,
}

#[derive(Queryable, Serialize, Deserialize, ToSchema, Debug)]
pub struct TagTemplate {
    pub uuid: Uuid,
    #[schema(example = "Author")]
    pub name: String,
    #[schema(example = "Author of the file.")]
    pub description: Option<String>,
    pub value_type: Option<TagValueTypeKind>,
    pub created_at: NaiveDateTime,
}

#[derive(DbEnum, Serialize, Deserialize, ToSchema, Debug)]
#[ExistingTypePath = "crate::db::schema::sql_types::TagValueType"]
pub enum TagValueTypeKind {
    #[serde(alias = "string")]
    String,
    #[serde(alias = "int")]
    #[serde(alias = "integer")]
    Integer,
    #[serde(alias = "bool")]
    #[serde(alias = "boolean")]
    Boolean,
}
