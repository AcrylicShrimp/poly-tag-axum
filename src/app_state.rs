use crate::db::DbPool;
use axum::extract::FromRef;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
}

impl AppState {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }
}

impl FromRef<AppState> for DbPool {
    fn from_ref(input: &AppState) -> Self {
        input.db_pool.clone()
    }
}
