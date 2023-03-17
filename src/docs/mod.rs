use crate::route_stagings::dto::*;
use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::route_stagings::new_staging,
        crate::route_stagings::get_staging,
    ),
    components(
        schemas(ErrorBody),
        schemas(NewStagingResponse),
        schemas(GetStagingResponse),
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
