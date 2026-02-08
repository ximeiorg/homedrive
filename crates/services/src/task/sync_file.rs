use crate::{Result, TaskHandler, TaskPayload};
use sea_orm::{
    ActiveModelTrait, ColumnTrait as _, DatabaseConnection, EntityTrait, QueryFilter as _, Set,
};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tracing::{debug, info, warn};

/// 进度更新节流配置
const PROGRESS_UPDATE_INTERVAL: usize = 10; // 每处理 10 个文件更新一次进度
const PROGRESS_UPDATE_MIN_INTERVAL_MS: u64 = 500; // 最小更新间隔 500ms

/// 同步目录处理器
/// 将 storage 目录下的文件同步到数据库
pub struct SyncFilesHandler {
    /// 存储根目录
    storage_root: String,
    /// 数据库连接（Arc 包装以支持 Clone）
    conn: Arc<DatabaseConnection>,
    /// 任务消息 ID（用于进度更新）
    task_message_id: Option<i64>,
    /// 进度追踪器
    progress_tracker: ProgressTracker,
}

impl SyncFilesHandler {
    pub fn new(storage_root: String, conn: Arc<DatabaseConnection>) -> Self {
        Self {
            storage_root,
            conn,
            task_message_id: None,
            progress_tracker: ProgressTracker::new(),
        }
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

/// 进度追踪器
#[derive(Clone, Debug)]
struct ProgressTracker {
    /// 已处理文件数
    processed_count: usize,
    /// 总文件数（预估）
    total_count: usize,
    /// 上次更新时间
    last_update: std::time::Instant,
    /// 已跳过的文件数
    skipped_count: usize,
}

impl ProgressTracker {
    fn new() -> Self {
        Self {
            processed_count: 0,
            total_count: 0,
            last_update: std::time::Instant::now(),
            skipped_count: 0,
        }
    }

    fn increment_processed(&mut self) {
        self.processed_count += 1;
    }

    fn increment_skipped(&mut self) {
        self.skipped_count += 1;
    }

    fn get_current_count(&self) -> usize {
        self.processed_count + self.skipped_count
    }

    fn should_update(&self) -> bool {
        let elapsed = self.last_update.elapsed().as_millis() as u64;
        self.get_current_count() % PROGRESS_UPDATE_INTERVAL == 0
            || elapsed >= PROGRESS_UPDATE_MIN_INTERVAL_MS
    }

    fn reset_update_time(&mut self) {
        self.last_update = std::time::Instant::now();
    }

    fn get_progress(&self) -> i32 {
        if self.total_count == 0 {
            return 0;
        }
        let current = self.get_current_count();
        ((current as f64 / self.total_count as f64) * 100.0) as i32
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
    /// 3. 预估总文件数并初始化进度
    /// 4. 对比文件是否在 file_contents 表中存在（通过 hash）
    /// 5. 对比 member_files 表是否存在记录
    /// 6. 如果不存在，添加记录到数据表
    /// 7. 定期更新进度到 task_message 表
    async fn handle(&self, payload: &TaskPayload) -> Result<()> {
        let storage_root = Path::new(&self.storage_root);

        info!("Starting file synchronization from: {}", self.storage_root);

        // 设置任务消息 ID
        let task_message_id = payload.task_message_id;

        // 用于追踪总进度的变量
        let mut total_files = 0;
        let _total_dirs = 0;

        // 先估算总文件数（用于进度显示）
        let mut dir_stack = vec![storage_root.to_path_buf()];
        while let Some(current_dir) = dir_stack.pop() {
            if let Ok(mut entries) = fs::read_dir(&current_dir).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        dir_stack.push(entry_path);
                    } else if entry_path.is_file() {
                        let file_name = entry.file_name().to_string_lossy().to_string();
                        let file_name_lower = file_name.to_lowercase();
                        // 跳过隐藏文件和系统文件
                        if !file_name.starts_with('.')
                            && file_name_lower != ".ds_store"
                            && file_name_lower != "thumbs.db"
                        {
                            total_files += 1;
                        }
                    }
                }
            }
        }

        info!("Estimated total files: {}", total_files);

        // 重置并初始化进度追踪器
        let mut handler = SyncFilesHandler {
            storage_root: self.storage_root.clone(),
            conn: self.conn.clone(),
            task_message_id,
            progress_tracker: ProgressTracker {
                processed_count: 0,
                total_count: total_files,
                last_update: std::time::Instant::now(),
                skipped_count: 0,
            },
        };

        // 重新开始遍历
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
                .one(&*handler.conn)
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
                        handler.sync_file(&entry_path, member_id).await?;
                    }
                }
            }
        }

        // 最终更新进度到 100%
        handler.update_progress(100).await?;

        // 更新任务状态为完成
        handler.update_status("completed").await?;

        info!("File synchronization completed");
        Ok(())
    }
}

impl SyncFilesHandler {
    /// 更新进度到数据库
    async fn update_progress(&self, progress: i32) -> Result<()> {
        if let Some(task_message_id) = self.task_message_id {
            let active_model = store::entity::task_messages::ActiveModel {
                id: sea_orm::Set(task_message_id),
                progress: sea_orm::Set(progress.clamp(0, 100)),
                status: sea_orm::Set("processing".to_string()),
                updated_at: sea_orm::Set(chrono::Utc::now()),
                ..Default::default()
            };

            store::entity::task_messages::Entity::update(active_model)
                .exec(&*self.conn)
                .await?;

            debug!("Updated progress to {}%", progress);
        }
        Ok(())
    }

    /// 更新任务状态到数据库
    async fn update_status(&self, status: &str) -> Result<()> {
        if let Some(task_message_id) = self.task_message_id {
            let active_model = store::entity::task_messages::ActiveModel {
                id: sea_orm::Set(task_message_id),
                status: sea_orm::Set(status.to_string()),
                progress: sea_orm::Set(100),
                updated_at: sea_orm::Set(chrono::Utc::now()),
                completed_at: sea_orm::Set(Some(chrono::Utc::now())),
                ..Default::default()
            };

            store::entity::task_messages::Entity::update(active_model)
                .exec(&*self.conn)
                .await?;

            debug!("Updated status to {}", status);
        }
        Ok(())
    }

    /// 同步单个文件
    async fn sync_file(&mut self, file_path: &Path, member_id: i64) -> Result<()> {
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
            self.progress_tracker.increment_skipped();
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

        // 更新进度追踪
        self.progress_tracker.increment_processed();

        // 节流更新进度
        if self.progress_tracker.should_update() {
            let progress = self.progress_tracker.get_progress();
            self.update_progress(progress).await?;
            self.progress_tracker.reset_update_time();
        }

        info!("Synced file: {} (hash: {})", file_name, content_hash);
        Ok(())
    }
}
