//! 认证模块路由
//!
//! 定义所有认证相关的路由

use crate::{handler::member::login, state::AppState};
use axum::{Router, routing::post};

/// 创建认证路由
pub fn auth_router() -> Router<AppState> {
    Router::new().route("/login", post(login))
}
