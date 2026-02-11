use crate::auth::Authorized;
use crate::error::AppError;
use crate::state::AppState;
use axum::{
    extract::Path,
    extract::Query,
    extract::State,
    Json,
};
use std::sync::Arc;

use schema::album::{
    AddFilesRequest, AddFilesResponse, AlbumFileInfo, AlbumFilesResponse, AlbumListItem,
    AlbumListResponse, AlbumResponse, CreateAlbumRequest, MessageResponse, PaginationQuery,
    RemoveFilesRequest, RemoveFilesResponse, UpdateAlbumRequest,
};

/// 创建相册
/// POST /api/members/{id}/albums
pub async fn create_album(
    State(state): State<Arc<AppState>>,
    auth: Authorized,
    Path(member_id): Path<i64>,
    Json(request): Json<CreateAlbumRequest>,
) -> Result<Json<AlbumResponse>, AppError> {
    // 验证用户只能为自己创建相册
    if auth.0 != member_id {
        return Err(AppError::Forbidden);
    }

    let params = services::CreateAlbumParams {
        name: request.name,
        description: request.description,
        cover_file_id: request.cover_file_id,
        file_ids: request.file_ids,
    };

    let album = services::AlbumService::create_album(&state.conn, member_id, params).await?;

    Ok(Json(AlbumResponse {
        id: album.id,
        member_id: album.member_id,
        name: album.name,
        description: album.description,
        cover_file_id: album.cover_file_id,
        file_count: album.file_count,
        created_at: album.created_at,
        updated_at: album.updated_at,
    }))
}

/// 列出用户的相册
/// GET /api/members/{id}/albums
pub async fn list_albums(
    State(state): State<Arc<AppState>>,
    auth: Authorized,
    Path(member_id): Path<i64>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<AlbumListResponse>, AppError> {
    // 验证用户只能查看自己的相册
    if auth.0 != member_id {
        return Err(AppError::Forbidden);
    }

    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let (albums, total) =
        services::AlbumService::list_albums(&state.conn, member_id, Some(page), Some(page_size))
            .await?;

    let items: Vec<AlbumListItem> = albums
        .into_iter()
        .map(|a| AlbumListItem {
            id: a.id,
            name: a.name,
            description: a.description,
            cover_file_id: a.cover_file_id,
            file_count: a.file_count,
            created_at: a.created_at,
            updated_at: a.updated_at,
        })
        .collect();

    Ok(Json(AlbumListResponse {
        albums: items,
        total,
        page,
        page_size,
    }))
}

/// 获取相册详情
/// GET /api/members/{id}/albums/{album_id}
pub async fn get_album(
    State(state): State<Arc<AppState>>,
    auth: Authorized,
    Path((member_id, album_id)): Path<(i64, i64)>,
) -> Result<Json<AlbumResponse>, AppError> {
    // 验证用户权限
    if auth.0 != member_id {
        return Err(AppError::Forbidden);
    }

    let album = services::AlbumService::get_album(&state.conn, member_id, album_id).await?;

    Ok(Json(AlbumResponse {
        id: album.id,
        member_id: album.member_id,
        name: album.name,
        description: album.description,
        cover_file_id: album.cover_file_id,
        file_count: album.file_count,
        created_at: album.created_at,
        updated_at: album.updated_at,
    }))
}

/// 更新相册
/// PUT /api/members/{id}/albums/{album_id}
pub async fn update_album(
    State(state): State<Arc<AppState>>,
    auth: Authorized,
    Path((member_id, album_id)): Path<(i64, i64)>,
    Json(request): Json<UpdateAlbumRequest>,
) -> Result<Json<AlbumResponse>, AppError> {
    // 验证用户权限
    if auth.0 != member_id {
        return Err(AppError::Forbidden);
    }

    let params = services::UpdateAlbumParams {
        name: request.name,
        description: request.description,
        cover_file_id: request.cover_file_id,
    };

    let album =
        services::AlbumService::update_album(&state.conn, member_id, album_id, params).await?;

    Ok(Json(AlbumResponse {
        id: album.id,
        member_id: album.member_id,
        name: album.name,
        description: album.description,
        cover_file_id: album.cover_file_id,
        file_count: album.file_count,
        created_at: album.created_at,
        updated_at: album.updated_at,
    }))
}

/// 删除相册
/// DELETE /api/members/{id}/albums/{album_id}
pub async fn delete_album(
    State(state): State<Arc<AppState>>,
    auth: Authorized,
    Path((member_id, album_id)): Path<(i64, i64)>,
) -> Result<Json<MessageResponse>, AppError> {
    // 验证用户权限
    if auth.0 != member_id {
        return Err(AppError::Forbidden);
    }

    services::AlbumService::delete_album(&state.conn, member_id, album_id).await?;

    Ok(Json(MessageResponse {
        message: "相册已删除".to_string(),
    }))
}

/// 列出相册中的文件
/// GET /api/members/{id}/albums/{album_id}/files
pub async fn list_album_files(
    State(state): State<Arc<AppState>>,
    auth: Authorized,
    Path((member_id, album_id)): Path<(i64, i64)>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<AlbumFilesResponse>, AppError> {
    // 验证用户权限
    if auth.0 != member_id {
        return Err(AppError::Forbidden);
    }

    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(100);

    let (files, total) = services::AlbumService::list_album_files(
        &state.conn,
        member_id,
        album_id,
        Some(page),
        Some(page_size),
    )
    .await?;

    let file_infos: Vec<AlbumFileInfo> = files
        .into_iter()
        .map(|(member_file, file_content)| {
            let (file_size, mime_type) = match file_content {
                Some(fc) => (fc.file_size, fc.mime_type),
                None => (0, "application/octet-stream".to_string()),
            };

            AlbumFileInfo {
                id: member_file.id,
                file_name: member_file.file_name,
                file_size,
                mime_type,
                description: member_file.description,
                created_at: member_file.created_at,
                updated_at: member_file.updated_at,
            }
        })
        .collect();

    Ok(Json(AlbumFilesResponse {
        files: file_infos,
        total,
        page,
        page_size,
    }))
}

/// 添加文件到相册
/// POST /api/members/{id}/albums/{album_id}/files
pub async fn add_files_to_album(
    State(state): State<Arc<AppState>>,
    auth: Authorized,
    Path((member_id, album_id)): Path<(i64, i64)>,
    Json(request): Json<AddFilesRequest>,
) -> Result<Json<AddFilesResponse>, AppError> {
    // 验证用户权限
    if auth.0 != member_id {
        return Err(AppError::Forbidden);
    }

    let added_count =
        services::AlbumService::add_files_to_album(&state.conn, member_id, album_id, request.file_ids)
            .await?;

    Ok(Json(AddFilesResponse { added_count }))
}

/// 从相册中移除文件
/// DELETE /api/members/{id}/albums/{album_id}/files
pub async fn remove_files_from_album(
    State(state): State<Arc<AppState>>,
    auth: Authorized,
    Path((member_id, album_id)): Path<(i64, i64)>,
    Json(request): Json<RemoveFilesRequest>,
) -> Result<Json<RemoveFilesResponse>, AppError> {
    // 验证用户权限
    if auth.0 != member_id {
        return Err(AppError::Forbidden);
    }

    let removed_count = services::AlbumService::remove_files_from_album(
        &state.conn,
        member_id,
        album_id,
        request.file_ids,
    )
    .await?;

    Ok(Json(RemoveFilesResponse { removed_count }))
}
