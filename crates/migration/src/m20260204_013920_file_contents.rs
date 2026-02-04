use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FileContents::Table)
                    .if_not_exists()
                    .col(pk_auto(FileContents::Id))
                    .col(
                        string_len(FileContents::ContentHash, 64)
                            .unique_key()
                            .not_null(),
                    )
                    .col(big_integer(FileContents::FileSize).not_null())
                    .col(string_len(FileContents::StoragePath, 500).not_null())
                    .col(string_len(FileContents::MimeType, 100))
                    .col(integer(FileContents::Width))
                    .col(integer(FileContents::Height))
                    .col(integer(FileContents::Duration))
                    .col(integer(FileContents::RefCount).default(1))
                    .col(
                        timestamp(FileContents::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(integer(FileContents::FirstUploadedBy))
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(FileContents::FirstUploadedBy)
                            .to(Members::Table, Members::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建索引
        manager
            .create_index(
                Index::create()
                    .name("idx_file_contents_hash")
                    .table(FileContents::Table)
                    .col(FileContents::ContentHash)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_file_contents_size")
                    .table(FileContents::Table)
                    .col(FileContents::FileSize)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_file_contents_created")
                    .table(FileContents::Table)
                    .col(FileContents::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FileContents::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum FileContents {
    Table,
    Id,
    ContentHash,
    FileSize,
    StoragePath,
    MimeType,
    Width,
    Height,
    Duration,
    RefCount,
    CreatedAt,
    FirstUploadedBy,
}

#[derive(Iden)]
enum Members {
    #[iden = "members"]
    Table,
    Id,
}
