use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DbErr};
pub use sea_orm::DatabaseConnection;

pub async fn connect_db<S: ToString>(dsn: S,debug_model:bool) -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(dsn.to_string());
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(debug_model); // Setting default PostgreSQL schema

    let db = Database::connect(opt).await?;
    Ok(db)
}