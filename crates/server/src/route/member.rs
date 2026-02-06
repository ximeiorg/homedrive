//! 成员模块路由（受保护）
//!
//! 定义所有需要认证的成员相关路由

use crate::{
    handler::member::{
        create_member, delete_member, get_member, get_member_by_username, list_members,
        update_member, update_member_avatar, update_member_password,
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
        .route("/", post(create_member))
        .route("/", get(list_members))
        .route("/{id}", get(get_member))
        .route("/{id}", put(update_member))
        .route("/{id}", delete(delete_member))
        .route("/username/{username}", get(get_member_by_username))
        .route("/{id}/avatar", put(update_member_avatar))
        .route("/{id}/password", put(update_member_password))
}
