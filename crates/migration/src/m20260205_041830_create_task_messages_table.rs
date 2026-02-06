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
                    .table(TaskMessages::Table)
                    .if_not_exists()
                    .col(pk_auto(TaskMessages::Id))
                    .col(integer(TaskMessages::MemberId).not_null()) // 关联的成员ID
                    .col(string(TaskMessages::MessageType).not_null()) // 消息类型: sync_directory, sync_database
                    .col(integer(TaskMessages::Progress).default(0)) // 进度: 0-100
                    .col(string(TaskMessages::Status).not_null().default("pending")) // 状态: pending, processing, completed, failed
                    .col(text(TaskMessages::Payload)) // 消息内容(JSON格式，存储同步参数)
                    .col(text(TaskMessages::ErrorMessage).null()) // 错误信息
                    .col(
                        timestamp(TaskMessages::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(TaskMessages::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(timestamp(TaskMessages::CompletedAt).null()) // 完成时间
                    .to_owned(),
            )
            .await?;

        // 创建索引
        manager
            .create_index(
                Index::create()
                    .name("idx_task_messages_status")
                    .table(TaskMessages::Table)
                    .col(TaskMessages::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_task_messages_message_type")
                    .table(TaskMessages::Table)
                    .col(TaskMessages::MessageType)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TaskMessages::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum TaskMessages {
    #[iden = "task_messages"]
    Table,
    Id,
    MemberId,
    MessageType,
    Progress,
    Status,
    Payload,
    ErrorMessage,
    CreatedAt,
    UpdatedAt,
    CompletedAt,
}
