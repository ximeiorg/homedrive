//! 相册路由模块

use crate::handler::album::{
    add_files_to_album, create_album, delete_album, get_album, list_album_files, list_albums,
    remove_files_from_album, update_album,
};
use crate::state::AppState;
use axum::{
    Router,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

/// 创建相册模块路由
pub fn album_router() -> Router<Arc<AppState>> {
    Router::new()
        // 创建相册 POST /api/members/{id}/albums
        .route("/", post(create_album))
        // 列出相册列表 GET /api/members/{id}/albums
        .route("/", get(list_albums))
        // 获取相册详情 GET /api/members/{id}/albums/{album_id}
        .route("/{album_id}", get(get_album))
        // 更新相册 PUT /api/members/{id}/albums/{album_id}
        .route("/{album_id}", put(update_album))
        // 删除相册 DELETE /api/members/{id}/albums/{album_id}
        .route("/{album_id}", delete(delete_album))
        // 列出相册中的文件 GET /api/members/{id}/albums/{album_id}/files
        .route("/{album_id}/files", get(list_album_files))
        // 添加文件到相册 POST /api/members/{id}/albums/{album_id}/files
        .route("/{album_id}/files", post(add_files_to_album))
        // 从相册中移除文件 DELETE /api/members/{id}/albums/{album_id}/files
        .route("/{album_id}/files", delete(remove_files_from_album))
}
