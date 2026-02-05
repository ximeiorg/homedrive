//! 文件模块路由
//!
//! 定义所有文件相关的路由

use crate::{
    auth::auth_middleware,
    handler::file::{
        check_file_hash_exists, list_files, serve_file, trigger_sync_files, upload_file,
    },
    state::AppState,
};
use axum::{
    Router,
    routing::{get, post},
};

/// 创建文件路由
pub fn file_router() -> Router<AppState> {
    Router::new()
        // 公开路由
        .route("/check-hash", get(check_file_hash_exists))
        // 受保护路由（每个路由单独添加中间件）
        .route(
            "/upload",
            post(upload_file).layer(axum::middleware::from_fn(auth_middleware)),
        )
        .route(
            "/list",
            get(list_files).layer(axum::middleware::from_fn(auth_middleware)),
        )
        .route(
            "/sync",
            post(trigger_sync_files).layer(axum::middleware::from_fn(auth_middleware)),
        )
}

/// 创建静态文件路由
pub fn static_router() -> Router<AppState> {
    Router::new().route(
        "/{storage_tag}/{*path}",
        get(serve_file).layer(axum::middleware::from_fn(auth_middleware)),
    )
}
