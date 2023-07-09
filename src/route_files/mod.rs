use crate::app_state::AppState;
use axum::{routing::post, Router};

pub mod handler_prepare;

pub fn router() -> Router<AppState> {
    Router::new().route("/files", post(handler_prepare::handle))
}
