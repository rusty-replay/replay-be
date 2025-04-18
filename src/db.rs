use sea_orm::{Database, DatabaseConnection};
use std::env;
use tracing::{info, instrument, debug};
use sea_orm::ConnectOptions;
use std::time::Duration;

#[instrument]
pub async fn init_db() -> anyhow::Result<DatabaseConnection> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    info!("데이터베이스 연결 설정 중...");

    let mut options = ConnectOptions::new(database_url);
    options
        .max_connections(10)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true);

    info!("데이터베이스에 연결 시도 중...");
    let db = Database::connect(options).await?;
    info!("데이터베이스 연결 완료");

    Ok(db)
}