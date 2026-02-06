use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, Query, State},
};
use schema::member::{
    CreateMemberRequest, InitAdminRequest, InitAdminResponse, ListMembersQuery, LoginRequest,
    LoginResponse, MemberListResponse, MemberResponse, UpdateAvatarRequest, UpdateMemberRequest,
    UpdatePasswordRequest,
};
use std::sync::Arc;

/// 创建新成员
pub async fn create_member(
    State(state): State<Arc<AppState>>,
    axum::Json(payload): axum::Json<CreateMemberRequest>,
) -> crate::error::Result<Json<MemberResponse>> {
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
            tracing::error!("Member not found: {}", id);
            Err(crate::error::AppError::MemberNotFound)
        }
    }
}

/// 更新成员信息
pub async fn update_member(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    axum::Json(payload): axum::Json<UpdateMemberRequest>,
) -> crate::error::Result<Json<MemberResponse>> {
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
    Query(query): Query<ListMembersQuery>,
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
    match services::MemberService::get_member_by_username(&state.conn, &username).await? {
        Some(member) => Ok(Json(member)),
        None => {
            tracing::error!("Member not found by username: {}", username);
            Err(crate::error::AppError::MemberNotFound)
        }
    }
}

/// 检查用户名是否存在
pub async fn check_username_exists(
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>,
) -> crate::error::Result<Json<bool>> {
    let exists = services::MemberService::username_exists(&state.conn, &username).await?;
    Ok(Json(exists))
}

/// 更新成员头像
pub async fn update_member_avatar(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    axum::Json(payload): axum::Json<UpdateAvatarRequest>,
) -> crate::error::Result<Json<MemberResponse>> {
    let member = services::MemberService::update_avatar(&state.conn, id, payload.avatar).await?;
    Ok(Json(member))
}

/// 更新成员密码
pub async fn update_member_password(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    axum::Json(payload): axum::Json<UpdatePasswordRequest>,
) -> crate::error::Result<Json<MemberResponse>> {
    let member =
        services::MemberService::update_password(&state.conn, id, payload.new_password).await?;
    Ok(Json(member))
}

/// 登录
pub async fn login(
    State(state): State<Arc<AppState>>,
    axum::Json(payload): axum::Json<LoginRequest>,
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
    axum::Json(payload): axum::Json<InitAdminRequest>,
) -> crate::error::Result<Json<InitAdminResponse>> {
    let response = services::MemberService::init_admin(&state.conn, payload).await?;
    Ok(Json(response))
}
