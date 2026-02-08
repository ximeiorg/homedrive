//! 路由模块
//!
//! 定义应用的所有路由入口，按模块分区组织

mod auth;
mod file;
mod member;
mod task;

use crate::{
    handler::member::{check_members_empty, check_username_exists, init_admin},
    handler::system::get_system_stats,
    state::AppState,
};
pub use auth::auth_router;
use axum::{
    Router,
    routing::{get, post},
};
pub use file::{file_router, static_router};
use hyper::Method;
pub use member::member_router;
pub use task::task_router;
use std::sync::Arc;
use tower_http::cors::Any;
use tower_http::cors::CorsLayer;

/// 创建应用主路由
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // 公开路由
        .route("/username/{username}/exists", get(check_username_exists))
        .route("/empty", get(check_members_empty))
        .route("/init", post(init_admin))
        // 系统状态路由（公开）
        .route("/system/stats", get(get_system_stats))
        // 成员模块路由（需要认证）
        .nest("/members", member_router())
        // 文件模块路由
        .nest("/files", file_router())
        // 任务模块路由
        .nest("/tasks", task_router())
        // 静态文件路由
        .nest("/static", static_router())
        // 认证模块路由
        .nest("/auth", auth_router())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers(Any),
        )
}
