use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing::info;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(database_url)
        .await?;
    info!("connected to PostgreSQL");
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    info!("running database migrations");
    sqlx::migrate!().run(pool).await?;
    info!("migrations completed");
    Ok(())
}
