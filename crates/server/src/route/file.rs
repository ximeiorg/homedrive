//! 文件模块路由
//!
//! 定义所有文件相关的路由

use crate::{
    handler::file::{check_file_hash_exists, list_files, serve_file, sync_files, upload_file},
    state::AppState,
};
use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;

/// 创建文件路由
pub fn file_router() -> Router<Arc<AppState>> {
    Router::new()
        // 公开路由
        .route("/check-hash", get(check_file_hash_exists))
        // 受保护路由（使用 FromRequestParts 自动认证）
        .route("/upload", post(upload_file))
        .route("/", get(list_files))
        .route("/sync", post(sync_files))
}

/// 创建静态文件路由
/// 支持两种格式：
/// 1. /api/static/{storage_tag}/{*path}?token={token} - URL query 中带 token（用于 img 标签等）
/// 2. /api/static/{storage_tag}/{*path} - 需要 Authorization header
pub fn static_router() -> Router<Arc<AppState>> {
    Router::new().route("/{storage_tag}/{*path}", get(serve_file))
}
