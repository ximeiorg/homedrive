use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 添加 deleted_at 字段用于软删除
        manager
            .alter_table(
                Table::alter()
                    .table(MemberFiles::Table)
                    .add_column(timestamp_null(MemberFiles::DeletedAt))
                    .to_owned(),
            )
            .await?;

        // 创建索引以加速回收站查询
        manager
            .create_index(
                Index::create()
                    .name("idx_member_files_deleted_at")
                    .table(MemberFiles::Table)
                    .col(MemberFiles::MemberId)
                    .col(MemberFiles::DeletedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 删除索引
        manager
            .drop_index(
                Index::drop()
                    .name("idx_member_files_deleted_at")
                    .table(MemberFiles::Table)
                    .to_owned(),
            )
            .await?;

        // 删除字段
        manager
            .alter_table(
                Table::alter()
                    .table(MemberFiles::Table)
                    .drop_column(MemberFiles::DeletedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum MemberFiles {
    #[iden = "member_files"]
    Table,
    Id,
    MemberId,
    DeletedAt,
}