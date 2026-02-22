//! 视频缩略图生成任务处理器
//!
//! 扫描指定目录下的视频文件，为每个视频生成缩略图

use crate::{Result, TaskHandler, TaskPayload};
use sea_orm::{ColumnTrait as _, DatabaseConnection, EntityTrait, QueryFilter};
use std::path::Path;
use std::sync::Arc;
use thumbnail::{ThumbnailConfig, generate_thumbnail};

/// 进度更新节流配置
const PROGRESS_UPDATE_INTERVAL: usize = 5; // 每处理 5 个文件更新一次进度

/// 视频缩略图生成任务处理器
pub struct GenerateThumbnailHandler {
    /// 存储根目录
    storage_root: String,
    /// 数据库连接（Arc 包装以支持 Clone）
    conn: Arc<DatabaseConnection>,
    /// 任务消息 ID（用于进度更新）
    task_message_id: Option<i64>,
    /// 已处理文件计数
    processed_count: usize,
    /// 总文件数（用于计算进度）
    total_count: usize,
}

impl GenerateThumbnailHandler {
    pub fn new(storage_root: impl Into<String>, conn: Arc<DatabaseConnection>) -> Self {
        Self {
            storage_root: storage_root.into(),
            conn,
            task_message_id: None,
            processed_count: 0,
            total_count: 0,
        }
    }

    /// 更新进度到数据库
    async fn update_progress(&self, progress: i64) -> Result<()> {
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

            tracing::debug!("Updated progress to {}%", progress);
        }
        Ok(())
    }

    /// 更新任务状态到数据库
    async fn update_status(&self, status: &str) -> Result<()> {
        if let Some(task_message_id) = self.task_message_id {
            let progress = if status == "completed" { 100 } else { 0 };
            let active_model = store::entity::task_messages::ActiveModel {
                id: sea_orm::Set(task_message_id),
                status: sea_orm::Set(status.to_string()),
                progress: sea_orm::Set(progress),
                updated_at: sea_orm::Set(chrono::Utc::now()),
                completed_at: if status == "completed" {
                    sea_orm::Set(Some(chrono::Utc::now()))
                } else {
                    sea_orm::Set(None)
                },
                ..Default::default()
            };

            store::entity::task_messages::Entity::update(active_model)
                .exec(&*self.conn)
                .await?;

            tracing::debug!("Updated status to {}", status);
        }
        Ok(())
    }

    /// 更新文件的缩略图路径到数据库
    async fn update_file_thumbnail(
        &self,
        file_content_id: i64,
        thumbnail_path: &str,
    ) -> Result<()> {
        let active_model = store::entity::file_contents::ActiveModel {
            id: sea_orm::Set(file_content_id),
            thumbnail: sea_orm::Set(Some(thumbnail_path.to_string())),
            ..Default::default()
        };

        store::entity::file_contents::Entity::update(active_model)
            .exec(&*self.conn)
            .await?;

        tracing::debug!(
            "Updated thumbnail path for file_content {}: {}",
            file_content_id,
            thumbnail_path
        );
        Ok(())
    }
}

#[async_trait::async_trait]
impl TaskHandler for GenerateThumbnailHandler {
    fn task_type(&self) -> &'static str {
        "generate_thumbnail"
    }

    async fn handle(&self, payload: &TaskPayload) -> Result<()> {
        tracing::info!(
            "开始生成视频缩略图, 路径: {}, 成员ID: {}",
            payload.path,
            payload.member_id
        );

        // 设置任务消息 ID
        let task_message_id = payload.task_message_id;

        // 视频文件扩展名
        let video_extensions = ["mp4", "avi", "mov", "mkv", "webm", "flv", "wmv"];

        // 获取该成员下的所有文件（包含视频）
        let files = store::entity::member_files::Entity::find()
            .filter(store::entity::member_files::Column::MemberId.eq(payload.member_id))
            .find_with_related(store::entity::file_contents::Entity)
            .all(&*self.conn)
            .await?;

        // 统计需要处理的视频文件总数
        let mut total_videos = 0;
        for (member_file, file_contents) in &files {
            let ext = Path::new(&member_file.file_name)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            if !video_extensions.contains(&ext.as_str()) {
                continue;
            }

            if let Some(content) = file_contents.first() {
                if content.thumbnail.is_none() {
                    total_videos += 1;
                }
            }
        }

        tracing::info!("需要生成缩略图的视频文件总数: {}", total_videos);

        // 创建带有 task_message_id 的处理器实例
        let mut handler = GenerateThumbnailHandler {
            storage_root: self.storage_root.clone(),
            conn: self.conn.clone(),
            task_message_id,
            processed_count: 0,
            total_count: total_videos,
        };

        // 更新状态为处理中
        handler.update_status("processing").await?;

        let mut success_count = 0;
        let mut fail_count = 0;
        let mut skip_count = 0;

        for (member_file, file_contents) in files {
            // 获取文件扩展名
            let ext = Path::new(&member_file.file_name)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            // 只处理视频文件
            if !video_extensions.contains(&ext.as_str()) {
                continue;
            }

            // 获取文件内容（可能为空）
            let Some(content) = file_contents.first() else {
                tracing::warn!("文件 {} 没有关联的内容记录", member_file.file_name);
                skip_count += 1;
                continue;
            };

            // 检查是否已有缩略图
            if content.thumbnail.is_some() {
                tracing::debug!("文件 {} 已有缩略图，跳过", member_file.file_name);
                skip_count += 1;
                continue;
            }

            // 视频文件的完整路径
            let video_path = Path::new(&handler.storage_root).join(&content.storage_path);

            // 获取视频文件所在的目录和文件名
            let video_dir = video_path.parent().unwrap_or(Path::new(""));
            let video_file_stem = Path::new(&member_file.file_name)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&member_file.file_name);

            // 创建 .thumbnail 子目录（与视频文件在同一目录）
            let thumbnail_dir = video_dir.join(".thumbnail");

            // 缩略图文件名与视频名称一致（加上 .jpg 后缀）
            let thumbnail_filename = format!("{video_file_stem}.jpg");
            let thumbnail_path = thumbnail_dir.join(&thumbnail_filename);

            // 确保 .thumbnail 目录存在
            if !thumbnail_dir.exists() {
                if let Err(e) = std::fs::create_dir_all(&thumbnail_dir) {
                    tracing::error!("创建缩略图目录失败: {} - {}", thumbnail_dir.display(), e);
                    fail_count += 1;
                    handler.processed_count += 1;
                    continue;
                }
            }

            // 生成缩略图
            match generate_thumbnail(&video_path, &thumbnail_path, ThumbnailConfig::default()).await
            {
                Ok(_) => {
                    tracing::info!("生成缩略图成功: {}", thumbnail_filename);

                    // 计算相对路径（相对于存储根目录）
                    let relative_thumbnail_path = thumbnail_path
                        .strip_prefix(&handler.storage_root)
                        .unwrap_or(&thumbnail_path)
                        .to_string_lossy()
                        .to_string();

                    // 更新数据库中的缩略图路径
                    if let Err(e) = handler
                        .update_file_thumbnail(content.id, &relative_thumbnail_path)
                        .await
                    {
                        tracing::error!("更新缩略图路径失败: {}", e);
                    }

                    success_count += 1;
                }
                Err(e) => {
                    tracing::error!("生成缩略图失败: {} - {}", member_file.file_name, e);
                    fail_count += 1;
                }
            }

            // 更新进度
            handler.processed_count += 1;
            if handler.processed_count % PROGRESS_UPDATE_INTERVAL == 0
                || handler.processed_count == handler.total_count
            {
                let progress = if handler.total_count > 0 {
                    (handler.processed_count as f64 / handler.total_count as f64 * 100.0) as i64
                } else {
                    100
                };
                handler.update_progress(progress).await?;
            }
        }

        // 更新任务状态为完成
        handler.update_status("completed").await?;

        tracing::info!(
            "视频缩略图生成完成, 成功: {}, 失败: {}, 跳过: {}",
            success_count,
            fail_count,
            skip_count
        );

        Ok(())
    }
}
