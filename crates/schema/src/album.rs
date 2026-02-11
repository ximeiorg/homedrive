use serde::{Deserialize, Serialize};

/// 创建相册请求
#[derive(Debug, Deserialize)]
pub struct CreateAlbumRequest {
    /// 相册名称
    pub name: String,
    /// 相册描述
    pub description: Option<String>,
    /// 封面图片 ID (member_file_id)
    pub cover_file_id: Option<i64>,
    /// 初始添加的文件 ID 列表
    pub file_ids: Option<Vec<i64>>,
}

/// 更新相册请求
#[derive(Debug, Deserialize)]
pub struct UpdateAlbumRequest {
    /// 相册名称
    pub name: Option<String>,
    /// 相册描述
    pub description: Option<String>,
    /// 封面图片 ID
    pub cover_file_id: Option<i64>,
}

/// 添加文件到相册请求
#[derive(Debug, Deserialize)]
pub struct AddFilesRequest {
    /// 文件 ID 列表
    pub file_ids: Vec<i64>,
}

/// 从相册移除文件请求
#[derive(Debug, Deserialize)]
pub struct RemoveFilesRequest {
    /// 文件 ID 列表
    pub file_ids: Vec<i64>,
}

/// 相册响应
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

/// 相册列表响应
#[derive(Debug, Serialize)]
pub struct AlbumListResponse {
    pub albums: Vec<AlbumListItem>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

/// 文件信息（用于相册文件列表）
#[derive(Debug, Serialize)]
pub struct AlbumFileInfo {
    pub id: i64,
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: String,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 相册文件列表响应
#[derive(Debug, Serialize)]
pub struct AlbumFilesResponse {
    pub files: Vec<AlbumFileInfo>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

/// 添加文件响应
#[derive(Debug, Serialize)]
pub struct AddFilesResponse {
    pub added_count: u64,
}

/// 移除文件响应
#[derive(Debug, Serialize)]
pub struct RemoveFilesResponse {
    pub removed_count: u64,
}

/// 通用消息响应
#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

/// 分页查询参数
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    /// 页码，从 1 开始
    pub page: Option<u64>,
    /// 每页大小
    pub page_size: Option<u64>,
}
