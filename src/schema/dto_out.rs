use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct PaginationMetadataDto {
    pub has_prev: bool,
    pub has_next: bool,
}

#[derive(Serialize, ToSchema, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct CollectionDto {
    #[schema(example = "1")]
    pub id: i32,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub uuid: Uuid,
    #[schema(example = "Movies")]
    pub name: String,
    #[schema(example = "Movies I like.")]
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct FindCollectionsResultDto {
    pub pagination: PaginationMetadataDto,
    pub items: Vec<CollectionDto>,
}
