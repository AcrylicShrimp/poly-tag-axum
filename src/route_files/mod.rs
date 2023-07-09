use crate::app_state::AppState;
use axum::{
    routing::{post, put},
    Router,
};

pub mod handler_prepare;
pub mod handler_upload;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/files", post(handler_prepare::handle))
        .route("/files/:uuid", put(handler_upload::handle))
}
