use crate::route_uploads::dto::GetUploadResponse;
use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::route_uploads::get_upload,
    ),
    components(
        schemas(ErrorBody),
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
