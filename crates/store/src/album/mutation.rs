use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DeleteResult, EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};

use super::super::entity::{album_files, albums};

/// 创建相册的参数
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAlbum {
    pub member_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub cover_file_id: Option<i64>,
}

/// 更新相册的参数
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UpdateAlbum {
    pub name: Option<String>,
    pub description: Option<String>,
    pub cover_file_id: Option<i64>,
}

/// 添加文件到相册的参数
#[derive(Debug, Serialize, Deserialize)]
pub struct AddFileToAlbum {
    pub album_id: i64,
    pub member_file_id: i64,
}

pub struct Mutation;

impl Mutation {
    /// 创建新相册
    pub async fn create_album(
        db: &DatabaseConnection,
        data: CreateAlbum,
    ) -> Result<albums::Model, sea_orm::DbErr> {
        let album = albums::ActiveModel {
            member_id: Set(data.member_id),
            name: Set(data.name),
            description: Set(data.description),
            cover_file_id: Set(data.cover_file_id),
            ..Default::default()
        };

        album.insert(db).await
    }

    /// 更新相册信息
    pub async fn update_album(
        db: &DatabaseConnection,
        id: i64,
        data: UpdateAlbum,
    ) -> Result<albums::Model, sea_orm::DbErr> {
        let album: Option<albums::Model> = albums::Entity::find_by_id(id).one(db).await?;
        let album = album.ok_or_else(|| sea_orm::DbErr::Custom("Album not found".to_string()))?;

        let mut active_model: albums::ActiveModel = album.into();

        if let Some(name) = data.name {
            active_model.name = Set(name);
        }
        if let Some(description) = data.description {
            active_model.description = Set(Some(description));
        }
        if let Some(cover_file_id) = data.cover_file_id {
            active_model.cover_file_id = Set(Some(cover_file_id));
        }

        active_model.update(db).await
    }

    /// 删除相册（同时删除关联的 album_files 记录）
    pub async fn delete_album(
        db: &DatabaseConnection,
        album_id: i64,
    ) -> Result<DeleteResult, sea_orm::DbErr> {
        // 先删除关联的 album_files 记录
        album_files::Entity::delete_many()
            .filter(album_files::Column::AlbumId.eq(album_id))
            .exec(db)
            .await?;

        // 再删除相册本身
        albums::Entity::delete_by_id(album_id).exec(db).await
    }

    /// 添加文件到相册
    pub async fn add_file_to_album(
        db: &DatabaseConnection,
        data: AddFileToAlbum,
    ) -> Result<album_files::Model, sea_orm::DbErr> {
        let album_file = album_files::ActiveModel {
            album_id: Set(data.album_id),
            member_file_id: Set(data.member_file_id),
            ..Default::default()
        };

        album_file.insert(db).await
    }

    /// 批量添加文件到相册
    pub async fn add_files_to_album(
        db: &DatabaseConnection,
        album_id: i64,
        member_file_ids: Vec<i64>,
    ) -> Result<Vec<album_files::Model>, sea_orm::DbErr> {
        let mut results = Vec::new();
        for member_file_id in member_file_ids {
            let album_file = album_files::ActiveModel {
                album_id: Set(album_id),
                member_file_id: Set(member_file_id),
                ..Default::default()
            };
            let result = album_file.insert(db).await?;
            results.push(result);
        }
        Ok(results)
    }

    /// 从相册中移除文件
    pub async fn remove_file_from_album(
        db: &DatabaseConnection,
        album_id: i64,
        member_file_id: i64,
    ) -> Result<DeleteResult, sea_orm::DbErr> {
        album_files::Entity::delete_many()
            .filter(album_files::Column::AlbumId.eq(album_id))
            .filter(album_files::Column::MemberFileId.eq(member_file_id))
            .exec(db)
            .await
    }

    /// 批量从相册中移除文件
    pub async fn remove_files_from_album(
        db: &DatabaseConnection,
        album_id: i64,
        member_file_ids: Vec<i64>,
    ) -> Result<DeleteResult, sea_orm::DbErr> {
        album_files::Entity::delete_many()
            .filter(album_files::Column::AlbumId.eq(album_id))
            .filter(album_files::Column::MemberFileId.is_in(member_file_ids))
            .exec(db)
            .await
    }

    /// 清空相册中的所有文件
    pub async fn clear_album(
        db: &DatabaseConnection,
        album_id: i64,
    ) -> Result<DeleteResult, sea_orm::DbErr> {
        album_files::Entity::delete_many()
            .filter(album_files::Column::AlbumId.eq(album_id))
            .exec(db)
            .await
    }

    /// 更新相册封面
    pub async fn update_album_cover(
        db: &DatabaseConnection,
        id: i64,
        cover_file_id: Option<i64>,
    ) -> Result<albums::Model, sea_orm::DbErr> {
        let album: Option<albums::Model> = albums::Entity::find_by_id(id).one(db).await?;
        let album = album.ok_or_else(|| sea_orm::DbErr::Custom("Album not found".to_string()))?;

        let mut active_model: albums::ActiveModel = album.into();
        active_model.cover_file_id = Set(cover_file_id);
        active_model.update(db).await
    }
}
