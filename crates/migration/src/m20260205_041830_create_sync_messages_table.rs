use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建同步消息表
        manager
            .create_table(
                Table::create()
                    .table(SyncMessages::Table)
                    .if_not_exists()
                    .col(pk_auto(SyncMessages::Id))
                    .col(string(SyncMessages::MessageType).not_null()) // 消息类型: sync_directory, sync_database
                    .col(integer(SyncMessages::Progress).default(0)) // 进度: 0-100
                    .col(string(SyncMessages::Status).not_null().default("pending")) // 状态: pending, processing, completed, failed
                    .col(text(SyncMessages::Payload)) // 消息内容(JSON格式，存储同步参数)
                    .col(text(SyncMessages::ErrorMessage).nullable()) // 错误信息
                    .col(timestamp(SyncMessages::CreatedAt).default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)))
                    .col(timestamp(SyncMessages::UpdatedAt).default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)))
                    .col(timestamp(SyncMessages::CompletedAt).nullable()) // 完成时间
                    .to_owned(),
            )
            .await?;

        // 创建索引
        manager
            .create_index(
                Index::create()
                    .name("idx_sync_messages_status")
                    .table(SyncMessages::Table)
                    .col(SyncMessages::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_sync_messages_message_type")
                    .table(SyncMessages::Table)
                    .col(SyncMessages::MessageType)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SyncMessages::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum SyncMessages {
    #[iden = "sync_messages"]
    Table,
    Id,
    MessageType,
    Progress,
    Status,
    Payload,
    ErrorMessage,
    CreatedAt,
    UpdatedAt,
    CompletedAt,
}
