use crate::route_uploads::dto::*;
use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::route_uploads::new_upload,
        crate::route_uploads::get_upload,
    ),
    components(
        schemas(ErrorBody),
        schemas(NewUploadResponse),
        schemas(GetUploadResponse),
    ),
    tags(
        (name = "uploading", description = "File uploading API"),
    ),
)]
pub struct ApiDoc;

#[derive(ToSchema)]
pub struct ErrorBody {
    pub error: String,
}
