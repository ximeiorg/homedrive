use serde::{Deserialize, Serialize};
use validator::Validate;

/// 创建成员请求
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateMemberRequest {
    /// 用户名，3-50个字符，只允许字母、数字、下划线
    #[validate(length(min = 3, max = 50, message = "用户名长度必须在3-50个字符之间"))]
    pub username: String,
    
    /// 密码，至少6个字符
    #[validate(length(min = 6, max = 128, message = "密码长度必须在6-128个字符之间"))]
    pub password: String,
    
    /// 头像URL（可选）
    #[validate(url(message = "头像必须是有效的URL格式"))]
    pub avatar: Option<String>,
    
    /// 存储标签，1-50个字符
    #[validate(length(min = 1, max = 50, message = "存储标签长度必须在1-50个字符之间"))]
    pub storage_tag: String,
}

/// 更新成员请求
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateMemberRequest {
    /// 用户名（可选），3-50个字符
    #[validate(length(min = 3, max = 50, message = "用户名长度必须在3-50个字符之间"))]
    pub username: Option<String>,
    
    /// 密码（可选），至少6个字符
    #[validate(length(min = 6, max = 128, message = "密码长度必须在6-128个字符之间"))]
    pub password: Option<String>,
    
    /// 头像URL（可选）
    #[validate(url(message = "头像必须是有效的URL格式"))]
    pub avatar: Option<String>,
    
    /// 存储标签（可选），1-50个字符
    #[validate(length(min = 1, max = 50, message = "存储标签长度必须在1-50个字符之间"))]
    pub storage_tag: Option<String>,
}

/// 成员列表查询参数
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ListMembersQuery {
    /// 页码，最小为1
    #[validate(range(min = 1, message = "页码必须大于0"))]
    pub page: Option<u64>,
    
    /// 每页大小，1-100
    #[validate(range(min = 1, max = 100, message = "每页大小必须在1-100之间"))]
    pub page_size: Option<u64>,
}

/// 更新头像请求
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateAvatarRequest {
    /// 头像URL
    #[validate(url(message = "头像必须是有效的URL格式"))]
    pub avatar: String,
}

/// 更新密码请求
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdatePasswordRequest {
    /// 新密码，至少6个字符
    #[validate(length(min = 6, max = 128, message = "密码长度必须在6-128个字符之间"))]
    pub new_password: String,
}

/// 成员响应
#[derive(Debug, Serialize, Deserialize)]
pub struct MemberResponse {
    pub id: i64,
    pub username: String,
    pub avatar: Option<String>,
    pub storage_tag: String,
    pub storage_used: i64,  // 已使用存储（字节）
    pub storage_total: i64, // 总存储空间（字节）
    pub last_active: Option<chrono::DateTime<chrono::Utc>>, // 最后活跃时间
    pub status: String,     // online, offline, away
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 成员列表响应
#[derive(Debug, Serialize, Deserialize)]
pub struct MemberListResponse {
    pub members: Vec<MemberResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

/// 登录请求
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    /// 用户名
    #[validate(length(min = 1, max = 50, message = "用户名不能为空且不能超过50个字符"))]
    pub username: String,
    
    /// 密码
    #[validate(length(min = 1, max = 128, message = "密码不能为空"))]
    pub password: String,
}

/// 登录响应
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub member: MemberResponse,
}

/// 刷新Token请求
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RefreshTokenRequest {
    #[validate(length(min = 1, message = "刷新令牌不能为空"))]
    pub refresh_token: String,
}

/// 检查 member 表是否为空响应
#[derive(Debug, Serialize, Deserialize)]
pub struct IsEmptyResponse {
    pub is_empty: bool,
}

/// 初始化管理员请求
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct InitAdminRequest {
    /// 用户名，3-50个字符
    #[validate(length(min = 3, max = 50, message = "用户名长度必须在3-50个字符之间"))]
    pub username: String,
    
    /// 密码，至少6个字符
    #[validate(length(min = 6, max = 128, message = "密码长度必须在6-128个字符之间"))]
    pub password: String,
    
    /// 存储标签，1-50个字符
    #[validate(length(min = 1, max = 50, message = "存储标签长度必须在1-50个字符之间"))]
    pub storage_tag: String,
}

/// 初始化管理员响应
#[derive(Debug, Serialize, Deserialize)]
pub struct InitAdminResponse {
    pub success: bool,
    pub message: String,
    pub member: Option<MemberResponse>,
}

/// 验证用户名格式：只允许字母、数字、下划线
pub fn validate_username_format(username: &str) -> bool {
    if username.is_empty() {
        return false;
    }
    username.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// 验证存储标签格式：只允许字母、数字、下划线和连字符
pub fn validate_storage_tag_format(tag: &str) -> bool {
    if tag.is_empty() {
        return false;
    }
    tag.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}
