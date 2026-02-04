use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建 member_files 关联表
        manager
            .create_table(
                Table::create()
                    .table(MemberFiles::Table)
                    .if_not_exists()
                    .col(pk_auto(MemberFiles::Id))
                    .col(integer(MemberFiles::MemberId).not_null())
                    .col(integer(MemberFiles::FileContentId).not_null())
                    .col(string(MemberFiles::FileName).not_null()) // 用户看到的原始文件名
                    .col(text(MemberFiles::Description)) // 文件描述
                    .col(
                        timestamp(MemberFiles::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(MemberFiles::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(MemberFiles::MemberId)
                            .to(Members::Table, Members::Id)
                            .on_delete(ForeignKeyAction::Cascade) // 成员删除时，关联文件记录也删除
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(MemberFiles::FileContentId)
                            .to(FileContents::Table, FileContents::Id)
                            .on_delete(ForeignKeyAction::Restrict) // 如果文件被引用，则不允许删除
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建索引
        manager
            .create_index(
                Index::create()
                    .name("idx_member_files_member_id")
                    .table(MemberFiles::Table)
                    .col(MemberFiles::MemberId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_member_files_file_content_id")
                    .table(MemberFiles::Table)
                    .col(MemberFiles::FileContentId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MemberFiles::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Members {
    #[iden = "members"]
    Table,
    Id,
}

#[derive(Iden)]
enum MemberFiles {
    #[iden = "member_files"]
    Table,
    Id,
    MemberId,
    FileContentId,
    FileName,
    Description,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum FileContents {
    Table,
    Id,
}
