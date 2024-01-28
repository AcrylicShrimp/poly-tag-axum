use serde::Deserialize;
use std::num::NonZeroU32;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

fn default_page_size() -> NonZeroU32 {
    NonZeroU32::new(25).unwrap()
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum PaginationOrderDto {
    Asc,
    Desc,
}

impl Default for PaginationOrderDto {
    fn default() -> Self {
        Self::Desc
    }
}

#[derive(Deserialize, IntoParams, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct FindCollectionsQueryDto {
    #[into_params(example = "1", default = "1")]
    pub first_id: Option<NonZeroU32>,
    #[into_params(example = "1", default = "1")]
    pub last_id: Option<NonZeroU32>,
    #[into_params(example = "desc", default = "desc")]
    #[serde(default)]
    pub order: PaginationOrderDto,
    #[into_params(example = "25", default = "25", maximum = 100)]
    #[serde(default = "default_page_size")]
    pub page_size: NonZeroU32,
    #[into_params(example = "Movies")]
    pub filter_name: Option<String>,
}

#[derive(Deserialize, IntoParams, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Path)]
pub struct FindCollectionPathDto {
    #[into_params(example = "550e8400-e29b-41d4-a716-446655440000")]
    #[into_params(description = "uuid of the collection")]
    pub identifier: Uuid,
}

#[derive(Deserialize, ToSchema, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct CreateCollectionBodyDto {
    #[schema(example = "Movies")]
    pub name: String,
    #[schema(example = "Movies I like.")]
    pub description: Option<String>,
}

#[derive(Deserialize, IntoParams, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Path)]
pub struct RemoveCollectionPathDto {
    #[into_params(example = "1")]
    #[into_params(description = "id of the collection")]
    pub identifier: NonZeroU32,
}

#[derive(Deserialize, IntoParams, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Path)]
pub struct UpdateCollectionPathDto {
    #[into_params(example = "1")]
    #[into_params(description = "id of the collection")]
    pub identifier: NonZeroU32,
}

#[derive(Deserialize, ToSchema, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCollectionBodyDto {
    #[schema(example = "Movies")]
    pub name: String,
    #[schema(example = "Movies I like.")]
    pub description: Option<String>,
}

#[derive(Deserialize, ToSchema, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct FindFilesBodyDto {
    #[schema(example = "1", default = "1")]
    pub page: NonZeroU32,
    #[serde(default = "default_page_size")]
    #[schema(example = "25", default = "25", maximum = 100)]
    pub page_size: NonZeroU32,
    #[schema(example = "john wick")]
    pub query: Option<String>,
    pub filter: Option<FindFilesBodyFilterDto>,
}

#[derive(Deserialize, ToSchema, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct FindFilesBodyFilterDto {
    #[schema(
        example = json!([
            "550e8400-e29b-41d4-a716-446655440000",
            "ef14ca60-eace-40f0-9cd5-5ee8798ba954"
        ])
    )]
    pub uuids: Option<Vec<Uuid>>,
    #[schema(example = "file.txt")]
    pub name: Option<String>,
    #[schema(example = "text/plain")]
    pub mime: Option<String>,
    pub size: Option<FindFilesBodyFilterSizeDto>,
    #[schema(example = "1234567890")]
    pub hash: Option<i64>,
}

#[derive(Deserialize, ToSchema, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct FindFilesBodyFilterSizeDto {
    #[schema(example = "1024")]
    pub min: Option<u32>,
    #[schema(example = "2048")]
    pub max: Option<u32>,
}

#[derive(Deserialize, ToSchema, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct PrepareUploadBodyDto {
    #[schema(example = "file.txt")]
    pub name: String,
    #[serde(default)]
    pub tags: Vec<PrepareUploadBodyTagDto>,
}

#[derive(Deserialize, ToSchema, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct PrepareUploadBodyTagDto {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub uuid: Uuid,
    pub value: PrepareUploadBodyTagValueDto,
}

#[derive(Deserialize, ToSchema, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum PrepareUploadBodyTagValueDto {
    #[schema(example = "foo")]
    String(String),
    #[schema(example = "123")]
    Integer(i64),
    #[schema(example = "true")]
    Boolean(bool),
}
