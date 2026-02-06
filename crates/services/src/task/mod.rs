//! 任务模块
//!
//! 提供通用的任务队列框架，支持多种任务类型。
//! 使用 tokio mpsc channel 实现异步任务处理。

use crate::Result;
use crate::ServiceError;
use chrono::Utc;
use sea_orm::{QueryOrder, QuerySelect, entity::prelude::*};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use store::entity::task_messages::{
    ActiveModel, Column, Entity as SyncMessages, Model, TaskStatus as DbSyncStatus,
};
use tracing::{debug, error, info};

pub(crate) mod sync_file;

pub use sync_file::SyncFilesHandler;

// SyncDatabaseHandler 待实现
#[derive(Clone)]
pub struct SyncDatabaseHandler;

impl SyncDatabaseHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl TaskHandler for SyncDatabaseHandler {
    fn task_type(&self) -> &'static str {
        "sync_database"
    }

    async fn handle(&self, _payload: &TaskPayload) -> Result<()> {
        // TODO: 实现从数据库同步到目录的逻辑
        Ok(())
    }
}

/// 任务类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    /// 同步目录到数据库
    SyncDirectory,
    /// 从数据库同步到目录
    SyncDatabase,
    /// 清理孤立文件
    CleanupOrphanedFiles,
    /// 其他任务
    Other(String),
}

impl TaskType {
    pub fn as_str(&self) -> &str {
        match self {
            TaskType::SyncDirectory => "sync_directory",
            TaskType::SyncDatabase => "sync_database",
            TaskType::CleanupOrphanedFiles => "cleanup_orphaned_files",
            TaskType::Other(s) => s.as_str(),
        }
    }
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<String> for TaskType {
    type Error = String;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        match value.as_str() {
            "sync_directory" => Ok(TaskType::SyncDirectory),
            "sync_database" => Ok(TaskType::SyncDatabase),
            "cleanup_orphaned_files" => Ok(TaskType::CleanupOrphanedFiles),
            _ => Ok(TaskType::Other(value)),
        }
    }
}

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl TaskStatus {
    pub fn as_str(&self) -> &str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::Processing => "processing",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
        }
    }
}

/// 任务负载
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPayload {
    pub task_type: String,
    pub member_id: i64,
    pub path: String,
    pub options: Option<TaskOptions>,
    /// 任务消息 ID（用于进度更新）
    pub task_message_id: Option<i64>,
}

/// 任务选项
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskOptions {
    /// 是否递归处理子目录
    pub recursive: Option<bool>,
    /// 文件类型过滤
    pub file_types: Option<Vec<String>>,
    /// 是否包含隐藏文件
    pub include_hidden: Option<bool>,
}

/// 任务消息（用于 channel）
#[derive(Debug)]
pub enum TaskMessage {
    /// 新建任务
    NewTask(TaskPayload),
    /// 停止信号
    Shutdown,
}

/// 任务处理器 trait
///
/// 处理器在创建时需要传入数据库连接（Arc 包装），这样 handle 方法就不需要每次都传入连接。
#[async_trait::async_trait]
pub trait TaskHandler: Send + Sync {
    /// 获取任务类型
    fn task_type(&self) -> &'static str;

    /// 处理任务
    /// 注意：handler 内部已经持有数据库连接，无需外部传入
    async fn handle(&self, payload: &TaskPayload) -> Result<()>;
}

/// 任务工作器配置
#[derive(Clone)]
pub struct TaskWorkerConfig {
    /// 处理任务的间隔（毫秒）
    pub poll_interval_ms: u64,
    /// 最大并发处理数
    pub max_concurrent: usize,
}

impl Default for TaskWorkerConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 1000,
            max_concurrent: 5,
        }
    }
}

/// 任务工作器
pub struct TaskWorker {
    /// 数据库连接
    conn: Arc<sea_orm::DatabaseConnection>,
    /// 配置
    config: TaskWorkerConfig,
    /// 任务处理器映射
    handlers: Vec<Arc<dyn TaskHandler>>,
    /// 消息接收端
    rx: tokio::sync::mpsc::Receiver<TaskMessage>,
    /// 消息发送端
    tx: tokio::sync::mpsc::Sender<TaskMessage>,
}

impl TaskWorker {
    /// 创建新的任务工作器
    pub fn new(
        conn: Arc<sea_orm::DatabaseConnection>,
        config: Option<TaskWorkerConfig>,
        buffer: usize,
    ) -> (Self, tokio::sync::mpsc::Sender<TaskMessage>) {
        let (tx, rx) = tokio::sync::mpsc::channel(buffer);
        let config = config.unwrap_or_default();

        let worker = Self {
            conn,
            config,
            handlers: Vec::new(),
            rx,
            tx: tx.clone(),
        };

        (worker, tx)
    }

    /// 注册任务处理器
    pub fn register_handler(&mut self, handler: Arc<dyn TaskHandler>) {
        self.handlers.push(handler);
    }

    /// 获取消息发送端
    pub fn sender(&self) -> tokio::sync::mpsc::Sender<TaskMessage> {
        self.tx.clone()
    }

    /// 启动任务工作器
    pub async fn start(&mut self) {
        info!(
            "任务工作器已启动，轮询间隔: {}ms, 注册处理器数量: {}",
            self.config.poll_interval_ms,
            self.handlers.len()
        );

        let poll_interval = std::time::Duration::from_millis(self.config.poll_interval_ms);
        let mut interval = tokio::time::interval(poll_interval);

        loop {
            tokio::select! {
                Some(message) = self.rx.recv() => {
                    match message {
                        TaskMessage::NewTask(payload) => {
                            if let Err(e) = self.process_task(&payload).await {
                                error!("处理任务失败: {}, payload={:?}", e, payload);
                            }
                        }
                        TaskMessage::Shutdown => {
                            info!("收到停止信号，任务工作器正在关闭...");
                            break;
                        }
                    }
                }
                _ = interval.tick() => {
                    // 定时检查数据库中的待处理任务
                    if let Err(e) = self.process_pending_db_tasks().await {
                        error!("处理数据库待处理任务失败: {}", e);
                    }
                }
            }
        }
    }

    /// 处理任务
    async fn process_task(&self, payload: &TaskPayload) -> Result<()> {
        debug!(
            "处理任务: type={}, member_id={}, path={}",
            payload.task_type, payload.member_id, payload.path
        );

        // 查找对应的处理器
        let handler = self
            .handlers
            .iter()
            .find(|h| h.task_type() == payload.task_type);

        if let Some(h) = handler {
            h.handle(payload).await?;
        } else {
            info!("未找到任务类型 {} 的处理器，跳过", payload.task_type);
        }

        Ok(())
    }

    /// 处理数据库中的待处理任务
    async fn process_pending_db_tasks(&self) -> Result<()> {
        let pending_tasks = SyncMessages::find()
            .filter(
                sea_orm::Condition::all()
                    .add(Column::Status.eq(DbSyncStatus::Pending.as_str().to_string())),
            )
            .order_by_asc(Column::CreatedAt)
            .limit(10)
            .all(&*self.conn)
            .await?;

        if pending_tasks.is_empty() {
            return Ok(());
        }

        info!("发现 {} 条待处理的任务", pending_tasks.len());

        for task in pending_tasks {
            // 更新状态为 processing
            if let Err(e) = self
                .update_task_status(&task, DbSyncStatus::Processing)
                .await
            {
                error!("更新任务状态失败: {}", e);
                continue;
            }

            // 解析任务负载并设置任务消息 ID
            let payload: Result<TaskPayload> =
                serde_json::from_str(&task.payload).map_err(|e| ServiceError::Other(e.to_string()));

            let payload = match payload {
                Ok(mut p) => {
                    p.task_message_id = Some(task.id);
                    Ok(p)
                }
                Err(e) => Err(e),
            };

            match payload {
                Ok(p) => match self.process_task(&p).await {
                    Ok(()) => {
                        if let Err(e) = self
                            .update_task_status(&task, DbSyncStatus::Completed)
                            .await
                        {
                            error!("更新任务完成状态失败: {}", e);
                        } else {
                            info!("任务 #{} 处理完成", task.id);
                        }
                    }
                    Err(e) => {
                        self.mark_task_failed(&task, &e.to_string()).await?;
                        error!("任务 #{} 处理失败: {}", task.id, e);
                    }
                },
                Err(e) => {
                    self.mark_task_failed(&task, &e.to_string()).await?;
                }
            }
        }

        Ok(())
    }

    /// 更新任务状态
    async fn update_task_status(&self, task: &Model, status: DbSyncStatus) -> Result<()> {
        let active_model = ActiveModel {
            id: sea_orm::Set(task.id),
            status: sea_orm::Set(status.as_str().to_string()),
            updated_at: sea_orm::Set(Utc::now()),
            ..Default::default()
        };

        SyncMessages::update(active_model).exec(&*self.conn).await?;
        Ok(())
    }

    /// 标记任务失败
    async fn mark_task_failed(&self, task: &Model, error_message: &str) -> Result<()> {
        let active_model = ActiveModel {
            id: sea_orm::Set(task.id),
            status: sea_orm::Set(DbSyncStatus::Failed.as_str().to_string()),
            error_message: sea_orm::Set(Some(error_message.to_string())),
            updated_at: sea_orm::Set(Utc::now()),
            completed_at: sea_orm::Set(Some(Utc::now())),
            ..Default::default()
        };

        SyncMessages::update(active_model).exec(&*self.conn).await?;
        Ok(())
    }
}

/// 任务发送器
#[derive(Clone)]
pub struct TaskSender {
    tx: tokio::sync::mpsc::Sender<TaskMessage>,
}

impl TaskSender {
    /// 创建新的发送器
    pub fn new(tx: tokio::sync::mpsc::Sender<TaskMessage>) -> Self {
        Self { tx }
    }

    /// 发送任务
    pub async fn send(&self, payload: TaskPayload) -> Result<()> {
        self.tx
            .send(TaskMessage::NewTask(payload))
            .await
            .map_err(|e| ServiceError::Other(e.to_string()))
    }

    /// 发送停止信号
    pub async fn shutdown(&self) -> Result<()> {
        self.tx
            .send(TaskMessage::Shutdown)
            .await
            .map_err(|e| ServiceError::Other(e.to_string()))
    }
}

/// 创建任务 channel
pub fn create_task_channel(
    buffer: usize,
) -> (TaskSender, tokio::sync::mpsc::Receiver<TaskMessage>) {
    let (tx, rx) = tokio::sync::mpsc::channel(buffer);
    let sender = TaskSender::new(tx);
    (sender, rx)
}

/// 任务发送器类型别名
pub type TaskSenderType = Arc<TaskSender>;
