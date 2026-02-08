use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 添加 thumbnail 列
        manager
            .alter_table(
                Table::alter()
                    .table(FileContents::Table)
                    .add_column(ColumnDef::new(FileContents::Thumbnail).string_len(500))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(FileContents::Table)
                    .drop_column(FileContents::Thumbnail)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum FileContents {
    Table,
    Thumbnail,
}
