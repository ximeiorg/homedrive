//! 成员模块路由（受保护）
//!
//! 定义所有需要认证的成员相关路由

use crate::{
    handler::member::{
        create_member, delete_member, get_member, get_member_by_username,
        list_members, update_member, update_member_avatar, update_member_password,
        update_member_role,
    },
    state::AppState,
};
use axum::{
    Router,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

/// 创建成员路由（需要认证）
pub fn member_router() -> Router<Arc<AppState>> {
    Router::new()
        // 创建成员（仅管理员）
        .route("/", post(create_member))
        // 获取成员列表（仅管理员）
        .route("/", get(list_members))
        // 获取成员详情
        .route("/{id}", get(get_member))
        // 更新成员信息
        .route("/{id}", put(update_member))
        // 删除成员（仅管理员）
        .route("/{id}", delete(delete_member))
        // 根据用户名查询成员（仅管理员）
        .route("/username/{username}", get(get_member_by_username))
        // 更新成员头像
        .route("/{id}/avatar", put(update_member_avatar))
        // 更新成员密码
        .route("/{id}/password", put(update_member_password))
        // 更新成员角色（仅管理员）
        .route("/{id}/role", put(update_member_role))
}
