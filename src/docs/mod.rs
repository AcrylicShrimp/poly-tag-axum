use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::route_collections::handlers::find_collections,
        crate::route_collections::handlers::find_collection,
        crate::route_collections::handlers::create_collection,
        crate::route_collections::handlers::update_collection,
        crate::route_collections::handlers::remove_collection,

        crate::route_tag_templates::handler_list::handle,
        crate::route_tag_templates::handler_post::handle,
        
        crate::route_files::handler_list::handle,
        crate::route_files::handler_prepare::handle,
        crate::route_files::handler_upload::handle,
    ),
    components(
        schemas(ErrorBody),

        schemas(crate::db::model::TagTemplate),
        schemas(crate::db::model::TagValueTypeKind),

        schemas(crate::schema::dto_in::CreateCollectionBodyDto),
        schemas(crate::schema::dto_in::UpdateCollectionBodyDto),
        
        schemas(crate::schema::dto_out::PaginationMetadataDto),
        schemas(crate::schema::dto_out::CollectionDto),
        schemas(crate::schema::dto_out::FindCollectionsResultDto),
        
        schemas(crate::route_tag_templates::handler_list::dto::TagTemplateListRes),
        
        schemas(crate::route_tag_templates::handler_post::dto::TagTemplatePostReqBody),
        schemas(crate::route_tag_templates::handler_post::dto::TagTemplatePostRes),

        schemas(crate::route_files::handler_list::dto::FileListReqBody),
        schemas(crate::route_files::handler_list::dto::FileListReqBodyTag),
        schemas(crate::route_files::handler_list::dto::FileListReqBodyTagValue),
        schemas(crate::route_files::handler_list::dto::FileListReqBodyTagValueParam),
        schemas(crate::route_files::handler_list::dto::FileListRes),
        schemas(crate::route_files::handler_list::dto::FileListResFile),

        schemas(crate::route_files::handler_prepare::dto::FilePrepareReqBody),
        schemas(crate::route_files::handler_prepare::dto::FilePrepareReqBodyTag),
        schemas(crate::route_files::handler_prepare::dto::FilePrepareReqBodyTagValue),
        schemas(crate::route_files::handler_prepare::dto::FilePrepareRes),

        schemas(crate::route_files::handler_upload::dto::FileUploadRes),
    ),
    tags(
        (name = "tag-template", description = "Tag template API for file tagging."),
        (name = "file", description = "File API for file management."),
    ),
)]
pub struct ApiDoc;

#[derive(ToSchema)]
pub struct ErrorBody {
    #[schema(example = "internal server error")]
    pub error: String,
}
