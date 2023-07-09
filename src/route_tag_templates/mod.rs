use crate::app_state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub mod handler_list;
pub mod handler_post;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tag-templates", get(handler_list::handle))
        .route("/tag-templates", post(handler_post::handle))
}
