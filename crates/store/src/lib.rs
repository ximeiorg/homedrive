use std::time::Duration;

use migration::{Migrator, MigratorTrait};
pub use sea_orm::DatabaseConnection;
pub use sea_orm::{ConnectOptions, Database, DbErr};
pub mod album;
pub mod entity;
pub mod file_content;
pub mod member;
pub mod member_file;

pub async fn connect_db<S: ToString>(
    dsn: S,
    debug_model: bool,
) -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(dsn.to_string());
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(debug_model); // Setting default PostgreSQL schema

    let db = Database::connect(opt).await?;

    Migrator::up(&db, None).await?;
    Ok(db)
}
