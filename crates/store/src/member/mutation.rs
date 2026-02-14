use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DeleteResult, EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};

use super::super::entity::members::{self, MemberRole};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMember {
    pub username: String,
    pub password: String,
    pub avatar: Option<String>,
    pub storage_tag: String,
    pub role: Option<MemberRole>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMember {
    pub username: Option<String>,
    pub password: Option<String>,
    pub avatar: Option<String>,
    pub storage_tag: Option<String>,
    pub role: Option<MemberRole>,
}

pub struct Mutation;

impl Mutation {
    /// 创建新成员
    pub async fn create(
        db: &DatabaseConnection,
        data: CreateMember,
    ) -> Result<members::Model, sea_orm::DbErr> {
        let member = members::ActiveModel {
            username: Set(data.username),
            password: Set(data.password),
            avatar: Set(data.avatar),
            storage_tag: Set(data.storage_tag),
            created_at: Set(chrono::Utc::now()),
            role: Set(data.role.unwrap_or(MemberRole::User)),
            ..Default::default()
        };

        member.insert(db).await
    }

    /// 更新成员信息
    pub async fn update(
        db: &DatabaseConnection,
        id: i64,
        data: UpdateMember,
    ) -> Result<members::Model, sea_orm::DbErr> {
        let member: Option<members::Model> = members::Entity::find_by_id(id).one(db).await?;
        let member =
            member.ok_or_else(|| sea_orm::DbErr::Custom("Member not found".to_string()))?;

        let mut active_model: members::ActiveModel = member.into();

        if let Some(username) = data.username {
            active_model.username = Set(username);
        }
        if let Some(password) = data.password {
            active_model.password = Set(password);
        }
        if let Some(avatar) = data.avatar {
            active_model.avatar = Set(Some(avatar));
        }
        if let Some(storage_tag) = data.storage_tag {
            active_model.storage_tag = Set(storage_tag);
        }
        if let Some(role) = data.role {
            active_model.role = Set(role);
        }

        active_model.update(db).await
    }

    /// 删除成员
    pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<DeleteResult, sea_orm::DbErr> {
        members::Entity::delete_by_id(id).exec(db).await
    }

    /// 批量删除成员
    pub async fn delete_batch(
        db: &DatabaseConnection,
        ids: Vec<i64>,
    ) -> Result<DeleteResult, sea_orm::DbErr> {
        members::Entity::delete_many()
            .filter(members::Column::Id.is_in(ids))
            .exec(db)
            .await
    }

    /// 更新成员头像
    pub async fn update_avatar(
        db: &DatabaseConnection,
        id: i64,
        avatar: String,
    ) -> Result<members::Model, sea_orm::DbErr> {
        let member: Option<members::Model> = members::Entity::find_by_id(id).one(db).await?;
        let member =
            member.ok_or_else(|| sea_orm::DbErr::Custom("Member not found".to_string()))?;

        let mut active_model: members::ActiveModel = member.into();
        active_model.avatar = Set(Some(avatar));
        active_model.update(db).await
    }

    /// 更新密码
    pub async fn update_password(
        db: &DatabaseConnection,
        id: i64,
        new_password: String,
    ) -> Result<members::Model, sea_orm::DbErr> {
        let member: Option<members::Model> = members::Entity::find_by_id(id).one(db).await?;
        let member =
            member.ok_or_else(|| sea_orm::DbErr::Custom("Member not found".to_string()))?;

        let mut active_model: members::ActiveModel = member.into();
        active_model.password = Set(new_password);
        active_model.update(db).await
    }

    /// 更新成员角色
    pub async fn update_role(
        db: &DatabaseConnection,
        id: i64,
        role: MemberRole,
    ) -> Result<members::Model, sea_orm::DbErr> {
        let member: Option<members::Model> = members::Entity::find_by_id(id).one(db).await?;
        let member =
            member.ok_or_else(|| sea_orm::DbErr::Custom("Member not found".to_string()))?;

        let mut active_model: members::ActiveModel = member.into();
        active_model.role = Set(role);
        active_model.update(db).await
    }
}
