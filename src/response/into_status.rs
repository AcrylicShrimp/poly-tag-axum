use axum::http::StatusCode;

pub trait IntoStatus {
    fn into_status(&self) -> StatusCode;
}

impl IntoStatus for () {
    fn into_status(&self) -> StatusCode {
        StatusCode::OK
    }
}

impl IntoStatus for StatusCode {
    fn into_status(&self) -> StatusCode {
        *self
    }
}
