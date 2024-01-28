use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

fn default_page_size() -> u64 {
    25
}

#[derive(Deserialize, ToSchema, Debug, Clone, PartialEq, Eq, Hash)]
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
    #[into_params(example = "1")]
    pub first_id: Option<i32>,
    #[into_params(example = "1")]
    pub last_id: Option<i32>,
    #[into_params(example = "desc", default = "desc")]
    #[serde(default)]
    pub order: PaginationOrderDto,
    #[into_params(example = "25", default = "25", minimum = 1, maximum = 100)]
    #[serde(default = "default_page_size")]
    pub page_size: u64,
    #[into_params(example = "Movies")]
    pub filter_name: Option<String>,
}

#[derive(Deserialize, IntoParams, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Path)]
pub struct FindCollectionPathDto {
    #[into_params(example = "550e8400-e29b-41d4-a716-446655440000")]
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
    pub identifier: i32,
}

#[derive(Deserialize, IntoParams, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Path)]
pub struct UpdateCollectionPathDto {
    #[into_params(example = "1")]
    pub identifier: i32,
}

#[derive(Deserialize, ToSchema, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCollectionBodyDto {
    #[schema(example = "Movies")]
    pub name: String,
    #[schema(example = "Movies I like.")]
    pub description: Option<String>,
}
