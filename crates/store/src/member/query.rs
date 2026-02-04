use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use super::super::entity::members;

#[derive(Debug, Serialize, Deserialize)]
pub struct MemberQuery {
    pub id: Option<i64>,
    pub username: Option<String>,
    pub storage_tag: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

pub struct Query;

impl Query {
    /// 根据ID查询单个成员
    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: i64,
    ) -> Result<Option<members::Model>, sea_orm::DbErr> {
        members::Entity::find_by_id(id).one(db).await
    }

    /// 根据用户名查询成员
    pub async fn find_by_username(
        db: &DatabaseConnection,
        username: &str,
    ) -> Result<Option<members::Model>, sea_orm::DbErr> {
        members::Entity::find()
            .filter(members::Column::Username.eq(username))
            .one(db)
            .await
    }

    /// 根据存储标签查询成员
    pub async fn find_by_storage_tag(
        db: &DatabaseConnection,
        storage_tag: &str,
    ) -> Result<Option<members::Model>, sea_orm::DbErr> {
        members::Entity::find()
            .filter(members::Column::StorageTag.eq(storage_tag))
            .one(db)
            .await
    }

    /// 查询所有成员（分页）
    pub async fn find_all(
        db: &DatabaseConnection,
        query: MemberQuery,
    ) -> Result<(Vec<members::Model>, u64), sea_orm::DbErr> {
        let mut select = members::Entity::find();

        // 应用过滤条件
        if let Some(id) = query.id {
            select = select.filter(members::Column::Id.eq(id));
        }
        if let Some(ref username) = query.username {
            select = select.filter(members::Column::Username.eq(username));
        }
        if let Some(ref storage_tag) = query.storage_tag {
            select = select.filter(members::Column::StorageTag.eq(storage_tag));
        }

        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);

        // 获取总数
        let total = select.clone().count(db).await?;

        // 分页查询
        let paginator = select.paginate(db, page_size);
        let results = paginator.fetch_page(page - 1).await?;

        Ok((results, total))
    }

    /// 检查用户名是否存在
    pub async fn username_exists(
        db: &DatabaseConnection,
        username: &str,
    ) -> Result<bool, sea_orm::DbErr> {
        let count = members::Entity::find()
            .filter(members::Column::Username.eq(username))
            .count(db)
            .await?;

        Ok(count > 0)
    }

    /// 检查存储标签是否存在
    pub async fn storage_tag_exists(
        db: &DatabaseConnection,
        storage_tag: &str,
    ) -> Result<bool, sea_orm::DbErr> {
        let count = members::Entity::find()
            .filter(members::Column::StorageTag.eq(storage_tag))
            .count(db)
            .await?;

        Ok(count > 0)
    }

    /// 检查 member 表是否为空（没有成员）
    pub async fn is_empty(db: &DatabaseConnection) -> Result<bool, sea_orm::DbErr> {
        let count = members::Entity::find().count(db).await?;
        Ok(count == 0)
    }

    /// 获取 member 总数
    pub async fn count_all(db: &DatabaseConnection) -> Result<u64, sea_orm::DbErr> {
        members::Entity::find().count(db).await
    }
}
