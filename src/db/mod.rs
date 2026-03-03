use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use crate::error::AppError;

pub mod seed;

pub async fn init_pool(database_url: &str) -> Result<SqlitePool, AppError> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Run migrations (simple schema exec for now)
    let schema = include_str!("schema.sql");
    sqlx::query(schema).execute(&pool).await?;

    Ok(pool)
}
