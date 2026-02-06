use crate::{Result, TaskHandler, TaskPayload};
use sea_orm::{
    ActiveModelTrait, ColumnTrait as _, DatabaseConnection, EntityTrait, QueryFilter as _, Set,
};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tracing::{debug, info, warn};

/// 同步目录处理器
/// 将 storage 目录下的文件同步到数据库
pub struct SyncFilesHandler {
    /// 存储根目录
    storage_root: String,
    /// 数据库连接（Arc 包装以支持 Clone）
    conn: Arc<DatabaseConnection>,
}

impl SyncFilesHandler {
    pub fn new(storage_root: String, conn: Arc<DatabaseConnection>) -> Self {
        Self { storage_root, conn }
    }

    /// 计算文件的 xxhash3 哈希值
    /// 与前端 hash-wasm 的 xxh3 保持一致
    fn calculate_hash(content: &[u8]) -> String {
        // 使用 xxh3 64位模式（与前端一致）
        let hash = xxhash_rust::xxh3::xxh3_64(content);
        format!("{:016x}", hash)
    }

    /// 获取文件的 MIME 类型
    fn get_mime_type(file_name: &str) -> String {
        let ext = Path::new(file_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "svg" => "image/svg+xml",
            "mp4" => "video/mp4",
            "webm" => "video/webm",
            "mp3" => "audio/mpeg",
            "wav" => "audio/wav",
            "pdf" => "application/pdf",
            "txt" => "text/plain",
            "html" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "json" => "application/json",
            "zip" => "application/zip",
            _ => "application/octet-stream",
        }
        .to_string()
    }
}

#[async_trait::async_trait]
impl TaskHandler for SyncFilesHandler {
    fn task_type(&self) -> &'static str {
        "sync_files"
    }

    /// 处理任务
    /// 同步 storage_root 下的目录，其实 storage_root 下的目录是用户的 storage_tag 名称，
    /// 每个目录下的文件都是用户的文件，需要同步到用户的存储中。
    ///
    /// 步骤：
    /// 1. 列出 storage_root 下的所有目录（storage_tag）
    /// 2. 查询 storage_tag 对应的用户 id (member_id)
    /// 3. 对比文件是否在 file_contents 表中存在（通过 hash）
    /// 4. 对比 member_files 表是否存在记录
    /// 5. 如果不存在，添加记录到数据表
    async fn handle(&self, _payload: &TaskPayload) -> Result<()> {
        let storage_root = Path::new(&self.storage_root);

        info!("Starting file synchronization from: {}", self.storage_root);

        // 1. 列出 storage_root 下的所有目录（storage_tag）
        let mut entries = fs::read_dir(storage_root).await.map_err(|e| {
            crate::ServiceError::Storage(format!("Failed to read storage root directory: {}", e))
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            crate::ServiceError::Storage(format!("Failed to read directory entry: {}", e))
        })? {
            let entry_path = entry.path();

            // 只处理目录（storage_tag）
            if !entry_path.is_dir() {
                debug!("Skipping non-directory entry: {:?}", entry_path);
                continue;
            }

            let storage_tag = entry.file_name().to_string_lossy().to_string();
            debug!("Processing storage_tag: {}", storage_tag);

            // 2. 查询 storage_tag 对应的用户 id (member_id)
            let member = store::entity::members::Entity::find()
                .filter(store::entity::members::Column::StorageTag.eq(storage_tag.clone()))
                .one(&*self.conn)
                .await?;

            if member.is_none() {
                warn!("No member found for storage_tag: {}, skipping", storage_tag);
                continue;
            }

            let member_id = member.unwrap().id;
            info!(
                "Found member {} for storage_tag: {}",
                member_id, storage_tag
            );

            // 3. 使用栈遍历目录下的所有文件（非递归）
            let mut dir_stack = vec![entry_path.clone()];

            while let Some(current_dir) = dir_stack.pop() {
                let mut entries = fs::read_dir(&current_dir).await.map_err(|e| {
                    crate::ServiceError::Storage(format!("Failed to read directory: {}", e))
                })?;

                while let Some(entry) = entries.next_entry().await.map_err(|e| {
                    crate::ServiceError::Storage(format!("Failed to read directory entry: {}", e))
                })? {
                    let entry_path = entry.path();

                    if entry_path.is_dir() {
                        // 将子目录加入栈中
                        dir_stack.push(entry_path);
                    } else if entry_path.is_file() {
                        // 处理文件
                        self.sync_file(&entry_path, member_id).await?;
                    }
                }
            }
        }

        info!("File synchronization completed");
        Ok(())
    }
}

impl SyncFilesHandler {
    /// 同步单个文件
    async fn sync_file(&self, file_path: &Path, member_id: i64) -> Result<()> {
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| crate::ServiceError::Storage("Invalid file name".to_string()))?
            .to_string();

        let file_name_lower = file_name.to_lowercase();

        // 跳过隐藏文件和系统文件
        if file_name.starts_with('.')
            || file_name_lower == ".ds_store"
            || file_name_lower == "thumbs.db"
        {
            debug!("Skipping hidden/system file: {:?}", file_path);
            return Ok(());
        }

        // 读取文件内容用于计算哈希
        let content = fs::read(file_path)
            .await
            .map_err(|e| crate::ServiceError::Storage(format!("Failed to read file: {}", e)))?;

        // 计算文件哈希（与前端一致）
        let content_hash = Self::calculate_hash(&content);
        debug!("File: {}, Hash: {}", file_name, content_hash);

        let file_size = content.len() as i64;

        // 获取 MIME 类型
        let mime_type = Self::get_mime_type(&file_name);

        // 4. 检查 file_contents 表中是否已存在
        let existing_content = store::entity::file_contents::Entity::find()
            .filter(store::entity::file_contents::Column::ContentHash.eq(content_hash.clone()))
            .one(&*self.conn)
            .await?;

        let file_content_id: i64;

        if let Some(content) = existing_content {
            // 文件已存在，更新 ref_count
            file_content_id = content.id;
            let mut active_model: store::entity::file_contents::ActiveModel = content.into();
            // 获取当前 ref_count，如果不存在则默认为 0
            let current_ref_count = match active_model.ref_count {
                sea_orm::ActiveValue::Set(v) => v,
                _ => 0,
            };
            active_model.ref_count = Set(current_ref_count + 1);
            active_model.update(&*self.conn).await?;
            debug!(
                "File content already exists, id: {}, updated ref_count",
                file_content_id
            );
        } else {
            // 5. 添加新记录到 file_contents
            // 计算相对路径（相对于 storage_root）
            let storage_path = file_path.to_string_lossy().replace(&self.storage_root, "");
            let storage_path = storage_path.trim_start_matches('/').to_string();

            let new_content = store::entity::file_contents::ActiveModel {
                content_hash: Set(content_hash.clone()),
                file_size: Set(file_size),
                storage_path: Set(storage_path),
                mime_type: Set(mime_type),
                width: Set(0),
                height: Set(0),
                duration: Set(0),
                ref_count: Set(1),
                created_at: Set(chrono::Utc::now()),
                first_uploaded_by: Set(member_id),
                ..Default::default()
            };

            let result = new_content.insert(&*self.conn).await?;
            file_content_id = result.id;
            debug!("Created new file_content, id: {}", file_content_id);
        }

        // 6. 检查 member_files 表中是否已存在
        let existing_member_file = store::entity::member_files::Entity::find()
            .filter(store::entity::member_files::Column::MemberId.eq(member_id))
            .filter(store::entity::member_files::Column::FileContentId.eq(file_content_id))
            .one(&*self.conn)
            .await?;

        if existing_member_file.is_none() {
            // 添加新记录到 member_files
            let new_member_file = store::entity::member_files::ActiveModel {
                member_id: Set(member_id),
                file_content_id: Set(file_content_id),
                file_name: Set(file_name.clone()),
                description: Set(String::new()),
                created_at: Set(chrono::Utc::now()),
                updated_at: Set(chrono::Utc::now()),
                ..Default::default()
            };

            new_member_file.insert(&*self.conn).await?;
            debug!(
                "Created new member_file for member: {}, file: {}",
                member_id, file_name
            );
        } else {
            debug!(
                "Member file already exists for member: {}, file: {}",
                member_id, file_name
            );
        }

        info!("Synced file: {} (hash: {})", file_name, content_hash);
        Ok(())
    }
}
