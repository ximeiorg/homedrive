use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建 album_files 关联表
        manager
            .create_table(
                Table::create()
                    .table(AlbumFiles::Table)
                    .if_not_exists()
                    .col(pk_auto(AlbumFiles::Id))
                    .col(integer(AlbumFiles::AlbumId).not_null())
                    .col(integer(AlbumFiles::MemberFileId).not_null())
                    .col(
                        timestamp(AlbumFiles::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(AlbumFiles::AlbumId)
                            .to(Albums::Table, Albums::Id)
                            .on_delete(ForeignKeyAction::Cascade) // 相册删除时，关联记录也删除
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from_col(AlbumFiles::MemberFileId)
                            .to(MemberFiles::Table, MemberFiles::Id)
                            .on_delete(ForeignKeyAction::Cascade) // 文件删除时，关联记录也删除
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建 album_id + member_file_id 联合唯一索引（防止重复添加）
        manager
            .create_index(
                Index::create()
                    .name("idx_album_files_album_id_member_file_id")
                    .table(AlbumFiles::Table)
                    .col(AlbumFiles::AlbumId)
                    .col(AlbumFiles::MemberFileId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // 创建 album_id 索引
        manager
            .create_index(
                Index::create()
                    .name("idx_album_files_album_id")
                    .table(AlbumFiles::Table)
                    .col(AlbumFiles::AlbumId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AlbumFiles::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Albums {
    #[iden = "albums"]
    Table,
    Id,
}

#[derive(Iden)]
enum MemberFiles {
    #[iden = "member_files"]
    Table,
    Id,
}

#[derive(Iden)]
enum AlbumFiles {
    #[iden = "album_files"]
    Table,
    Id,
    AlbumId,
    MemberFileId,
    CreatedAt,
}
