use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Queryable, Serialize, Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Staging {
    pub uuid: Uuid,
    pub staged_at: NaiveDateTime,
}

#[derive(DbEnum, Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl Display for TagValueTypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TagValueTypeKind::String => write!(f, "string"),
            TagValueTypeKind::Integer => write!(f, "integer"),
            TagValueTypeKind::Boolean => write!(f, "boolean"),
        }
    }
}

#[derive(Queryable, Serialize, Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TagTemplate {
    pub uuid: Uuid,
    #[schema(example = "Author")]
    pub name: String,
    #[schema(example = "Author of the file.")]
    pub description: Option<String>,
    pub value_type: Option<TagValueTypeKind>,
    pub created_at: NaiveDateTime,
}

#[derive(Queryable, Serialize, Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TagTemplateCompact {
    pub uuid: Uuid,
    pub value_type: Option<TagValueTypeKind>,
}

#[derive(Queryable, Serialize, Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub uuid: Uuid,
    #[schema(example = "file.txt")]
    pub name: String,
    #[schema(example = "text/plain")]
    pub mime: String,
    #[schema(example = "1024")]
    pub size: i64,
    #[schema(example = "1234567890")]
    pub hash: i64,
    pub uploaded_at: NaiveDateTime,
}

#[derive(Queryable, Serialize, Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FileHeader {
    pub uuid: Uuid,
    pub name: String,
}
