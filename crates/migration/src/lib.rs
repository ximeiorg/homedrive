pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_member_table;
mod m20260204_013920_file_contents;
mod m20260204_013930_user_files;
mod m20260205_041830_create_sync_messages_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_member_table::Migration),
            Box::new(m20260204_013920_file_contents::Migration),
            Box::new(m20260204_013930_user_files::Migration),
            Box::new(m20260205_041830_create_sync_messages_table::Migration),
        ]
    }
}
