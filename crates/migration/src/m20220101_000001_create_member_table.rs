use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Member::Table)
                    .if_not_exists()
                    .col(pk_auto(Member::Id))  // 用户ID，自动递增主键
                    .col(string(Member::Username).not_null())  // 用户名，不能为空
                    .col(string(Member::Password).not_null())  // 密码，不能为空
                    .col(string(Member::Avatar).null())  // 头像，可为空
                    .col(string_uniq(Member::StorageTag).not_null())  // 存储标签，唯一且不能为空
                    .col(timestamp(Member::CreatedAt).not_null())  // 创建日期，不能为空
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Member::Table).to_owned())
            .await
    }
}

// 定义会员表的列枚举
#[derive(Iden)]
enum Member {
    #[iden = "member"]
    Table,
    Id,
    Username,
    Password,
    Avatar,
    StorageTag,
    CreatedAt,
}