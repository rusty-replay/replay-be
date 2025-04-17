use sea_orm::{Database, DatabaseConnection};
use anyhow::Result;

pub async fn init_db() -> Result<DatabaseConnection> {
    let url = std::env::var("DATABASE_URL")?;
    let conn = Database::connect(&url).await?;
    Ok(conn)
}