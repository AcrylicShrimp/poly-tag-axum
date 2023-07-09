use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::route_stagings::handler_post::handle,
        crate::route_stagings::handler_get::handle,
        crate::route_stagings::handler_put::handle,

        crate::route_tag_templates::handler_list::handle,
        crate::route_tag_templates::handler_post::handle,
    ),
    components(
        schemas(ErrorBody),

        schemas(crate::db::model::TagTemplate),
        schemas(crate::db::model::TagValueTypeKind),
        
        schemas(crate::route_stagings::handler_post::dto::StagingPostRes),
        schemas(crate::route_stagings::handler_get::dto::StagingGetRes),
        schemas(crate::route_stagings::handler_put::dto::StagingPutRes),

        schemas(crate::route_tag_templates::handler_list::dto::TagTemplateListReqQuery),
        schemas(crate::route_tag_templates::handler_list::dto::TagTemplateListRes),

        schemas(crate::route_tag_templates::handler_post::dto::TagTemplatePostReqBody),
        schemas(crate::route_tag_templates::handler_post::dto::TagTemplatePostRes),
    ),
    tags(
        (name = "staging", description = "File staging API for upload."),
        (name = "tag-template", description = "Tag template API for file tagging."),
    ),
)]
pub struct ApiDoc;

#[derive(ToSchema)]
pub struct ErrorBody {
    #[schema(example = "internal server error")]
    pub error: String,
}
