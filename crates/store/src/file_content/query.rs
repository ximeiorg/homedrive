use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use serde::{Deserialize, Serialize};

use super::super::entity::file_contents;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileContentQuery {
    pub id: Option<i64>,
    pub content_hash: Option<String>,
    pub mime_type: Option<String>,
    pub first_uploaded_by: Option<i64>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

pub struct Query;

impl Query {
    /// 根据ID查询单个文件内容
    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: i64,
    ) -> Result<Option<file_contents::Model>, sea_orm::DbErr> {
        file_contents::Entity::find_by_id(id).one(db).await
    }

    /// 根据内容哈希查询文件内容
    pub async fn find_by_hash(
        db: &DatabaseConnection,
        content_hash: &str,
    ) -> Result<Option<file_contents::Model>, sea_orm::DbErr> {
        file_contents::Entity::find()
            .filter(file_contents::Column::ContentHash.eq(content_hash))
            .one(db)
            .await
    }

    /// 根据上传者查询文件内容
    pub async fn find_by_uploader(
        db: &DatabaseConnection,
        uploader_id: i64,
    ) -> Result<Vec<file_contents::Model>, sea_orm::DbErr> {
        file_contents::Entity::find()
            .filter(file_contents::Column::FirstUploadedBy.eq(uploader_id))
            .all(db)
            .await
    }

    /// 根据MIME类型查询文件内容
    pub async fn find_by_mime_type(
        db: &DatabaseConnection,
        mime_type: &str,
    ) -> Result<Vec<file_contents::Model>, sea_orm::DbErr> {
        file_contents::Entity::find()
            .filter(file_contents::Column::MimeType.eq(mime_type))
            .all(db)
            .await
    }

    /// 查询所有文件内容（分页）
    pub async fn find_all(
        db: &DatabaseConnection,
        query: FileContentQuery,
    ) -> Result<(Vec<file_contents::Model>, u64), sea_orm::DbErr> {
        let mut select = file_contents::Entity::find();

        // 应用过滤条件
        if let Some(id) = query.id {
            select = select.filter(file_contents::Column::Id.eq(id));
        }
        if let Some(ref content_hash) = query.content_hash {
            select = select.filter(file_contents::Column::ContentHash.eq(content_hash));
        }
        if let Some(ref mime_type) = query.mime_type {
            select = select.filter(file_contents::Column::MimeType.eq(mime_type));
        }
        if let Some(first_uploaded_by) = query.first_uploaded_by {
            select = select.filter(file_contents::Column::FirstUploadedBy.eq(first_uploaded_by));
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

    /// 检查内容哈希是否存在
    pub async fn hash_exists(
        db: &DatabaseConnection,
        content_hash: &str,
    ) -> Result<bool, sea_orm::DbErr> {
        let count = file_contents::Entity::find()
            .filter(file_contents::Column::ContentHash.eq(content_hash))
            .count(db)
            .await?;

        Ok(count > 0)
    }

    /// 根据文件大小范围查询
    pub async fn find_by_size_range(
        db: &DatabaseConnection,
        min_size: i64,
        max_size: i64,
    ) -> Result<Vec<file_contents::Model>, sea_orm::DbErr> {
        file_contents::Entity::find()
            .filter(file_contents::Column::FileSize.gte(min_size))
            .filter(file_contents::Column::FileSize.lte(max_size))
            .all(db)
            .await
    }

    /// 获取引用计数最高的文件
    pub async fn find_most_referenced(
        db: &DatabaseConnection,
        limit: u64,
    ) -> Result<Vec<file_contents::Model>, sea_orm::DbErr> {
        file_contents::Entity::find()
            .order_by_desc(file_contents::Column::RefCount)
            .limit(limit)
            .all(db)
            .await
    }
}
