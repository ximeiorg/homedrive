use crate::auth::Auth;
use crate::state::AppState;
use axum::{extract::Query, Extension, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
