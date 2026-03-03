//! 文件相关的数据结构

use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize)]
pub struct HashCheckResponse {
    pub exists: bool,
}

#[derive(Deserialize, Validate)]
pub struct HashCheckQuery {
    /// 文件哈希值，必须是有效的xxh3哈希（32位十六进制）
    #[validate(length(min = 32, max = 128, message = "哈希值长度无效"))]
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
    pub created_at: String,        // 本地时区时间字符串
    pub updated_at: String,        // 本地时区时间字符串
}

/// 将 UTC 时间转换为本地时区字符串
pub fn format_datetime_local(dt: DateTime<Utc>) -> String {
    // 转换为本地时区
    let local_dt = dt.with_timezone(&Local);
    // 格式化为易读的字符串
    local_dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[derive(Serialize)]
pub struct FileListResponse {
    pub files: Vec<FileListItem>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
}

#[derive(Deserialize, Validate)]
pub struct ListFilesQuery {
    /// 文件路径（可选）
    #[validate(length(max = 500, message = "路径长度不能超过500个字符"))]
    pub path: Option<String>,

    /// 页码，最小为1
    #[validate(range(min = 1, message = "页码必须大于0"))]
    pub page: Option<u64>,

    /// 每页大小，1-100
    #[validate(range(min = 1, max = 100, message = "每页大小必须在1-100之间"))]
    pub page_size: Option<u64>,

    /// 排序字段
    #[validate(length(max = 50, message = "排序字段长度不能超过50个字符"))]
    pub sort_by: Option<String>,

    /// 排序方向
    #[validate(length(max = 10, message = "排序方向长度无效"))]
    pub sort_order: Option<String>,

    /// 文件类型过滤
    #[validate(length(max = 50, message = "文件类型长度不能超过50个字符"))]
    pub file_type: Option<String>,

    /// 搜索关键词
    #[validate(length(max = 200, message = "搜索关键词长度不能超过200个字符"))]
    pub search: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct TriggerSyncRequest {
    /// 同步路径（可选）
    #[validate(length(max = 500, message = "路径长度不能超过500个字符"))]
    pub path: Option<String>,

    /// 任务类型（可选）
    #[validate(length(max = 50, message = "任务类型长度不能超过50个字符"))]
    pub task_type: Option<String>,
}

#[derive(Serialize)]
pub struct TriggerSyncResponse {
    pub success: bool,
    pub task_id: i64,
    pub message: String,
}

/// 任务列表项
#[derive(Serialize)]
pub struct TaskItemResponse {
    pub id: i64,
    pub task_type: String,
    pub status: String,
    pub progress: i64,
    pub message: String,
    pub created_at: String,           // 本地时区时间字符串
    pub updated_at: String,           // 本地时区时间字符串
    pub completed_at: Option<String>, // 本地时区时间字符串
}

/// 任务列表响应
#[derive(Serialize)]
pub struct TaskListResponse {
    pub tasks: Vec<TaskItemResponse>,
}

/// 同步文件请求
#[derive(Deserialize, Validate)]
pub struct SyncFilesRequest {
    /// 同步路径（可选）
    #[validate(length(max = 500, message = "路径长度不能超过500个字符"))]
    pub path: Option<String>,
}

/// 同步文件响应
#[derive(Serialize)]
pub struct SyncFilesResponse {
    pub success: bool,
    pub task_id: i64,
    pub message: String,
}

/// 验证排序方向是否有效
pub fn is_valid_sort_order(order: &str) -> bool {
    order == "asc" || order == "desc"
}

// ==================== 回收站相关数据结构 ====================

/// 批量删除请求（移动到回收站）
#[derive(Deserialize, Validate)]
pub struct DeleteFilesRequest {
    /// 要删除的文件 ID 列表
    #[validate(length(min = 1, max = 100, message = "文件ID数量必须在1-100之间"))]
    pub file_ids: Vec<i64>,
}

/// 批量删除响应
#[derive(Serialize)]
pub struct DeleteFilesResponse {
    pub success: bool,
    pub deleted_count: u64,
    pub message: String,
}

/// 回收站文件列表项
#[derive(Serialize)]
pub struct TrashListItem {
    pub id: i64,
    pub file_name: String,
    pub description: String,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub thumbnail: Option<String>,
    pub url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: String, // 删除时间
}

/// 回收站文件列表响应
#[derive(Serialize)]
pub struct TrashListResponse {
    pub files: Vec<TrashListItem>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
}

/// 批量恢复请求
#[derive(Deserialize, Validate)]
pub struct RestoreFilesRequest {
    /// 要恢复的文件 ID 列表
    #[validate(length(min = 1, max = 100, message = "文件ID数量必须在1-100之间"))]
    pub file_ids: Vec<i64>,
}

/// 批量恢复响应
#[derive(Serialize)]
pub struct RestoreFilesResponse {
    pub success: bool,
    pub restored_count: u64,
    pub message: String,
}

/// 清空回收站响应
#[derive(Serialize)]
pub struct EmptyTrashResponse {
    pub success: bool,
    pub deleted_count: u64,
    pub message: String,
}
