use crate::{db::DBPool, file_driver::FileDriver};
use axum::extract::FromRef;
use meilisearch_sdk::Client;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: DBPool,
    pub file_driver: FileDriver,
    pub meilisearch_client: Arc<Client>,
}

impl AppState {
    pub fn new(db_pool: DBPool, file_driver: FileDriver, meilisearch_client: Client) -> Self {
        Self {
            db_pool,
            file_driver,
            meilisearch_client: Arc::new(meilisearch_client),
        }
    }
}

impl FromRef<AppState> for DBPool {
    fn from_ref(input: &AppState) -> Self {
        input.db_pool.clone()
    }
}

impl FromRef<AppState> for FileDriver {
    fn from_ref(input: &AppState) -> Self {
        input.file_driver.clone()
    }
}

impl FromRef<AppState> for Arc<Client> {
    fn from_ref(input: &AppState) -> Self {
        input.meilisearch_client.clone()
    }
}
