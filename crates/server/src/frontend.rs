//! 前端静态资源嵌入模块
//!
//! 使用 rust-embed 将构建后的前端资源嵌入到二进制文件中

use axum::{
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

/// 前端静态资源
#[derive(RustEmbed)]
#[folder = "../../web/build/client/"]
pub struct FrontendAssets;

pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();

        // 判断是否是静态资源请求（包含文件扩展名）
        let is_static_asset = path.contains('.');

        // 首先尝试直接查找
        let content = FrontendAssets::get(path.as_str());

        match content {
            Some(data) => {
                let mime = mime_guess::from_path(&path).first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref())], data.data).into_response()
            }
            None => {
                // 如果是静态资源且找不到，返回 404
                if is_static_asset {
                    return (StatusCode::NOT_FOUND, "Not Found").into_response();
                }

                // 否则返回 index.html 用于 SPA 路由
                let index_path = "index.html";
                if let Some(index_data) = FrontendAssets::get(index_path) {
                    let mime = mime_guess::from_path(index_path).first_or_octet_stream();
                    ([(header::CONTENT_TYPE, mime.as_ref())], index_data.data).into_response()
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, "index.html not found").into_response()
                }
            }
        }
    }
}

pub async fn index_handler(uri: Uri) -> impl IntoResponse + use<> {
    handler_404(uri).await
}

async fn handler_404(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/').to_string();

    // 处理 build 前缀
    let processed_path = if path.starts_with("build/") {
        path.replace("build/", "")
    } else {
        path
    };

    let final_path = if processed_path.is_empty() {
        "index.html".to_string()
    } else {
        processed_path
    };

    println!("[Handler] uri: {} -> path: {}", uri.path(), final_path);
    StaticFile(final_path)
}
