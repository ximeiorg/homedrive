use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DeleteResult, EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};

use super::super::entity::member_files;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMemberFile {
    pub member_id: i64,
    pub file_content_id: i64,
    pub file_name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMemberFile {
    pub file_name: Option<String>,
    pub description: Option<String>,
}

pub struct Mutation;

impl Mutation {
    /// 创建新的成员文件关联记录
    pub async fn create(
        db: &DatabaseConnection,
        data: CreateMemberFile,
    ) -> Result<member_files::Model, sea_orm::DbErr> {
        let member_file = member_files::ActiveModel {
            member_id: Set(data.member_id),
            file_content_id: Set(data.file_content_id),
            file_name: Set(data.file_name),
            description: Set(data.description),
            ..Default::default()
        };

        member_file.insert(db).await
    }

    /// 更新成员文件信息
    pub async fn update(
        db: &DatabaseConnection,
        id: i64,
        data: UpdateMemberFile,
    ) -> Result<member_files::Model, sea_orm::DbErr> {
        let member_file: Option<member_files::Model> =
            member_files::Entity::find_by_id(id).one(db).await?;
        let member_file = member_file
            .ok_or_else(|| sea_orm::DbErr::Custom("Member file not found".to_string()))?;

        let mut active_model: member_files::ActiveModel = member_file.into();

        if let Some(file_name) = data.file_name {
            active_model.file_name = Set(file_name);
        }
        if let Some(description) = data.description {
            active_model.description = Set(description);
        }

        active_model.update(db).await
    }

    /// 删除成员文件记录
    pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<DeleteResult, sea_orm::DbErr> {
        member_files::Entity::delete_by_id(id).exec(db).await
    }

    /// 批量删除成员文件记录
    pub async fn delete_batch(
        db: &DatabaseConnection,
        ids: Vec<i64>,
    ) -> Result<DeleteResult, sea_orm::DbErr> {
        member_files::Entity::delete_many()
            .filter(member_files::Column::Id.is_in(ids))
            .exec(db)
            .await
    }

    /// 根据成员ID和文件内容ID删除关联
    pub async fn delete_by_association(
        db: &DatabaseConnection,
        member_id: i64,
        file_content_id: i64,
    ) -> Result<DeleteResult, sea_orm::DbErr> {
        member_files::Entity::delete_many()
            .filter(member_files::Column::MemberId.eq(member_id))
            .filter(member_files::Column::FileContentId.eq(file_content_id))
            .exec(db)
            .await
    }

    /// 更新文件名
    pub async fn update_file_name(
        db: &DatabaseConnection,
        id: i64,
        new_file_name: String,
    ) -> Result<member_files::Model, sea_orm::DbErr> {
        let member_file: Option<member_files::Model> =
            member_files::Entity::find_by_id(id).one(db).await?;
        let member_file = member_file
            .ok_or_else(|| sea_orm::DbErr::Custom("Member file not found".to_string()))?;

        let mut active_model: member_files::ActiveModel = member_file.into();
        active_model.file_name = Set(new_file_name);
        active_model.update(db).await
    }

    /// 更新文件描述
    pub async fn update_description(
        db: &DatabaseConnection,
        id: i64,
        new_description: String,
    ) -> Result<member_files::Model, sea_orm::DbErr> {
        let member_file: Option<member_files::Model> =
            member_files::Entity::find_by_id(id).one(db).await?;
        let member_file = member_file
            .ok_or_else(|| sea_orm::DbErr::Custom("Member file not found".to_string()))?;

        let mut active_model: member_files::ActiveModel = member_file.into();
        active_model.description = Set(new_description);
        active_model.update(db).await
    }

    /// 移动文件到另一个成员（更改成员ID）
    pub async fn transfer_to_another_member(
        db: &DatabaseConnection,
        id: i64,
        new_member_id: i64,
    ) -> Result<member_files::Model, sea_orm::DbErr> {
        let member_file: Option<member_files::Model> =
            member_files::Entity::find_by_id(id).one(db).await?;
        let member_file = member_file
            .ok_or_else(|| sea_orm::DbErr::Custom("Member file not found".to_string()))?;

        let mut active_model: member_files::ActiveModel = member_file.into();
        active_model.member_id = Set(new_member_id);
        active_model.update(db).await
    }

    /// 更新文件关联（更改文件内容ID）
    pub async fn update_file_association(
        db: &DatabaseConnection,
        id: i64,
        new_file_content_id: i64,
    ) -> Result<member_files::Model, sea_orm::DbErr> {
        let member_file: Option<member_files::Model> =
            member_files::Entity::find_by_id(id).one(db).await?;
        let member_file = member_file
            .ok_or_else(|| sea_orm::DbErr::Custom("Member file not found".to_string()))?;

        let mut active_model: member_files::ActiveModel = member_file.into();
        active_model.file_content_id = Set(new_file_content_id);
        active_model.update(db).await
    }
}
