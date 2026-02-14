pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_member_table;
mod m20260204_013920_file_contents;
mod m20260204_013930_user_files;
mod m20260205_041830_create_task_messages_table;
mod m20260208_090000_add_thumbnail;
mod m20260211_063853_album;
mod m20260211_064048_album_files;
mod m20260214_000001_add_member_role;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_member_table::Migration),
            Box::new(m20260204_013920_file_contents::Migration),
            Box::new(m20260204_013930_user_files::Migration),
            Box::new(m20260205_041830_create_task_messages_table::Migration),
            Box::new(m20260208_090000_add_thumbnail::Migration),
            Box::new(m20260211_063853_album::Migration),
            Box::new(m20260211_064048_album_files::Migration),
            Box::new(m20260214_000001_add_member_role::Migration),
        ]
    }
}
