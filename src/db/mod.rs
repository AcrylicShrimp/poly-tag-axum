use crate::response::IntoStatus;
use axum::http::StatusCode;
use diesel::{Connection, PgConnection};
use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use thiserror::Error;

pub mod model;
pub mod schema;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/db/migrations");

pub type DBPool = Pool<AsyncPgConnection>;

#[derive(Error, Debug)]
pub enum DBError {
    #[error("internal server error")]
    PoolError(#[from] diesel_async::pooled_connection::deadpool::PoolError),
    #[error("internal server error")]
    DieselError(#[from] diesel::result::Error),
}

impl IntoStatus for DBError {
    fn into_status(&self) -> StatusCode {
        match self {
            DBError::PoolError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DBError::DieselError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub fn run_migrations() {
    tracing::info!("running database migrations");

    let database_url = std::env::var("DATABASE_URL").expect("env var `DATABASE_URL` must be set");
    let mut connection =
        PgConnection::establish(&database_url).expect("failed to establish database connection");
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("failed to run migrations");
}

pub fn init_pool() -> DBPool {
    tracing::info!("initializing database connection pool");

    let database_url = std::env::var("DATABASE_URL").expect("env var `DATABASE_URL` must be set");
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);

    let pool_max_size = num_cpus::get().max(32);
    tracing::debug!("max size of database connection pool is {}", pool_max_size);

    Pool::builder(manager)
        .max_size(pool_max_size)
        .build()
        .expect("failed to create database connection pool")
}
