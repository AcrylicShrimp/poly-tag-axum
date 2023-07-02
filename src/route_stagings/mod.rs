use crate::app_state::AppState;
use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post, put},
    Router,
};

pub mod handler_get;
pub mod handler_post;
pub mod handler_put;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/stagings", post(handler_post::handle))
        .route("/stagings/:uuid", get(handler_get::handle))
        .route(
            "/stagings/:uuid",
            put(handler_put::handle).layer(DefaultBodyLimit::disable()),
        )
}
