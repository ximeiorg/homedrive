//! 文件相关的数据结构

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct HashCheckResponse {
    pub exists: bool,
}

#[derive(Deserialize)]
pub struct HashCheckQuery {
    pub hash: String,
}

#[derive(Deserialize)]
pub struct UploadFileRequest {
    pub hash: String,
}

#[derive(Serialize)]
pub struct UploadFileResponse {
    pub success: bool,
    pub file_id: i64,
    pub message: String,
}

#[derive(Serialize)]
pub struct FileListItem {
    pub id: i64,
    pub file_name: String,
    pub description: String,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub thumbnail: Option<String>, // 缩略图地址
    pub url: Option<String>,       // 文件访问地址
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct FileListResponse {
    pub files: Vec<FileListItem>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
}

#[derive(Deserialize)]
pub struct ListFilesQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub file_type: Option<String>,
    pub search: Option<String>,
}

#[derive(Deserialize)]
pub struct TriggerSyncRequest {
    pub path: Option<String>,
    pub task_type: Option<String>,
}

#[derive(Serialize)]
pub struct TriggerSyncResponse {
    pub success: bool,
    pub message: String,
}

/// 任务列表项
#[derive(Serialize)]
pub struct TaskItemResponse {
    pub id: i64,
    pub task_type: String,
    pub status: String,
    pub progress: i32,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// 任务列表响应
#[derive(Serialize)]
pub struct TaskListResponse {
    pub tasks: Vec<TaskItemResponse>,
}

/// 同步文件请求
#[derive(Deserialize)]
pub struct SyncFilesRequest {
    pub path: Option<String>,
}

/// 同步文件响应
#[derive(Serialize)]
pub struct SyncFilesResponse {
    pub success: bool,
    pub task_id: i64,
    pub message: String,
}
