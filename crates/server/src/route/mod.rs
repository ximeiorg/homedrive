//! 路由模块
//!
//! 定义应用的所有路由入口，按模块分区组织

mod album;
mod auth;
mod file;
mod member;
mod task;

use crate::{
    auth::AdminOnly,
    frontend::{get_frontend_asset, get_mime_type},
    handler::member::{check_members_empty, check_username_exists, init_admin},
    handler::system::get_system_stats,
    state::AppState,
};
pub use album::album_router;
pub use auth::auth_router;
use axum::{
    body::Body,
    extract::Path,
    response::IntoResponse,
    Router,
    routing::{get, post},
};
pub use file::{file_router, static_router};
use hyper::{Method, Response};
use hyper::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
pub use member::member_router;
use std::sync::Arc;
pub use task::task_router;
use tower_http::cors::{AllowOrigin, CorsLayer};

/// 静态文件服务（SPA fallback）
async fn serve_frontend(
    Path(path): Path<String>,
) -> impl IntoResponse {
    let path = path.trim_start_matches('/');
    
    if let Some(asset) = get_frontend_asset(path) {
        let mime = get_mime_type(path);
        Response::builder()
            .status(hyper::StatusCode::OK)
            .header("Content-Type", mime)
            .header("Cache-Control", "public, max-age=31536000")
            .body(Body::from(asset.data.to_vec()))
            .unwrap()
    } else {
        // 返回 index.html（SPA 路由支持）
        if let Some(asset) = get_frontend_asset("index.html") {
            Response::builder()
                .status(hyper::StatusCode::OK)
                .header("Content-Type", "text/html; charset=utf-8")
                .body(Body::from(asset.data.to_vec()))
                .unwrap()
        } else {
            Response::builder()
                .status(hyper::StatusCode::NOT_FOUND)
                .body(Body::from("Not Found"))
                .unwrap()
        }
    }
}

/// 创建应用主路由
pub fn routes(state: &Arc<AppState>) -> Router<Arc<AppState>> {
    // 解析 CORS 允许来源
    let cors_origin = &state.config.server.cors_origin;
    let (allow_origin, allow_credentials) = if cors_origin == "*" {
        // 开发模式：允许所有来源，但不能使用 credentials
        tracing::warn!(
            "CORS is configured to allow all origins. This is not recommended for production."
        );
        (AllowOrigin::any(), false)
    } else {
        // 生产模式：只允许指定的来源，可以使用 credentials
        let origins: Vec<String> = cors_origin
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        // 解析为 HeaderValue
        let origin_values: Vec<hyper::header::HeaderValue> = origins
            .into_iter()
            .filter_map(|origin| {
                match hyper::header::HeaderValue::try_from(origin.as_str()) {
                    Ok(v) => Some(v),
                    Err(e) => {
                        tracing::warn!(origin = %origin, error = ?e, "Invalid CORS origin, skipping");
                        None
                    }
                }
            })
            .collect();

        if origin_values.is_empty() {
            tracing::warn!("No valid CORS origins configured, allowing all origins");
            (AllowOrigin::any(), false)
        } else {
            tracing::info!(origins = ?origin_values, "CORS origins configured");
            (AllowOrigin::list(origin_values), true)
        }
    };

    Router::new()
        // 公开路由
        .route("/username/{username}/exists", get(check_username_exists))
        .route("/empty", get(check_members_empty))
        .route("/init", post(init_admin))
        // 系统状态路由（公开）
        .route("/system/stats", get(get_system_stats))
        // 成员模块路由（需要认证）
        .nest("/members", member_router())
        // 相册模块路由（嵌套在 members 下）
        .nest("/members/{id}/albums", album_router())
        // 文件模块路由
        .nest("/files", file_router())
        // 任务模块路由
        .nest("/tasks", task_router())
        // 静态文件路由
        .nest("/static", static_router())
        // 认证模块路由
        .nest("/auth", auth_router())
        // 前端 SPA 路由（catch-all）
        .fallback(serve_frontend)
        .layer(
            CorsLayer::new()
                .allow_origin(allow_origin)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([ACCEPT, AUTHORIZATION, CONTENT_TYPE])
                .allow_credentials(allow_credentials)
                .max_age(std::time::Duration::from_secs(3600)),
        )
}
