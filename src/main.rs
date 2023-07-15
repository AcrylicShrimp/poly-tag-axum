mod app_state;
mod db;
mod docs;
mod file_driver;
mod response;
mod route_files;
mod route_tag_templates;

use crate::{docs::ApiDoc, file_driver::FileDriver};
use app_state::AppState;
use axum::{http::StatusCode, response::IntoResponse, Router, Server};
use std::net::SocketAddr;
use tokio::signal;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// TODO: Don't panic on errors, return a Result instead.

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!(
        "launching {} {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    db::run_migrations();
    let db_pool = db::init_pool();

    let mut file_driver = FileDriver::new("./files"); // TODO: make this configurable
    file_driver.create_dirs().await;

    let meilisearch_client = db::init_meilisearch_client().await;

    let app_state = AppState::new(db_pool, file_driver, meilisearch_client);
    let app = Router::new();

    let port = 3000; // TODO: make this configurable
    let addr = SocketAddr::from(([127, 0, 0, 1], port)); // TODO: make this configurable
    #[cfg(debug_assertions)]
    let app = {
        tracing::info!(
            "enabling swagger ui, will be accessible at: http://localhost:{}/swagger-ui",
            port
        );
        app.merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
    };

    let app = app
        .merge(route_files::router())
        // .merge(route_stagings::router())
        .merge(route_tag_templates::router())
        .fallback(handler_fallback)
        .with_state(app_state);

    tracing::info!("listening on {}", addr);
    Server::bind(&addr)
        .http2_enable_connect_protocol()
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!();
}

async fn handler_fallback() -> impl IntoResponse {
    StatusCode::NOT_FOUND
}
