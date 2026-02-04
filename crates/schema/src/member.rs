use serde::{Deserialize, Serialize};

/// 创建成员请求
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMemberRequest {
    pub username: String,
    pub password: String,
    pub avatar: Option<String>,
    pub storage_tag: String,
}

/// 更新成员请求
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMemberRequest {
    pub username: Option<String>,
    pub password: Option<String>,
    pub avatar: Option<String>,
    pub storage_tag: Option<String>,
}

/// 成员列表查询参数
#[derive(Debug, Serialize, Deserialize)]
pub struct ListMembersQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

/// 更新头像请求
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAvatarRequest {
    pub avatar: String,
}

/// 更新密码请求
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePasswordRequest {
    pub new_password: String,
}

/// 成员响应
#[derive(Debug, Serialize, Deserialize)]
pub struct MemberResponse {
    pub id: i64,
    pub username: String,
    pub avatar: Option<String>,
    pub storage_tag: String,
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
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 登录响应
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub member: MemberResponse,
}

/// 刷新Token请求
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}
