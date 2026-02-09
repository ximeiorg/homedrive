use crate::auth::Authorized;
use crate::error::AppError;
use crate::state::AppState;
use axum::{Json, extract::Path, extract::Query, extract::State, extract::Request, response::IntoResponse};
use chrono::Utc;
use sea_orm::QueryOrder;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use schema::file::{
    HashCheckQuery, HashCheckResponse, ListFilesQuery, SyncFilesRequest, SyncFilesResponse,
    TaskItemResponse, TaskListResponse, TriggerSyncRequest, TriggerSyncResponse,
    UploadFileResponse,
};

/// Check if a file hash already exists in the database
pub async fn check_file_hash_exists(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HashCheckQuery>,
) -> Json<HashCheckResponse> {
    let db = &state.conn;

    let exists = services::FileService::check_hash_exists(db, &query.hash)
        .await
        .unwrap_or(false);

    Json(HashCheckResponse { exists })
}

/// Upload file handler - requires authentication
pub async fn upload_file(
    State(state): State<Arc<AppState>>,
    auth: Authorized,
    mut multipart: axum::extract::Multipart,
) -> Json<UploadFileResponse> {
    let db = &state.conn;
    let uploader_id = auth.0;

    // Get the file from multipart
    if let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let filename = field.file_name().unwrap_or("unknown").to_string();
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        let data = field.bytes().await.unwrap_or_default();

        // Get the hash from form field
        let hash_field = multipart.next_field().await.unwrap_or(None);
        let hash = if let Some(f) = hash_field {
            f.text().await.unwrap_or_default()
        } else {
            String::new()
        };

        // Check if hash already exists
        if let Some(existing_id) = services::FileService::find_by_hash(db, &hash)
            .await
            .unwrap_or(None)
        {
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
    State(state): State<Arc<AppState>>,
    auth: Authorized,
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
    // LocalStorage uses: root/{storage_tag}/{file_path}
    // storage_path already contains: storage_tag/file_path
    let mut file_path_buf = PathBuf::from(storage_root);
    file_path_buf.push(storage_tag);
    file_path_buf.push(&file_path);

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
        if let Some((start, end)) =
            parse_range_header(range_value.to_str().unwrap_or(""), file_size)
        {
            let content_length = end - start + 1;

            // Open file for streaming
            let file = match File::open(&file_path_buf).await {
                Ok(f) => f,
                Err(_) => return Err(AppError::NotFound),
            };

            let stream = ReaderStream::new(file);
            let headers = vec![
                (axum::http::header::CONTENT_TYPE, content_type),
                (
                    axum::http::header::CONTENT_LENGTH,
                    content_length.to_string(),
                ),
                (
                    axum::http::header::CONTENT_RANGE,
                    format!("bytes {}-{}/{}", start, end, file_size),
                ),
            ];
            (
                axum::http::StatusCode::PARTIAL_CONTENT,
                headers,
                axum::body::Body::from_stream(stream),
            )
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
            (
                axum::http::StatusCode::OK,
                headers,
                axum::body::Body::from_stream(stream),
            )
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
        (
            axum::http::StatusCode::OK,
            headers,
            axum::body::Body::from_stream(stream),
        )
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

/// 列出当前用户文件 - 需要认证
pub async fn list_files(
    State(state): State<Arc<AppState>>,
    Authorized(member_id): Authorized,
    Query(query): Query<ListFilesQuery>,
) -> Json<schema::file::FileListResponse> {
    use schema::file::FileListItem;

    // 通过 services 层查询文件列表
    let (results, total) = services::FileService::list_member_files(
        &state.conn,
        member_id,
        query.page,
        query.page_size,
        query.sort_by,
        query.sort_order,
        query.file_type,
        query.search,
    )
    .await
    .unwrap_or((Vec::new(), 0));

    // 获取 base_url 用于构建文件访问 URL
    let base_url = state.config.base_url.clone();

    // 转换结果
    let files: Vec<FileListItem> = results
        .into_iter()
        .map(|(member_file, file_content)| {
            // 从 file_content 获取 storage_path 和 mime_type
            // storage_path 格式: storage_tag/file_path
            // 静态文件路由: /api/static/{storage_tag}/{*path}
            let (storage_path, mime_type, file_size, thumbnail) = match file_content {
                Some(ref fc) => (
                    fc.storage_path.clone(),
                    Some(fc.mime_type.clone()),
                    Some(fc.file_size),
                    fc.thumbnail.clone(),
                ),
                None => (String::new(), None, None, None),
            };

            // 构建文件访问 URL: {base_url}/api/static/{storage_path}
            let url = if storage_path.is_empty() {
                None
            } else {
                Some(format!("{}/api/static/{}", base_url, storage_path))
            };

            // 构建缩略图 URL
            let thumbnail_url = thumbnail.and_then(|t| {
                if t.is_empty() {
                    None
                } else {
                    Some(format!("{}/api/static/{}", base_url, t))
                }
            });

            FileListItem {
                id: member_file.id,
                file_name: member_file.file_name,
                description: member_file.description,
                file_size,
                mime_type,
                thumbnail: thumbnail_url,
                url,
                created_at: member_file.created_at,
                updated_at: member_file.updated_at,
            }
        })
        .collect();

    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(100);
    let total_pages = (total as f64 / page_size as f64).ceil() as u64;

    Json(schema::file::FileListResponse {
        files,
        total,
        page,
        page_size,
        total_pages,
    })
}

/// 触发同步任务 - 需要认证
pub async fn trigger_sync_files(
    State(state): State<Arc<AppState>>,
    Authorized(member_id): Authorized,
    Json(req): Json<TriggerSyncRequest>,
) -> Json<TriggerSyncResponse> {
    // 确定路径和任务类型
    let path = req
        .path
        .unwrap_or_else(|| state.config.storage.volume.clone());
    let task_type = req.task_type.as_deref().unwrap_or("sync_files");

    // 创建任务负载
    let payload = services::TaskPayload {
        task_type: task_type.to_string(),
        member_id,
        path,
        options: Some(services::TaskOptions {
            recursive: Some(true),
            file_types: None,
            include_hidden: Some(false),
        }),
        task_message_id: None,
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

/// 获取任务列表 - 需要认证
pub async fn list_tasks(
    State(state): State<Arc<AppState>>,
    Authorized(_member_id): Authorized,
) -> Json<TaskListResponse> {
    use sea_orm::{EntityTrait, QuerySelect};
    use store::entity::task_messages::{Column, Entity as TaskMessages};

    let db = &state.conn;

    // 查询最近的任务，按创建时间倒序
    let tasks = TaskMessages::find()
        .order_by_desc(Column::CreatedAt)
        .limit(50)
        .all(db)
        .await
        .unwrap_or_else(|_| Vec::new());

    // 转换为响应格式
    let task_items: Vec<TaskItemResponse> = tasks
        .into_iter()
        .map(|task| {
            let message =
                task.error_message
                    .clone()
                    .unwrap_or_else(|| match task.status.as_str() {
                        "pending" => "等待处理".to_string(),
                        "processing" => "处理中".to_string(),
                        "completed" => "已完成".to_string(),
                        "failed" => "处理失败".to_string(),
                        _ => "未知状态".to_string(),
                    });

            TaskItemResponse {
                id: task.id,
                task_type: task.message_type,
                status: task.status,
                progress: task.progress.clamp(0, 100),
                message,
                created_at: task.created_at,
                updated_at: task.updated_at,
                completed_at: task.completed_at,
            }
        })
        .collect();

    Json(TaskListResponse { tasks: task_items })
}

/// 获取单个任务详情 - 需要认证
pub async fn get_task(
    State(state): State<Arc<AppState>>,
    Authorized(_member_id): Authorized,
    Path(id): Path<i64>,
) -> Result<Json<TaskItemResponse>, AppError> {
    use sea_orm::EntityTrait;
    use store::entity::prelude::SyncMessages;

    let db = &state.conn;

    let task = SyncMessages::find_by_id(id)
        .one(db)
        .await
        .map_err(|_| AppError::NotFound)?;

    match task {
        Some(t) => {
            let message = t
                .error_message
                .clone()
                .unwrap_or_else(|| match t.status.as_str() {
                    "pending" => "等待处理".to_string(),
                    "processing" => "处理中".to_string(),
                    "completed" => "已完成".to_string(),
                    "failed" => "处理失败".to_string(),
                    _ => "未知状态".to_string(),
                });

            Ok(Json(TaskItemResponse {
                id: t.id,
                task_type: t.message_type,
                status: t.status,
                progress: t.progress.clamp(0, 100),
                message,
                created_at: t.created_at,
                updated_at: t.updated_at,
                completed_at: t.completed_at,
            }))
        }
        None => Err(AppError::NotFound),
    }
}

/// 同步文件信息 - 需要认证
/// 创建任务并将任务记录保存到数据库
pub async fn sync_files(
    State(state): State<Arc<AppState>>,
    Authorized(member_id): Authorized,
    Json(req): Json<SyncFilesRequest>,
) -> Json<SyncFilesResponse> {
    use sea_orm::{ActiveModelTrait, Set};
    use store::entity::task_messages::TaskStatus;
    use tracing::error;

    let db = &state.conn;
    let storage_root = state.config.storage.volume.clone();

    // 创建任务负载
    let payload = services::TaskPayload {
        task_type: "sync_files".to_string(),
        member_id,
        path: req.path.unwrap_or_else(|| storage_root.clone()),
        options: Some(services::TaskOptions {
            recursive: Some(true),
            file_types: None,
            include_hidden: Some(false),
        }),
        task_message_id: None,
    };

    // 将任务负载序列化为 JSON
    let payload_json = serde_json::to_string(&payload)
        .map_err(|e| services::ServiceError::Other(e.to_string()))
        .unwrap_or_else(|_| "{}".to_string());

    // 创建任务消息记录
    let task_message = store::entity::task_messages::ActiveModel {
        member_id: Set(member_id),
        message_type: Set("sync_files".to_string()),
        status: Set(TaskStatus::Pending.as_str().to_string()),
        progress: Set(0),
        payload: Set(payload_json),
        error_message: Set(None),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        completed_at: Set(None),
        ..Default::default()
    };

    // 保存到数据库
    let result = task_message.insert(db).await;

    match result {
        Ok(task) => {
            // 发送任务到任务队列
            let mut payload = payload;
            payload.task_message_id = Some(task.id);

            match state.sync_task_sender.send(payload).await {
                Ok(()) => Json(SyncFilesResponse {
                    success: true,
                    task_id: task.id,
                    message: "同步任务已创建".to_string(),
                }),
                Err(e) => {
                    error!("任务已创建但发送到队列失败: {}", e);
                    Json(SyncFilesResponse {
                        success: false,
                        task_id: task.id,
                        message: "任务已创建但发送到队列失败".to_string(),
                    })
                }
            }
        }
        Err(e) => {
            error!("创建同步任务失败: {}", e);
            Json(SyncFilesResponse {
                success: false,
                task_id: 0,
                message: "创建同步任务失败".to_string(),
            })
        }
    }
}
