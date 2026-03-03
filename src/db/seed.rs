use sqlx::SqlitePool;
use crate::error::AppError;

pub async fn load_rules(_db: &SqlitePool, _path: &str) -> Result<(), AppError> {
    // Placeholder for rule seeding logic
    Ok(())
}
