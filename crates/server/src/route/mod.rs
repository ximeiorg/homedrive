//! 路由模块
//!
//! 定义应用的所有路由入口，按模块分区组织

mod auth;
mod file;
mod member;

pub use auth::auth_router;
pub use file::{file_router, static_router};
pub use member::member_router;

use crate::{
    handler::member::{check_members_empty, check_username_exists, init_admin},
    state::AppState,
};
use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;

/// 创建应用主路由
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // 公开路由
        .route("/username/{username}/exists", get(check_username_exists))
        .route("/empty", get(check_members_empty))
        .route("/init", post(init_admin))
        // 成员模块路由（需要认证）
        .nest("/members", member_router())
        // 文件模块路由
        .nest("/files", file_router())
        // 静态文件路由
        .nest("/static", static_router())
        // 认证模块路由
        .nest("/auth", auth_router())
}
