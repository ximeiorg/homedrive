//! 前端静态资源嵌入模块
//!
//! 使用 rust-embed 将构建后的前端资源嵌入到二进制文件中

use rust_embed::RustEmbed;
use std::borrow::Cow;

/// 前端静态资源
#[derive(RustEmbed)]
#[folder = "../../web/build/client/"]
pub struct FrontendAssets;

/// 获取前端资源
pub fn get_frontend_asset(path: &str) -> Option<FrontendAsset> {
    // 去掉前导斜杠
    let path = path.strip_prefix('/').unwrap_or(path);
    
    // 尝试直接获取文件
    if let Some(asset) = FrontendAssets::get(path) {
        return Some(FrontendAsset {
            data: asset.data,
            is_html: path.ends_with("index.html"),
        });
    }
    
    // 如果是目录，尝试 index.html
    if !path.contains('.') {
        let index_path = format!("{}/index.html", path);
        if let Some(asset) = FrontendAssets::get(&index_path) {
            return Some(FrontendAsset {
                data: asset.data,
                is_html: true,
            });
        }
    }
    
    // 返回 index.html（SPA 路由支持）
    if let Some(asset) = FrontendAssets::get("index.html") {
        return Some(FrontendAsset {
            data: asset.data,
            is_html: true,
        });
    }
    
    None
}

/// 前端资源封装
pub struct FrontendAsset {
    pub data: Cow<'static, [u8]>,
    pub is_html: bool,
}

/// 获取 MIME 类型
pub fn get_mime_type(path: &str) -> &'static str {
    let path = path.strip_prefix('/').unwrap_or(path);
    
    if path.ends_with(".html") || path == "index.html" {
        return "text/html; charset=utf-8";
    }
    if path.ends_with(".js") {
        return "application/javascript; charset=utf-8";
    }
    if path.ends_with(".css") {
        return "text/css; charset=utf-8";
    }
    if path.ends_with(".json") {
        return "application/json; charset=utf-8";
    }
    if path.ends_with(".png") {
        return "image/png";
    }
    if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        return "image/jpeg";
    }
    if path.ends_with(".svg") {
        return "image/svg+xml";
    }
    if path.ends_with(".ico") {
        return "image/x-icon";
    }
    if path.ends_with(".woff") {
        return "font/woff";
    }
    if path.ends_with(".woff2") {
        return "font/woff2";
    }
    
    "application/octet-stream"
}
