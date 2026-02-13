use crate::extract::{ValidatedJson, ValidatedQuery};
use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
};
use schema::member::{
    CreateMemberRequest, InitAdminRequest, InitAdminResponse, ListMembersQuery, LoginRequest,
    LoginResponse, MemberListResponse, MemberResponse, UpdateAvatarRequest, UpdateMemberRequest,
    UpdatePasswordRequest, validate_username_format, validate_storage_tag_format,
};
use std::sync::Arc;

/// 创建新成员
pub async fn create_member(
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<CreateMemberRequest>,
) -> crate::error::Result<Json<MemberResponse>> {
    // 验证用户名格式
    if !validate_username_format(&payload.username) {
        return Err(crate::error::AppError::ValidationError(
            "用户名只能包含字母、数字和下划线".to_string()
        ));
    }
    
    // 验证存储标签格式
    if !validate_storage_tag_format(&payload.storage_tag) {
        return Err(crate::error::AppError::ValidationError(
            "存储标签只能包含字母、数字、下划线和连字符".to_string()
        ));
    }
    
    let member = services::MemberService::create_member(&state.conn, payload).await?;
    Ok(Json(member))
}

/// 获取成员详情
pub async fn get_member(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> crate::error::Result<Json<MemberResponse>> {
    match services::MemberService::get_member(&state.conn, id).await? {
        Some(member) => Ok(Json(member)),
        None => {
            tracing::warn!("Member not found: {}", id);
            Err(crate::error::AppError::MemberNotFound)
        }
    }
}

/// 更新成员信息
pub async fn update_member(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    ValidatedJson(payload): ValidatedJson<UpdateMemberRequest>,
) -> crate::error::Result<Json<MemberResponse>> {
    // 验证用户名格式（如果提供）
    if let Some(ref username) = payload.username {
        if !validate_username_format(username) {
            return Err(crate::error::AppError::ValidationError(
                "用户名只能包含字母、数字和下划线".to_string()
            ));
        }
    }
    
    // 验证存储标签格式（如果提供）
    if let Some(ref storage_tag) = payload.storage_tag {
        if !validate_storage_tag_format(storage_tag) {
            return Err(crate::error::AppError::ValidationError(
                "存储标签只能包含字母、数字、下划线和连字符".to_string()
            ));
        }
    }
    
    let member = services::MemberService::update_member(&state.conn, id, payload).await?;
    Ok(Json(member))
}

/// 删除成员
pub async fn delete_member(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> crate::error::Result<Json<()>> {
    services::MemberService::delete_member(&state.conn, id).await?;
    Ok(Json(()))
}

/// 获取成员列表
pub async fn list_members(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(query): ValidatedQuery<ListMembersQuery>,
) -> crate::error::Result<Json<MemberListResponse>> {
    let members =
        services::MemberService::list_members(&state.conn, query.page, query.page_size).await?;
    Ok(Json(members))
}

/// 根据用户名查询成员
pub async fn get_member_by_username(
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>,
) -> crate::error::Result<Json<MemberResponse>> {
    // 验证用户名长度
    if username.is_empty() || username.len() > 50 {
        return Err(crate::error::AppError::InvalidInput("Invalid username".to_string()));
    }
    
    match services::MemberService::get_member_by_username(&state.conn, &username).await? {
        Some(member) => Ok(Json(member)),
        None => {
            tracing::warn!("Member not found by username");
            Err(crate::error::AppError::MemberNotFound)
        }
    }
}

/// 检查用户名是否存在
pub async fn check_username_exists(
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>,
) -> crate::error::Result<Json<bool>> {
    // 验证用户名长度
    if username.is_empty() || username.len() > 50 {
        return Err(crate::error::AppError::InvalidInput("Invalid username".to_string()));
    }
    
    let exists = services::MemberService::username_exists(&state.conn, &username).await?;
    Ok(Json(exists))
}

/// 更新成员头像
pub async fn update_member_avatar(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    ValidatedJson(payload): ValidatedJson<UpdateAvatarRequest>,
) -> crate::error::Result<Json<MemberResponse>> {
    let member = services::MemberService::update_avatar(&state.conn, id, payload.avatar).await?;
    Ok(Json(member))
}

/// 更新成员密码
pub async fn update_member_password(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    ValidatedJson(payload): ValidatedJson<UpdatePasswordRequest>,
) -> crate::error::Result<Json<MemberResponse>> {
    let member =
        services::MemberService::update_password(&state.conn, id, payload.new_password).await?;
    Ok(Json(member))
}

/// 登录
pub async fn login(
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<LoginRequest>,
) -> crate::error::Result<Json<LoginResponse>> {
    let login_response = services::MemberService::login(&state.conn, payload).await?;
    Ok(Json(login_response))
}

/// 检查 member 表是否为空（无需认证）
pub async fn check_members_empty(
    State(state): State<Arc<AppState>>,
) -> crate::error::Result<Json<schema::member::IsEmptyResponse>> {
    let response = services::MemberService::is_empty(&state.conn).await?;
    Ok(Json(response))
}

/// 初始化管理员（无需认证，仅当 member 表为空时有效）
pub async fn init_admin(
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<InitAdminRequest>,
) -> crate::error::Result<Json<InitAdminResponse>> {
    // 验证用户名格式
    if !validate_username_format(&payload.username) {
        return Err(crate::error::AppError::ValidationError(
            "用户名只能包含字母、数字和下划线".to_string()
        ));
    }
    
    // 验证存储标签格式
    if !validate_storage_tag_format(&payload.storage_tag) {
        return Err(crate::error::AppError::ValidationError(
            "存储标签只能包含字母、数字、下划线和连字符".to_string()
        ));
    }
    
    let response = services::MemberService::init_admin(&state.conn, payload).await?;
    Ok(Json(response))
}
