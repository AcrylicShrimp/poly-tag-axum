use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::route_collections::handlers::find_collections,
        crate::route_collections::handlers::find_collection,
        crate::route_collections::handlers::create_collection,
        crate::route_collections::handlers::update_collection,
        crate::route_collections::handlers::remove_collection,
    ),
    components(
        schemas(ErrorBody),

        schemas(crate::schema::dto_in::PaginationOrderDto),
        schemas(crate::schema::dto_in::CreateCollectionBodyDto),
        schemas(crate::schema::dto_in::UpdateCollectionBodyDto),
        
        schemas(crate::schema::dto_out::PaginationMetadataDto),
        schemas(crate::schema::dto_out::CollectionDto),
        schemas(crate::schema::dto_out::FindCollectionsResultDto),
    ),
    tags(
        (name = "tag-template", description = "Tag template API for file tagging."),
        (name = "file", description = "File API for file management."),
        (name = "collection", description = "Collection API for file collection management."),
    ),
)]
pub struct ApiDoc;

#[derive(ToSchema)]
pub struct ErrorBody {
    #[schema(example = "internal server error")]
    pub error: String,
}
