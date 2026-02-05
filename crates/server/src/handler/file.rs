use crate::auth::Auth;
use crate::error::AppError;
use crate::state::AppState;
use axum::{extract::Path, extract::Query, Extension, Json, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

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
