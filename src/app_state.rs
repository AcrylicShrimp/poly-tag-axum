use crate::{db::DbPool, file_driver::FileDriver};
use axum::extract::FromRef;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub file_driver: FileDriver,
}

impl AppState {
    pub fn new(db_pool: DbPool, file_driver: FileDriver) -> Self {
        Self {
            db_pool,
            file_driver,
        }
    }
}

impl FromRef<AppState> for DbPool {
    fn from_ref(input: &AppState) -> Self {
        input.db_pool.clone()
    }
}

impl FromRef<AppState> for FileDriver {
    fn from_ref(input: &AppState) -> Self {
        input.file_driver.clone()
    }
}
