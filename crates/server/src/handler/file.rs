use crate::auth::Auth;
use crate::error::AppError;
use crate::state::AppState;
use axum::{extract::Path, extract::Query, Extension, Json, response::IntoResponse};
use sea_orm::prelude::DateTimeUtc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use store::member_file::query::{ListMemberFilesQuery, SortField, SortOrder, FileTypeFilter};

#[derive(Serialize)]
pub struct HashCheckResponse {
    pub exists: bool,
}

#[derive(Deserialize)]
pub struct HashCheckQuery {
    pub hash: String,
}

/// Check if a file hash already exists in the database
pub async fn check_file_hash_exists(
    Extension(state): Extension<Arc<AppState>>,
    Query(query): Query<HashCheckQuery>,
) -> Json<HashCheckResponse> {
    let db = &state.conn;

    let exists = services::FileService::check_hash_exists(db, &query.hash)
        .await
        .unwrap_or(false);

    Json(HashCheckResponse { exists })
}

/// Upload file request
#[derive(Deserialize)]
pub struct UploadFileRequest {
    pub hash: String,
}

/// Upload file response
#[derive(Serialize)]
pub struct UploadFileResponse {
    pub success: bool,
    pub file_id: i64,
    pub message: String,
}

/// Upload file handler - requires authentication
pub async fn upload_file(
    Extension(state): Extension<Arc<AppState>>,
    Extension(auth): Extension<Auth>,
    mut multipart: axum::extract::Multipart,
) -> Json<UploadFileResponse> {
    let db = &state.conn;
    let uploader_id = auth.0;

    // Get the file from multipart
    if let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let filename = field.file_name().unwrap_or("unknown").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        let data = field.bytes().await.unwrap_or_default();

        // Get the hash from form field
        let hash_field = multipart.next_field().await.unwrap_or(None);
        let hash = if let Some(f) = hash_field {
            f.text().await.unwrap_or_default()
        } else {
            String::new()
        };

        // Check if hash already exists
        if let Some(existing_id) = services::FileService::find_by_hash(db, &hash).await.unwrap_or(None) {
            return Json(UploadFileResponse {
                success: true,
                file_id: existing_id,
                message: "File already exists".to_string(),
            });
        }

        // Upload file (save to storage + create database record)
        let file_id = services::FileService::upload_file(
            db,
            &state.storage,
            data.to_vec(),
            hash,
            content_type,
            filename,
            uploader_id,
        )
        .await
        .unwrap_or(0);

        Json(UploadFileResponse {
            success: true,
            file_id,
            message: "File uploaded successfully".to_string(),
        })
    } else {
        Json(UploadFileResponse {
            success: false,
            file_id: 0,
            message: "No file provided".to_string(),
        })
    }
}

/// Parse Range header into (start, end) byte positions
fn parse_range_header(range: &str, file_size: u64) -> Option<(u64, u64)> {
    // Format: "bytes=start-end"
    if !range.starts_with("bytes=") {
        return None;
    }

    let range = &range[6..];
    let parts: Vec<&str> = range.split('-').collect();
    if parts.len() != 2 {
        return None;
    }

    let start = parts[0].parse::<u64>().ok()?;
    let end = parts[1].parse::<u64>().ok()?;

    if start >= file_size {
        return None;
    }

    Some((start, std::cmp::min(end, file_size - 1)))
}

/// Serve file handler with Range support - requires authentication
/// Path format: /files/{storage_tag}/{file_path}
pub async fn serve_file(
    Extension(state): Extension<Arc<AppState>>,
    Extension(auth): Extension<Auth>,
    Path((storage_tag, file_path)): Path<(String, String)>,
    req: axum::http::Request<axum::body::Body>,
) -> impl IntoResponse {
    let db = &state.conn;
    let user_id = auth.0;

    // Get user's storage_tag from database
    let user = match store::member::query::Query::find_by_id(db, user_id).await {
        Ok(Some(m)) => m,
        Ok(None) => return Err(AppError::InvalidCredentials),
        Err(_) => return Err(AppError::InvalidCredentials),
    };

    // Verify the requested storage_tag matches the user's storage_tag
    if storage_tag != user.storage_tag {
        return Err(AppError::Forbidden);
    }

    // Get storage root from config
    let storage_root = &state.config.storage.volume;
    
    // Build the file path following LocalStorage's directory structure
    // LocalStorage uses: root/{hash_prefix}/{storage_tag}/{file_path}
    let mut file_path_buf = PathBuf::from(storage_root);
    if storage_tag.len() >= 2 {
        file_path_buf.push(&storage_tag[0..2]);
    }
    file_path_buf.push(&storage_tag);
    
    // Add the file path components
    for part in file_path.split('/') {
        if !part.is_empty() {
            file_path_buf.push(part);
        }
    }

    // Check if file exists
    if !file_path_buf.exists() {
        return Err(AppError::NotFound);
    }

    // Get file metadata
    let metadata = match tokio::fs::metadata(&file_path_buf).await {
        Ok(m) => m,
        Err(_) => return Err(AppError::NotFound),
    };

    if !metadata.is_file() {
        return Err(AppError::NotFound);
    }

    let file_size = metadata.len();
    let content_type = mime_guess::from_path(&file_path_buf)
        .first()
        .map(|m| m.to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());

    // Check for Range header
    let range_header = req.headers().get("range");

    let (status, headers, body) = if let Some(range_value) = range_header {
        if let Some((start, end)) = parse_range_header(
            range_value.to_str().unwrap_or(""),
            file_size,
        ) {
            let content_length = end - start + 1;

            // Open file for streaming
            let file = match File::open(&file_path_buf).await {
                Ok(f) => f,
                Err(_) => return Err(AppError::NotFound),
            };

            let stream = ReaderStream::new(file);
            let headers = vec![
                (axum::http::header::CONTENT_TYPE, content_type),
                (axum::http::header::CONTENT_LENGTH, content_length.to_string()),
                (axum::http::header::CONTENT_RANGE, format!("bytes {}-{}/{}", start, end, file_size)),
            ];
            (axum::http::StatusCode::PARTIAL_CONTENT, headers, axum::body::Body::from_stream(stream))
        } else {
            // Invalid range, return full file
            let file = match File::open(&file_path_buf).await {
                Ok(f) => f,
                Err(_) => return Err(AppError::NotFound),
            };
            let stream = ReaderStream::new(file);
            let headers = vec![
                (axum::http::header::CONTENT_TYPE, content_type),
                (axum::http::header::CONTENT_LENGTH, file_size.to_string()),
            ];
            (axum::http::StatusCode::OK, headers, axum::body::Body::from_stream(stream))
        }
    } else {
        // No Range header, return full file
        let file = match File::open(&file_path_buf).await {
            Ok(f) => f,
            Err(_) => return Err(AppError::NotFound),
        };
        let stream = ReaderStream::new(file);
        let headers = vec![
            (axum::http::header::CONTENT_TYPE, content_type),
            (axum::http::header::CONTENT_LENGTH, file_size.to_string()),
        ];
        (axum::http::StatusCode::OK, headers, axum::body::Body::from_stream(stream))
    };

    let mut response = axum::http::Response::builder()
        .status(status)
        .body(body)
        .map_err(|_| AppError::InvalidCredentials)?;

    for (name, value) in headers {
        response.headers_mut().insert(name, value.parse().unwrap());
    }

    Ok(response)
}

/// 文件列表项响应
#[derive(Serialize)]
pub struct FileListItem {
    pub id: i64,
    pub file_name: String,
    pub description: String,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

/// 文件列表响应
#[derive(Serialize)]
pub struct FileListResponse {
    pub files: Vec<FileListItem>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
}

/// 列出用户文件的查询参数
#[derive(Deserialize)]
pub struct ListFilesQuery {
    /// 页码，从 1 开始
    pub page: Option<u64>,
    /// 每页大小，默认 100
    pub page_size: Option<u64>,
    /// 排序字段：created_at, file_name, file_size
    pub sort_by: Option<String>,
    /// 排序方向：asc, desc
    pub sort_order: Option<String>,
    /// 文件类型：image, video, audio, document, archive, other
    pub file_type: Option<String>,
    /// 文件名搜索
    pub search: Option<String>,
}

/// 列出当前用户文件 - 需要认证
pub async fn list_files(
    Extension(state): Extension<Arc<AppState>>,
    Extension(auth): Extension<Auth>,
    Query(query): Query<ListFilesQuery>,
) -> Json<FileListResponse> {
    let db = &state.conn;
    let member_id = auth.0;

    // 转换查询参数
    let sort_by = query.sort_by.as_ref().map(|s| match s.as_str() {
        "file_name" => SortField::FileName,
        "file_size" => SortField::FileSize,
        _ => SortField::CreatedAt,
    });

    let sort_order = query.sort_order.as_ref().map(|s| match s.as_str() {
        "asc" => SortOrder::Asc,
        _ => SortOrder::Desc,
    });

    let file_type = query.file_type.as_ref().map(|s| match s.as_str() {
        "image" => FileTypeFilter::Image,
        "video" => FileTypeFilter::Video,
        "audio" => FileTypeFilter::Audio,
        "document" => FileTypeFilter::Document,
        "archive" => FileTypeFilter::Archive,
        _ => FileTypeFilter::Other,
    });

    let list_query = ListMemberFilesQuery {
        page: query.page,
        page_size: query.page_size,
        sort_by,
        sort_order,
        file_type,
        search: query.search,
    };

    // 查询文件列表
    let (results, total) = store::member_file::query::Query::list_files_by_member(
        db,
        member_id,
        list_query,
    )
    .await
    .unwrap_or((Vec::new(), 0));

    // 转换结果
    let files: Vec<FileListItem> = results
        .into_iter()
        .map(|member_file| {
            FileListItem {
                id: member_file.id,
                file_name: member_file.file_name,
                description: member_file.description,
                file_size: None, // 需要关联查询获取
                mime_type: None, // 需要关联查询获取
                created_at: member_file.created_at,
                updated_at: member_file.updated_at,
            }
        })
        .collect();

    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(100);
    let total_pages = (total as f64 / page_size as f64).ceil() as u64;

    Json(FileListResponse {
        files,
        total,
        page,
        page_size,
        total_pages,
    })
}



/// 触发同步任务的请求
#[derive(Deserialize)]
pub struct TriggerSyncRequest {
    pub path: Option<String>,
    pub task_type: Option<String>,
    pub recursive: Option<bool>,
}

/// 触发同步任务的响应
#[derive(Serialize)]
pub struct TriggerSyncResponse {
    pub success: bool,
    pub message: String,
}

/// 触发同步任务 - 需要认证
pub async fn trigger_sync_files(
    Extension(state): Extension<Arc<AppState>>,
    Extension(auth): Extension<Auth>,
    Json(req): Json<TriggerSyncRequest>,
) -> Json<TriggerSyncResponse> {
    let member_id = auth.0;
    
    // 确定路径和任务类型
    let path = req.path.unwrap_or_else(|| state.config.storage.volume.clone());
    let task_type = req.task_type.as_deref().unwrap_or("sync_files");
    
    // 创建任务负载
    let payload = services::TaskPayload {
        task_type: task_type.to_string(),
        member_id,
        path,
        options: Some(services::TaskOptions {
            recursive: req.recursive,
            file_types: None,
            include_hidden: Some(false),
        }),
    };

    match state.sync_task_sender.send(payload).await {
        Ok(()) => Json(TriggerSyncResponse {
            success: true,
            message: "Task queued successfully".to_string(),
        }),
        Err(e) => Json(TriggerSyncResponse {
            success: false,
            message: format!("Failed to queue task: {}", e),
        }),
    }
}
