//! 任务模块路由
//!
//! 定义所有任务相关的路由

use crate::{
    handler::file::{get_task, list_tasks},
    state::AppState,
};
use axum::{Router, routing::get};
use std::sync::Arc;

/// 创建任务路由
pub fn task_router() -> Router<Arc<AppState>> {
    Router::new()
        // 任务相关路由
        .route("/", get(list_tasks))
        .route("/{id}", get(get_task))
}
