use crate::app_state::AppState;
use axum::{
    routing::{get, post, put},
    Router,
};

pub mod handler_list;
pub mod handler_prepare;
pub mod handler_upload;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/files", get(handler_list::handle))
        .route("/files", post(handler_prepare::handle))
        .route("/files/:uuid", put(handler_upload::handle))
}
