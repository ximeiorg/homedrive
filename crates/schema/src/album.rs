use serde::{Deserialize, Serialize};
use validator::Validate;

/// 创建相册请求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateAlbumRequest {
    /// 相册名称，1-100个字符
    #[validate(length(min = 1, max = 100, message = "相册名称长度必须在1-100个字符之间"))]
    pub name: String,

    /// 相册描述（可选），最多500个字符
    #[validate(length(max = 500, message = "相册描述长度不能超过500个字符"))]
    pub description: Option<String>,

    /// 封面图片 ID (member_file_id)
    pub cover_file_id: Option<i64>,

    /// 初始添加的文件 ID 列表，最多1000个
    #[validate(length(max = 1000, message = "单次最多添加1000个文件"))]
    pub file_ids: Option<Vec<i64>>,
}

/// 更新相册请求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateAlbumRequest {
    /// 相册名称（可选），1-100个字符
    #[validate(length(min = 1, max = 100, message = "相册名称长度必须在1-100个字符之间"))]
    pub name: Option<String>,

    /// 相册描述（可选），最多500个字符
    #[validate(length(max = 500, message = "相册描述长度不能超过500个字符"))]
    pub description: Option<String>,

    /// 封面图片 ID
    pub cover_file_id: Option<i64>,
}

/// 添加文件到相册请求
#[derive(Debug, Deserialize, Validate)]
pub struct AddFilesRequest {
    /// 文件 ID 列表，最多1000个
    #[validate(length(min = 1, max = 1000, message = "文件ID列表长度必须在1-1000之间"))]
    pub file_ids: Vec<i64>,
}

/// 从相册移除文件请求
#[derive(Debug, Deserialize, Validate)]
pub struct RemoveFilesRequest {
    /// 文件 ID 列表，最多1000个
    #[validate(length(min = 1, max = 1000, message = "文件ID列表长度必须在1-1000之间"))]
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
    pub cover_url: Option<String>, // 封面图片 URL
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
    pub thumbnail: Option<String>, // 缩略图 URL
    pub url: Option<String>,       // 原图 URL
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
#[derive(Debug, Deserialize, Validate)]
pub struct PaginationQuery {
    /// 页码，从 1 开始
    #[validate(range(min = 1, message = "页码必须大于0"))]
    pub page: Option<u64>,

    /// 每页大小，1-100
    #[validate(range(min = 1, max = 100, message = "每页大小必须在1-100之间"))]
    pub page_size: Option<u64>,
}
