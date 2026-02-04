use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DeleteResult, EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};

use super::super::entity::file_contents;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFileContent {
    pub content_hash: String,
    pub file_size: i64,
    pub storage_path: String,
    pub mime_type: String,
    pub width: i64,
    pub height: i64,
    pub duration: i64,
    pub first_uploaded_by: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFileContent {
    pub storage_path: Option<String>,
    pub mime_type: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub duration: Option<i64>,
}

pub struct Mutation;

impl Mutation {
    /// 创建新的文件内容记录
    pub async fn create(
        db: &DatabaseConnection,
        data: CreateFileContent,
    ) -> Result<file_contents::Model, sea_orm::DbErr> {
        let file_content = file_contents::ActiveModel {
            content_hash: Set(data.content_hash),
            file_size: Set(data.file_size),
            storage_path: Set(data.storage_path),
            mime_type: Set(data.mime_type),
            width: Set(data.width),
            height: Set(data.height),
            duration: Set(data.duration),
            first_uploaded_by: Set(data.first_uploaded_by),
            ref_count: Set(1), // 初始引用计数为1
            ..Default::default()
        };

        file_content.insert(db).await
    }

    /// 更新文件内容信息
    pub async fn update(
        db: &DatabaseConnection,
        id: i64,
        data: UpdateFileContent,
    ) -> Result<file_contents::Model, sea_orm::DbErr> {
        let file_content: Option<file_contents::Model> =
            file_contents::Entity::find_by_id(id).one(db).await?;
        let file_content = file_content
            .ok_or_else(|| sea_orm::DbErr::Custom("File content not found".to_string()))?;

        let mut active_model: file_contents::ActiveModel = file_content.into();

        if let Some(storage_path) = data.storage_path {
            active_model.storage_path = Set(storage_path);
        }
        if let Some(mime_type) = data.mime_type {
            active_model.mime_type = Set(mime_type);
        }
        if let Some(width) = data.width {
            active_model.width = Set(width);
        }
        if let Some(height) = data.height {
            active_model.height = Set(height);
        }
        if let Some(duration) = data.duration {
            active_model.duration = Set(duration);
        }

        active_model.update(db).await
    }

    /// 删除文件内容记录
    pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<DeleteResult, sea_orm::DbErr> {
        file_contents::Entity::delete_by_id(id).exec(db).await
    }

    /// 批量删除文件内容记录
    pub async fn delete_batch(
        db: &DatabaseConnection,
        ids: Vec<i64>,
    ) -> Result<DeleteResult, sea_orm::DbErr> {
        file_contents::Entity::delete_many()
            .filter(file_contents::Column::Id.is_in(ids))
            .exec(db)
            .await
    }

    /// 增加引用计数
    pub async fn increment_ref_count(
        db: &DatabaseConnection,
        id: i64,
    ) -> Result<file_contents::Model, sea_orm::DbErr> {
        let file_content: Option<file_contents::Model> =
            file_contents::Entity::find_by_id(id).one(db).await?;
        let file_content = file_content
            .ok_or_else(|| sea_orm::DbErr::Custom("File content not found".to_string()))?;

        let current_ref_count = file_content.ref_count;
        let mut active_model: file_contents::ActiveModel = file_content.into();

        active_model.ref_count = Set(current_ref_count + 1);

        active_model.update(db).await
    }

    /// 减少引用计数
    pub async fn decrement_ref_count(
        db: &DatabaseConnection,
        id: i64,
    ) -> Result<file_contents::Model, sea_orm::DbErr> {
        let file_content: Option<file_contents::Model> =
            file_contents::Entity::find_by_id(id).one(db).await?;
        let file_content = file_content
            .ok_or_else(|| sea_orm::DbErr::Custom("File content not found".to_string()))?;

        let current_ref_count = file_content.ref_count;
        let mut active_model: file_contents::ActiveModel = file_content.into();

        active_model.ref_count = Set(std::cmp::max(0, current_ref_count - 1));

        active_model.update(db).await
    }

    /// 更新存储路径
    pub async fn update_storage_path(
        db: &DatabaseConnection,
        id: i64,
        new_path: String,
    ) -> Result<file_contents::Model, sea_orm::DbErr> {
        let file_content: Option<file_contents::Model> =
            file_contents::Entity::find_by_id(id).one(db).await?;
        let file_content = file_content
            .ok_or_else(|| sea_orm::DbErr::Custom("File content not found".to_string()))?;

        let mut active_model: file_contents::ActiveModel = file_content.into();
        active_model.storage_path = Set(new_path);
        active_model.update(db).await
    }

    /// 更新媒体元数据（宽高、时长等）
    pub async fn update_media_metadata(
        db: &DatabaseConnection,
        id: i64,
        width: i64,
        height: i64,
        duration: i64,
    ) -> Result<file_contents::Model, sea_orm::DbErr> {
        let file_content: Option<file_contents::Model> =
            file_contents::Entity::find_by_id(id).one(db).await?;
        let file_content = file_content
            .ok_or_else(|| sea_orm::DbErr::Custom("File content not found".to_string()))?;

        let mut active_model: file_contents::ActiveModel = file_content.into();
        active_model.width = Set(width);
        active_model.height = Set(height);
        active_model.duration = Set(duration);
        active_model.update(db).await
    }
}
