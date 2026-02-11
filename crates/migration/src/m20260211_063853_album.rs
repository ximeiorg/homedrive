use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建 albums 表
        manager
            .create_table(
                Table::create()
                    .table(Albums::Table)
                    .if_not_exists()
                    .col(pk_auto(Albums::Id))
                    .col(integer(Albums::MemberId).not_null())
                    .col(string(Albums::Name).not_null()) // 相册名称
                    .col(text(Albums::Description).null()) // 相册描述
                    .col(integer(Albums::CoverFileId).null()) // 封面图片 ID (member_files.id)
                    .col(
                        timestamp(Albums::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(Albums::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(Albums::MemberId)
                            .to(Members::Table, Members::Id)
                            .on_delete(ForeignKeyAction::Cascade) // 用户删除时，相册也删除
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建 member_id + name 联合唯一索引
        manager
            .create_index(
                Index::create()
                    .name("idx_albums_member_id_name")
                    .table(Albums::Table)
                    .col(Albums::MemberId)
                    .col(Albums::Name)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // 创建 member_id 索引
        manager
            .create_index(
                Index::create()
                    .name("idx_albums_member_id")
                    .table(Albums::Table)
                    .col(Albums::MemberId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Albums::Table).to_owned())
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
enum Albums {
    #[iden = "albums"]
    Table,
    Id,
    MemberId,
    Name,
    Description,
    CoverFileId,
    CreatedAt,
    UpdatedAt,
}
