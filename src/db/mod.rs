pub mod model;
pub mod schema;

use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/db/migrations");

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn init_pool() -> DbPool {
    tracing::info!("initializing database connection pool");

    let database_url = std::env::var("DATABASE_URL").expect("env var `DATABASE_URL` must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    let pool_max_size = num_cpus::get().max(32) as u32;
    tracing::debug!("max size of database connection pool is {}", pool_max_size);

    Pool::builder()
        .max_size(pool_max_size)
        .build(manager)
        .expect("failed to create database connection pool")
}

pub fn run_migrations(pool: &DbPool) {
    tracing::info!("running database migrations");

    let mut connection = pool.get().expect("failed to get connection from pool");
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("failed to run migrations");
}
