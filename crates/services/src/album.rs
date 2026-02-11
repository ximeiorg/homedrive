use crate::error::{Result, ServiceError};
use serde::{Deserialize, Serialize};
use store::DatabaseConnection;

/// 创建相册的参数
#[derive(Debug, Default, Clone, Deserialize)]
pub struct CreateAlbumParams {
    /// 相册名称
    pub name: String,
    /// 相册描述
    pub description: Option<String>,
    /// 封面图片 ID (member_file_id)
    pub cover_file_id: Option<i64>,
    /// 初始添加的文件 ID 列表
    pub file_ids: Option<Vec<i64>>,
}

/// 更新相册的参数
#[derive(Debug, Default, Clone, Deserialize)]
pub struct UpdateAlbumParams {
    /// 相册名称
    pub name: Option<String>,
    /// 相册描述
    pub description: Option<String>,
    /// 封面图片 ID
    pub cover_file_id: Option<i64>,
}

/// 相册信息响应
#[derive(Debug, Serialize)]
pub struct AlbumResponse {
    pub id: i64,
    pub member_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub cover_file_id: Option<i64>,
    pub file_count: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 相册列表项
#[derive(Debug, Serialize)]
pub struct AlbumListItem {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub cover_file_id: Option<i64>,
    pub file_count: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 相册服务
pub struct AlbumService;

impl AlbumService {
    /// 创建相册
    pub async fn create_album(
        db: &DatabaseConnection,
        member_id: i64,
        params: CreateAlbumParams,
    ) -> Result<AlbumResponse> {
        // 检查相册名称是否已存在
        if store::album::query::Query::album_name_exists(db, member_id, &params.name).await? {
            return Err(ServiceError::Validation("相册名称已存在".to_string()));
        }

        // 创建相册
        let create_data = store::album::mutation::CreateAlbum {
            member_id,
            name: params.name,
            description: params.description,
            cover_file_id: params.cover_file_id,
        };

        let album = store::album::mutation::Mutation::create_album(db, create_data).await?;

        // 如果有初始文件，添加到相册
        if let Some(file_ids) = params.file_ids {
            if !file_ids.is_empty() {
                store::album::mutation::Mutation::add_files_to_album(db, album.id, file_ids)
                    .await?;
            }
        }

        // 获取文件数量
        let file_count = store::album::query::Query::count_files_in_album(db, album.id).await?;

        Ok(AlbumResponse {
            id: album.id,
            member_id: album.member_id,
            name: album.name,
            description: album.description,
            cover_file_id: album.cover_file_id,
            file_count,
            created_at: album.created_at,
            updated_at: album.updated_at,
        })
    }

    /// 更新相册
    pub async fn update_album(
        db: &DatabaseConnection,
        member_id: i64,
        album_id: i64,
        params: UpdateAlbumParams,
    ) -> Result<AlbumResponse> {
        // 查找相册并验证所有权
        let album = store::album::query::Query::find_album_by_id(db, album_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("相册不存在".to_string()))?;

        if album.member_id != member_id {
            return Err(ServiceError::Forbidden("无权访问此相册".to_string()));
        }

        // 如果要更新名称，检查新名称是否已存在
        if let Some(ref name) = params.name {
            if name != &album.name {
                if store::album::query::Query::album_name_exists(db, member_id, name).await? {
                    return Err(ServiceError::Validation("相册名称已存在".to_string()));
                }
            }
        }

        // 更新相册
        let update_data = store::album::mutation::UpdateAlbum {
            name: params.name,
            description: params.description,
            cover_file_id: params.cover_file_id,
        };

        let updated_album =
            store::album::mutation::Mutation::update_album(db, album_id, update_data).await?;

        // 获取文件数量
        let file_count =
            store::album::query::Query::count_files_in_album(db, updated_album.id).await?;

        Ok(AlbumResponse {
            id: updated_album.id,
            member_id: updated_album.member_id,
            name: updated_album.name,
            description: updated_album.description,
            cover_file_id: updated_album.cover_file_id,
            file_count,
            created_at: updated_album.created_at,
            updated_at: updated_album.updated_at,
        })
    }

    /// 删除相册
    pub async fn delete_album(
        db: &DatabaseConnection,
        member_id: i64,
        album_id: i64,
    ) -> Result<()> {
        // 查找相册并验证所有权
        let album = store::album::query::Query::find_album_by_id(db, album_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("相册不存在".to_string()))?;

        if album.member_id != member_id {
            return Err(ServiceError::Forbidden("无权访问此相册".to_string()));
        }

        store::album::mutation::Mutation::delete_album(db, album_id).await?;

        Ok(())
    }

    /// 获取用户的相册列表
    pub async fn list_albums(
        db: &DatabaseConnection,
        member_id: i64,
        page: Option<u64>,
        page_size: Option<u64>,
    ) -> Result<(Vec<AlbumListItem>, u64)> {
        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(20);

        let (albums, total) =
            store::album::query::Query::list_albums_by_member(db, member_id, page, page_size)
                .await?;

        let mut items = Vec::new();
        for album in albums {
            let file_count = store::album::query::Query::count_files_in_album(db, album.id).await?;
            items.push(AlbumListItem {
                id: album.id,
                name: album.name,
                description: album.description,
                cover_file_id: album.cover_file_id,
                file_count,
                created_at: album.created_at,
                updated_at: album.updated_at,
            });
        }

        Ok((items, total))
    }

    /// 获取相册详情
    pub async fn get_album(
        db: &DatabaseConnection,
        member_id: i64,
        album_id: i64,
    ) -> Result<AlbumResponse> {
        let album = store::album::query::Query::find_album_by_id(db, album_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("相册不存在".to_string()))?;

        if album.member_id != member_id {
            return Err(ServiceError::Forbidden("无权访问此相册".to_string()));
        }

        let file_count = store::album::query::Query::count_files_in_album(db, album.id).await?;

        Ok(AlbumResponse {
            id: album.id,
            member_id: album.member_id,
            name: album.name,
            description: album.description,
            cover_file_id: album.cover_file_id,
            file_count,
            created_at: album.created_at,
            updated_at: album.updated_at,
        })
    }

    /// 添加文件到相册
    pub async fn add_files_to_album(
        db: &DatabaseConnection,
        member_id: i64,
        album_id: i64,
        file_ids: Vec<i64>,
    ) -> Result<u64> {
        // 验证相册所有权
        let album = store::album::query::Query::find_album_by_id(db, album_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("相册不存在".to_string()))?;

        if album.member_id != member_id {
            return Err(ServiceError::Forbidden("无权访问此相册".to_string()));
        }

        // 验证文件所有权并过滤已存在的文件
        let mut valid_file_ids = Vec::new();
        for file_id in file_ids {
            // 检查文件是否属于当前用户
            if let Some(member_file) =
                store::member_file::query::Query::find_by_id(db, file_id).await?
            {
                if member_file.member_id == member_id {
                    // 检查文件是否已在相册中
                    if !store::album::query::Query::file_in_album(db, album_id, file_id).await? {
                        valid_file_ids.push(file_id);
                    }
                }
            }
        }

        if valid_file_ids.is_empty() {
            return Ok(0);
        }

        let count = valid_file_ids.len() as u64;
        store::album::mutation::Mutation::add_files_to_album(db, album_id, valid_file_ids).await?;

        Ok(count)
    }

    /// 从相册中移除文件
    pub async fn remove_files_from_album(
        db: &DatabaseConnection,
        member_id: i64,
        album_id: i64,
        file_ids: Vec<i64>,
    ) -> Result<u64> {
        // 验证相册所有权
        let album = store::album::query::Query::find_album_by_id(db, album_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("相册不存在".to_string()))?;

        if album.member_id != member_id {
            return Err(ServiceError::Forbidden("无权访问此相册".to_string()));
        }

        let result = store::album::mutation::Mutation::remove_files_from_album(
            db,
            album_id,
            file_ids.clone(),
        )
        .await?;

        Ok(result.rows_affected)
    }

    /// 获取相册中的文件列表
    pub async fn list_album_files(
        db: &DatabaseConnection,
        member_id: i64,
        album_id: i64,
        page: Option<u64>,
        page_size: Option<u64>,
    ) -> Result<(
        Vec<(
            store::entity::member_files::Model,
            Option<store::entity::file_contents::Model>,
        )>,
        u64,
    )> {
        // 验证相册所有权
        let album = store::album::query::Query::find_album_by_id(db, album_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("相册不存在".to_string()))?;

        if album.member_id != member_id {
            return Err(ServiceError::Forbidden("无权访问此相册".to_string()));
        }

        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(100);

        let (album_files, total) =
            store::album::query::Query::list_files_in_album(db, album_id, page, page_size).await?;

        // 转换为 (member_files, file_contents) 格式
        let results: Vec<(
            store::entity::member_files::Model,
            Option<store::entity::file_contents::Model>,
        )> = album_files
            .into_iter()
            .map(|(_, member_file, file_content)| (member_file, file_content))
            .collect();

        Ok((results, total))
    }
}
