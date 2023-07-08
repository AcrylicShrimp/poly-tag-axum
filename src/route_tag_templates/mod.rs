use crate::app_state::AppState;
use axum::{routing::post, Router};

pub mod handler_post;

pub fn router() -> Router<AppState> {
    Router::new().route("/tag-templates", post(handler_post::handle))
}
