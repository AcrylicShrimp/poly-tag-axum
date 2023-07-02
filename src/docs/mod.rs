use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::route_stagings::handler_post::handle,
        crate::route_stagings::handler_get::handle,
        crate::route_stagings::handler_put::handle,
    ),
    components(
        schemas(ErrorBody),
        schemas(crate::route_stagings::handler_post::dto::StagingPostRes),
        schemas(crate::route_stagings::handler_get::dto::StagingGetRes),
        schemas(crate::route_stagings::handler_put::dto::StagingPutRes),
    ),
    tags(
        (name = "staging", description = "File staging API for upload."),
    ),
)]
pub struct ApiDoc;

#[derive(ToSchema)]
pub struct ErrorBody {
    pub error: String,
}
