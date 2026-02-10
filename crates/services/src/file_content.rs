use crate::error::Result;
use crate::{ServiceError, StorageService};
use chrono::Utc;
use std::sync::Arc;
use store::DatabaseConnection;
use store::member_file::query::{FileTypeFilter, ListMemberFilesQuery, SortField, SortOrder};
use tokio::io::{AsyncWriteExt, BufWriter};

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
    /// 计算文件内容的哈希值（使用 xxh3 算法）
    pub fn calculate_hash(data: &[u8]) -> String {
        use xxhash_rust::xxh3::xxh3_128;
        let hash_bytes = xxh3_128(data);
        format!("{:032x}", hash_bytes)
    }

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

    /// 流式上传文件（支持大文件）- 边写入边计算 hash
    /// 返回 (file_id, message)
    pub async fn upload_file_stream<S, E>(
        db: &DatabaseConnection,
        storage_root: &str,
        mut stream: S,
        mime_type: String,
        filename: String,
        uploader_id: i64,
    ) -> std::result::Result<(i64, String), ServiceError>
    where
        S: futures::Stream<Item = std::result::Result<bytes::Bytes, E>> + Send + Unpin,
        E: std::error::Error + Send + Sync + 'static,
    {
        use futures::StreamExt;
        use std::path::PathBuf;

        // 生成临时存储路径（先写入临时位置）
        let temp_key = format!("temp/{}_{}", Utc::now().timestamp_millis(), filename);
        let temp_path = PathBuf::from(storage_root).join(&temp_key);
        
        // 创建临时目录
        if let Some(parent) = temp_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| ServiceError::Storage(e.to_string()))?;
        }

        // 创建文件
        let file = tokio::fs::File::create(&temp_path)
            .await
            .map_err(|e| ServiceError::Storage(e.to_string()))?;
        
        let mut writer = BufWriter::new(file);
        let mut hasher = xxhash_rust::xxh3::Xxh3::new();
        let mut file_size: u64 = 0;

        // 流式写入并计算 hash
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| ServiceError::Storage(e.to_string()))?;
            writer
                .write_all(&chunk)
                .await
                .map_err(|e| ServiceError::Storage(e.to_string()))?;
            hasher.update(&chunk);
            file_size += chunk.len() as u64;
        }

        // 确保数据写入磁盘
        writer
            .flush()
            .await
            .map_err(|e| ServiceError::Storage(e.to_string()))?;

        // 获取最终 hash
        let content_hash = format!("{:032x}", hasher.digest128());

        // 检查 hash 是否已存在
        if let Some(existing_id) = Self::find_by_hash(db, &content_hash).await? {
            // 删除临时文件
            let _ = tokio::fs::remove_file(&temp_path).await;
            return Ok((existing_id, "File already exists".to_string()));
        }

        // 生成最终存储路径
        let storage_key = format!(
            "files/{}/{}/{}",
            Utc::now().format("%Y/%m/%d"),
            content_hash,
            filename
        );
        let final_path = PathBuf::from(storage_root).join(&storage_key);
        
        // 创建最终目录
        if let Some(parent) = final_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| ServiceError::Storage(e.to_string()))?;
        }
        
        // 移动临时文件到最终位置
        if let Err(e) = tokio::fs::rename(&temp_path, &final_path).await {
            // 如果跨设备移动失败，尝试复制+删除
            tokio::fs::copy(&temp_path, &final_path)
                .await
                .map_err(|copy_err| ServiceError::Storage(format!("{} (copy: {})", e, copy_err)))?;
            let _ = tokio::fs::remove_file(&temp_path).await;
        }

        // 创建数据库记录
        let create_data = store::file_content::mutation::CreateFileContent {
            content_hash,
            file_size: file_size as i64,
            storage_path: storage_key,
            mime_type,
            width: 0,
            height: 0,
            duration: 0,
            first_uploaded_by: uploader_id,
        };

        let file_content = store::file_content::mutation::Mutation::create(db, create_data).await?;
        Ok((file_content.id, "File uploaded successfully".to_string()))
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

    /// 列出用户文件（支持翻页、排序、类型过滤）
    pub async fn list_member_files(
        db: &DatabaseConnection,
        member_id: i64,
        page: Option<u64>,
        page_size: Option<u64>,
        sort_by: Option<String>,
        sort_order: Option<String>,
        file_type: Option<String>,
        search: Option<String>,
    ) -> Result<(
        Vec<(
            store::entity::member_files::Model,
            Option<store::entity::file_contents::Model>,
        )>,
        u64,
    )> {
        // 转换排序参数
        let sort_by = sort_by.as_ref().map(|s| match s.as_str() {
            "file_name" => SortField::FileName,
            "file_size" => SortField::FileSize,
            _ => SortField::CreatedAt,
        });

        let sort_order = sort_order.as_ref().map(|s| match s.as_str() {
            "asc" => SortOrder::Asc,
            _ => SortOrder::Desc,
        });

        // 转换文件类型参数
        let file_type = file_type.as_ref().map(|s| match s.as_str() {
            "image" => FileTypeFilter::Image,
            "video" => FileTypeFilter::Video,
            "audio" => FileTypeFilter::Audio,
            "document" => FileTypeFilter::Document,
            "archive" => FileTypeFilter::Archive,
            _ => FileTypeFilter::Other,
        });

        let list_query = ListMemberFilesQuery {
            page,
            page_size,
            sort_by,
            sort_order,
            file_type,
            search,
        };

        store::member_file::query::Query::list_files_by_member(db, member_id, list_query)
            .await
            .map_err(ServiceError::Database)
    }
}
