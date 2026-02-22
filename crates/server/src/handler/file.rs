use crate::auth::Authorized;
use crate::error::AppError;
use crate::extract::{ValidatedJson, ValidatedQuery};
use crate::state::AppState;
use axum::{Json, extract::Path, extract::State, response::IntoResponse};
use chrono::Utc;
use futures::TryStreamExt;
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

/// 允许上传的文件类型白名单
const ALLOWED_MIME_TYPES: &[&str] = &[
    // 图片
    "image/jpeg",
    "image/png",
    "image/gif",
    "image/webp",
    "image/svg+xml",
    "image/bmp",
    "image/tiff",
    // 视频
    "video/mp4",
    "video/mpeg",
    "video/quicktime",
    "video/x-msvideo",
    "video/x-ms-wmv",
    "video/webm",
    // 音频
    "audio/mpeg",
    "audio/wav",
    "audio/ogg",
    "audio/aac",
    "audio/flac",
    // 文档
    "application/pdf",
    "application/msword",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "application/vnd.ms-excel",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    "application/vnd.ms-powerpoint",
    "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    "text/plain",
    "text/csv",
    "text/markdown",
    // 压缩文件
    "application/zip",
    "application/x-rar-compressed",
    "application/x-7z-compressed",
    "application/x-tar",
    "application/gzip",
    // 其他
    "application/json",
    "application/xml",
];

/// 危险文件扩展名黑名单
const DANGEROUS_EXTENSIONS: &[&str] = &[
    "exe", "bat", "cmd", "com", "pif", "scr", "vbs", "js", "jar", "msi", "dll", "sh", "bash",
    "zsh", "fish", "php", "asp", "aspx", "jsp", "cgi", "pl", "py", "html", "htm", "xhtml", "shtml",
];

/// 检查 MIME 类型是否在白名单中
fn is_mime_type_allowed(mime_type: &str) -> bool {
    // 允许所有以 image/, video/, audio/ 开头的类型
    if mime_type.starts_with("image/")
        || mime_type.starts_with("video/")
        || mime_type.starts_with("audio/")
    {
        return true;
    }

    // 检查白名单
    ALLOWED_MIME_TYPES.contains(&mime_type)
}

/// 检查文件扩展名是否危险
fn is_dangerous_extension(filename: &str) -> bool {
    let filename_lower = filename.to_lowercase();
    for ext in DANGEROUS_EXTENSIONS {
        if filename_lower.ends_with(&format!(".{ext}")) {
            return true;
        }
    }
    false
}

/// 检查路径是否安全（允许绝对路径和相对路径）
/// 绝对路径必须不能包含路径遍历攻击
fn is_path_safe(path: &str) -> bool {
    // 不允许空路径
    if path.is_empty() {
        return false;
    }

    // 不允许路径遍历
    if path.contains("..") {
        return false;
    }

    // 允许绝对路径（如 /home/user/files）
    // 允许相对路径（如 uploads/, subdir/files）

    // 不允许 Windows 风格的绝对路径（如 C:\）
    if path.len() > 2 && path.chars().nth(1) == Some(':') {
        return false;
    }

    // 不允许空字节注入
    if path.contains('\0') {
        return false;
    }

    true
}

/// 检查文件名是否安全
fn is_filename_safe(filename: &str) -> bool {
    // 不允许空文件名
    if filename.is_empty() {
        return false;
    }

    // 不允许路径遍历
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return false;
    }

    // 不允许空字节注入
    if filename.contains('\0') {
        return false;
    }

    // 限制文件名长度
    if filename.len() > 255 {
        return false;
    }

    true
}

/// Check if a file hash already exists in the database
pub async fn check_file_hash_exists(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(query): ValidatedQuery<HashCheckQuery>,
) -> crate::error::Result<Json<HashCheckResponse>> {
    let db = &state.conn;

    let exists = services::FileService::check_hash_exists(db, &query.hash)
        .await
        .unwrap_or(false);

    Ok(Json(HashCheckResponse { exists }))
}

/// Upload file handler with streaming support for large files - requires authentication
///
/// Multipart form fields:
/// - file: (required) file content
///
/// Hash is automatically calculated during upload using xxh3 algorithm.
pub async fn upload_file(
    State(state): State<Arc<AppState>>,
    auth: Authorized,
    mut multipart: axum::extract::Multipart,
) -> crate::error::Result<Json<UploadFileResponse>> {
    let db = &state.conn;
    let uploader_id = auth.0;
    let storage_root = state.config.storage.volume.clone();

    tracing::info!(user_id = uploader_id, "Upload request received");

    // 获取用户的 storage_tag
    let user = match store::member::query::Query::find_by_id(db, uploader_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Ok(Json(UploadFileResponse {
                success: false,
                file_id: 0,
                message: "User not found".to_string(),
            }));
        }
        Err(e) => {
            tracing::error!(error = ?e, "Database error while finding user");
            return Ok(Json(UploadFileResponse {
                success: false,
                file_id: 0,
                message: "Database error".to_string(),
            }));
        }
    };
    let storage_tag = user.storage_tag;
    tracing::debug!(user_id = uploader_id, storage_tag = %storage_tag, "User storage_tag resolved");

    // 遍历 multipart 字段，找到文件
    loop {
        let field_result = multipart.next_field().await;

        match field_result {
            Ok(Some(field)) => {
                let field_name = field.name().unwrap_or("").to_string();
                tracing::debug!(field_name = %field_name, "Received multipart field");

                if field_name == "file" {
                    // 找到文件字段，处理文件上传
                    let filename = field.file_name().unwrap_or("unknown").to_string();

                    // 验证文件名安全性
                    if !is_filename_safe(&filename) {
                        tracing::warn!(filename = %filename, "Invalid filename rejected");
                        return Ok(Json(UploadFileResponse {
                            success: false,
                            file_id: 0,
                            message: "Invalid filename".to_string(),
                        }));
                    }

                    // 检查危险扩展名
                    if is_dangerous_extension(&filename) {
                        tracing::warn!(filename = %filename, user_id = uploader_id, "Dangerous file extension rejected");
                        return Ok(Json(UploadFileResponse {
                            success: false,
                            file_id: 0,
                            message: "File type not allowed for security reasons".to_string(),
                        }));
                    }

                    let content_type = field
                        .content_type()
                        .unwrap_or("application/octet-stream")
                        .to_string();

                    // 检查 MIME 类型白名单
                    if !is_mime_type_allowed(&content_type) {
                        tracing::warn!(
                            filename = %filename,
                            content_type = %content_type,
                            user_id = uploader_id,
                            "MIME type not in whitelist"
                        );
                        return Ok(Json(UploadFileResponse {
                            success: false,
                            file_id: 0,
                            message: "File type not allowed".to_string(),
                        }));
                    }

                    tracing::info!(filename = %filename, content_type = %content_type, "Processing file upload");

                    // 将 field 转换为流
                    let stream = field.map_err(std::io::Error::other);

                    // 调用 service 层处理上传，传入 storage_tag
                    match services::FileService::upload_file_stream(
                        db,
                        &storage_root,
                        &storage_tag,
                        stream,
                        content_type,
                        filename,
                        uploader_id,
                    )
                    .await
                    {
                        Ok((file_id, message)) => {
                            tracing::info!(
                                file_id = file_id,
                                user_id = uploader_id,
                                "File uploaded successfully"
                            );
                            return Ok(Json(UploadFileResponse {
                                success: true,
                                file_id,
                                message,
                            }));
                        }
                        Err(e) => {
                            tracing::error!(error = ?e, user_id = uploader_id, "File upload failed");
                            return Ok(Json(UploadFileResponse {
                                success: false,
                                file_id: 0,
                                message: "Upload failed".to_string(),
                            }));
                        }
                    }
                }
            }
            Ok(None) => {
                tracing::debug!("No more multipart fields");
                break;
            }
            Err(e) => {
                tracing::error!(error = ?e, "Error parsing multipart field");
                return Ok(Json(UploadFileResponse {
                    success: false,
                    file_id: 0,
                    message: "Error parsing request".to_string(),
                }));
            }
        }
    }

    tracing::warn!(user_id = uploader_id, "No file provided in upload request");
    Ok(Json(UploadFileResponse {
        success: false,
        file_id: 0,
        message: "No file provided".to_string(),
    }))
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

    // 验证 storage_tag 安全性
    if !is_path_safe(&storage_tag) {
        tracing::warn!(user_id = user_id, storage_tag = %storage_tag, "Invalid storage_tag rejected");
        return Err(AppError::InvalidInput("Invalid storage tag".to_string()));
    }

    // 验证 file_path 安全性
    if !is_path_safe(&file_path) {
        tracing::warn!(user_id = user_id, file_path = %file_path, "Invalid file_path rejected");
        return Err(AppError::InvalidInput("Invalid file path".to_string()));
    }

    // Get user's storage_tag from database
    let user = match store::member::query::Query::find_by_id(db, user_id).await {
        Ok(Some(m)) => m,
        Ok(None) => return Err(AppError::InvalidCredentials),
        Err(_) => return Err(AppError::InvalidCredentials),
    };

    // Verify the requested storage_tag matches the user's storage_tag
    if storage_tag != user.storage_tag {
        tracing::warn!(
            user_id = user_id,
            requested_storage_tag = %storage_tag,
            user_storage_tag = %user.storage_tag,
            "Storage tag mismatch - potential unauthorized access attempt"
        );
        return Err(AppError::Forbidden);
    }

    // Get storage root from config
    let storage_root = &state.config.storage.volume;

    // Build the file path following LocalStorage's directory structure
    // LocalStorage uses: root/{storage_tag}/{file_path}
    // storage_path already contains: storage_tag/file_path
    let mut file_path_buf = PathBuf::from(storage_root);
    file_path_buf.push(&storage_tag);
    file_path_buf.push(&file_path);

    println!("{file_path_buf:?}");

    // 规范化路径并检查是否仍在允许的目录内
    let canonical_path = match file_path_buf.canonicalize() {
        Ok(p) => p,
        Err(_) => return Err(AppError::NotFound),
    };

    let canonical_root = match PathBuf::from(storage_root).canonicalize() {
        Ok(p) => p,
        Err(_) => return Err(AppError::NotFound),
    };

    // 确保解析后的路径仍在存储根目录内
    if !canonical_path.starts_with(&canonical_root) {
        tracing::warn!(
            user_id = user_id,
            attempted_path = ?canonical_path,
            "Path traversal attempt detected"
        );
        return Err(AppError::Forbidden);
    }

    // Check if file exists
    if !canonical_path.exists() {
        return Err(AppError::NotFound);
    }

    // Get file metadata
    let metadata = match tokio::fs::metadata(&canonical_path).await {
        Ok(m) => m,
        Err(_) => return Err(AppError::NotFound),
    };

    if !metadata.is_file() {
        return Err(AppError::NotFound);
    }

    let file_size = metadata.len();
    let content_type = mime_guess::from_path(&canonical_path)
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
            let file = match File::open(&canonical_path).await {
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
                    format!("bytes {start}-{end}/{file_size}"),
                ),
            ];
            (
                axum::http::StatusCode::PARTIAL_CONTENT,
                headers,
                axum::body::Body::from_stream(stream),
            )
        } else {
            // Invalid range, return full file
            let file = match File::open(&canonical_path).await {
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
        let file = match File::open(&canonical_path).await {
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
    auth: Authorized,
    ValidatedQuery(query): ValidatedQuery<ListFilesQuery>,
) -> crate::error::Result<Json<schema::file::FileListResponse>> {
    use schema::file::FileListItem;

    let member_id = auth.user_id();

    // 通过 services 层查询文件列表
    let params = services::ListMemberFilesParams {
        page: query.page,
        page_size: query.page_size,
        sort_by: query.sort_by,
        sort_order: query.sort_order,
        file_type: query.file_type,
        search: query.search,
    };

    let (results, total) = services::FileService::list_member_files(&state.conn, member_id, params)
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
                Some(format!("{base_url}/api/static/{storage_path}"))
            };

            // 构建缩略图 URL
            let thumbnail_url = thumbnail.and_then(|t| {
                if t.is_empty() {
                    None
                } else {
                    Some(format!("{base_url}/api/static/{t}"))
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
                created_at: schema::file::format_datetime_local(member_file.created_at),
                updated_at: schema::file::format_datetime_local(member_file.updated_at),
            }
        })
        .collect();

    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(100);
    let total_pages = (total as f64 / page_size as f64).ceil() as u64;

    Ok(Json(schema::file::FileListResponse {
        files,
        total,
        page,
        page_size,
        total_pages,
    }))
}

/// 触发同步任务 - 需要认证
pub async fn trigger_sync_files(
    State(state): State<Arc<AppState>>,
    auth: Authorized,
    ValidatedJson(req): ValidatedJson<TriggerSyncRequest>,
) -> crate::error::Result<Json<TriggerSyncResponse>> {
    let member_id = auth.user_id();

    // 确定路径和任务类型
    let path = req
        .path
        .unwrap_or_else(|| state.config.storage.volume.clone());

    // 验证路径安全性
    if !is_path_safe(&path) {
        tracing::warn!(user_id = member_id, path = %path, "Invalid sync path rejected");
        return Ok(Json(TriggerSyncResponse {
            success: false,
            task_id: 0,
            message: "Invalid path".to_string(),
        }));
    }

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
        Ok(()) => Ok(Json(TriggerSyncResponse {
            success: true,
            task_id: 0,
            message: "Task queued successfully".to_string(),
        })),
        Err(e) => {
            tracing::error!(error = ?e, user_id = member_id, "Failed to queue task");
            Ok(Json(TriggerSyncResponse {
                success: false,
                task_id: 0,
                message: "Failed to queue task".to_string(),
            }))
        }
    }
}

/// 获取任务列表 - 需要认证
pub async fn list_tasks(
    State(state): State<Arc<AppState>>,
    _auth: Authorized,
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
                progress: task.progress.clamp(0i64, 100i64),
                message,
                created_at: schema::file::format_datetime_local(task.created_at),
                updated_at: schema::file::format_datetime_local(task.updated_at),
                completed_at: task.completed_at.map(schema::file::format_datetime_local),
            }
        })
        .collect();

    Json(TaskListResponse { tasks: task_items })
}

/// 获取单个任务详情 - 需要认证
pub async fn get_task(
    State(state): State<Arc<AppState>>,
    _auth: Authorized,
    Path(id): Path<i64>,
) -> Result<Json<TaskItemResponse>, AppError> {
    use sea_orm::EntityTrait;
    use store::entity::prelude::TaskMessages;

    let db = &state.conn;

    let task = TaskMessages::find_by_id(id)
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
                created_at: schema::file::format_datetime_local(t.created_at),
                updated_at: schema::file::format_datetime_local(t.updated_at),
                completed_at: t.completed_at.map(schema::file::format_datetime_local),
            }))
        }
        None => Err(AppError::NotFound),
    }
}

/// 同步文件信息 - 需要认证
/// 创建任务并将任务记录保存到数据库
pub async fn sync_files(
    State(state): State<Arc<AppState>>,
    auth: Authorized,
    ValidatedJson(req): ValidatedJson<SyncFilesRequest>,
) -> crate::error::Result<Json<SyncFilesResponse>> {
    use sea_orm::{ActiveModelTrait, Set};
    use store::entity::task_messages::TaskStatus;

    let member_id = auth.user_id();
    let db = &state.conn;
    let storage_root = state.config.storage.volume.clone();

    // 获取路径并验证安全性
    let sync_path = req.path.unwrap_or_else(|| storage_root.clone());
    if !is_path_safe(&sync_path) {
        tracing::warn!(user_id = member_id, path = %sync_path, "Invalid sync path rejected");
        return Ok(Json(SyncFilesResponse {
            success: false,
            task_id: 0,
            message: "Invalid path".to_string(),
        }));
    }

    // 创建任务负载
    let payload = services::TaskPayload {
        task_type: "sync_files".to_string(),
        member_id,
        path: sync_path,
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
                Ok(()) => Ok(Json(SyncFilesResponse {
                    success: true,
                    task_id: task.id,
                    message: "同步任务已创建".to_string(),
                })),
                Err(e) => {
                    tracing::error!(error = ?e, task_id = task.id, "Task created but failed to send to queue");
                    Ok(Json(SyncFilesResponse {
                        success: false,
                        task_id: task.id,
                        message: "任务已创建但发送到队列失败".to_string(),
                    }))
                }
            }
        }
        Err(e) => {
            tracing::error!(error = ?e, user_id = member_id, "Failed to create sync task");
            Ok(Json(SyncFilesResponse {
                success: false,
                task_id: 0,
                message: "创建同步任务失败".to_string(),
            }))
        }
    }
}

/// 触发视频缩略图生成任务 - 需要认证
pub async fn trigger_thumbnail_generation(
    State(state): State<Arc<AppState>>,
    auth: Authorized,
) -> crate::error::Result<Json<TriggerSyncResponse>> {
    use sea_orm::{ActiveModelTrait, Set};
    use store::entity::task_messages::TaskStatus;

    let member_id = auth.user_id();
    let db = &state.conn;
    let storage_root = state.config.storage.volume.clone();

    // 创建任务负载
    let payload = services::TaskPayload {
        task_type: "generate_thumbnail".to_string(),
        member_id,
        path: storage_root,
        options: None,
        task_message_id: None,
    };

    // 将任务负载序列化为 JSON
    let payload_json = serde_json::to_string(&payload)
        .map_err(|e| services::ServiceError::Other(e.to_string()))
        .unwrap_or_else(|_| "{}".to_string());

    // 创建任务消息记录
    let task_message = store::entity::task_messages::ActiveModel {
        member_id: Set(member_id),
        message_type: Set("generate_thumbnail".to_string()),
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
                Ok(()) => Ok(Json(TriggerSyncResponse {
                    success: true,
                    task_id: task.id,
                    message: "缩略图生成任务已创建".to_string(),
                })),
                Err(e) => {
                    tracing::error!(error = ?e, task_id = task.id, "Task created but failed to send to queue");
                    Ok(Json(TriggerSyncResponse {
                        success: false,
                        task_id: task.id,
                        message: "任务已创建但发送到队列失败".to_string(),
                    }))
                }
            }
        }
        Err(e) => {
            tracing::error!(error = ?e, user_id = member_id, "Failed to create thumbnail task");
            Ok(Json(TriggerSyncResponse {
                success: false,
                task_id: 0,
                message: "创建缩略图生成任务失败".to_string(),
            }))
        }
    }
}
