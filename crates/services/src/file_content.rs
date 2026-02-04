use crate::error::Result;
use crate::{ServiceError, StorageService};
use chrono::Utc;
use std::sync::Arc;
use store::DatabaseConnection;

/// 文件上传响应
#[derive(Debug, serde::Serialize)]
pub struct FileUploadResponse {
    pub success: bool,
    pub file_id: i64,
    pub message: String,
}

/// 文件检查响应
#[derive(Debug, serde::Serialize)]
pub struct FileCheckResponse {
    pub exists: bool,
}

/// 文件服务
pub struct FileService;

impl FileService {
    /// 检查文件哈希是否存在
    pub async fn check_hash_exists(db: &DatabaseConnection, hash: &str) -> Result<bool> {
        let exists = store::file_content::query::Query::hash_exists(db, hash).await?;
        Ok(exists)
    }

    /// 检查文件哈希是否存在，返回文件ID
    pub async fn find_by_hash(db: &DatabaseConnection, hash: &str) -> Result<Option<i64>> {
        let file = store::file_content::query::Query::find_by_hash(db, hash).await?;
        Ok(file.map(|f| f.id))
    }

    /// 上传文件（保存文件数据到存储，并创建数据库记录）
    pub async fn upload_file(
        db: &DatabaseConnection,
        storage: &Arc<dyn StorageService>,
        content: Vec<u8>,
        content_hash: String,
        mime_type: String,
        filename: String,
        uploader_id: i64,
    ) -> Result<i64> {
        // 生成存储路径：files/年月/哈希/文件名
        let storage_key = format!(
            "files/{}/{}/{}",
            Utc::now().format("%Y/%m/%d"),
            content_hash,
            filename
        );

        // 保存文件到存储
        storage.save(&storage_key, &content).await?;

        // 创建数据库记录
        let storage_path = storage_key;
        let create_data = store::file_content::mutation::CreateFileContent {
            content_hash,
            file_size: content.len() as i64,
            storage_path,
            mime_type,
            width: 0,
            height: 0,
            duration: 0,
            first_uploaded_by: uploader_id,
        };

        let file_content = store::file_content::mutation::Mutation::create(db, create_data).await?;
        Ok(file_content.id)
    }

    /// 删除文件（删除存储文件，并删除数据库记录）
    pub async fn delete_file(
        db: &DatabaseConnection,
        storage: &Arc<dyn StorageService>,
        file_id: i64,
    ) -> Result<()> {
        // 查找文件记录
        let file = store::file_content::query::Query::find_by_id(db, file_id)
            .await?
            .ok_or(ServiceError::FileNotFound)?;

        // 删除存储文件
        storage.delete(&file.storage_path).await?;

        // 删除数据库记录
        store::file_content::mutation::Mutation::delete(db, file_id).await?;

        Ok(())
    }

    /// 获取文件信息
    pub async fn get_file_info(
        db: &DatabaseConnection,
        file_id: i64,
    ) -> Result<Option<store::entity::file_contents::Model>> {
        let file = store::file_content::query::Query::find_by_id(db, file_id).await?;
        Ok(file)
    }

    /// 获取文件内容
    pub async fn get_file_content(
        db: &DatabaseConnection,
        storage: &Arc<dyn StorageService>,
        file_id: i64,
    ) -> Result<Vec<u8>> {
        let file = store::file_content::query::Query::find_by_id(db, file_id)
            .await?
            .ok_or(ServiceError::FileNotFound)?;

        storage.get(&file.storage_path).await
    }

    /// 获取文件 URL
    pub async fn get_file_url(
        db: &DatabaseConnection,
        storage: &Arc<dyn StorageService>,
        file_id: i64,
    ) -> Result<String> {
        let file = store::file_content::query::Query::find_by_id(db, file_id)
            .await?
            .ok_or(ServiceError::FileNotFound)?;

        storage.url(&file.storage_path).await
    }
}
