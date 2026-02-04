use crate::state::AppState;
use axum::{Extension, Json, Query};
use sea_orm::{entity::prelude::*, query::*, Condition, DatabaseConnection};
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct HashCheckResponse {
    exists: bool,
}

#[derive(Deserialize)]
pub struct HashCheckQuery {
    hash: String,
}

/// Check if a file hash already exists in the database
pub async fn check_file_hash_exists(
    Extension(state): Extension<Arc<AppState>>,
    Query(query): Query<HashCheckQuery>,
) -> Json<HashCheckResponse> {
    let db = &state.conn;
    
    // Check if the hash exists in file_contents table
    let exists = entity::file_contents::Entity::find()
        .filter(entity::file_contents::Column::Hash.eq(&query.hash))
        .one(db)
        .await
        .unwrap_or(None)
        .is_some();
    
    Json(HashCheckResponse { exists })
}

/// Upload file request
#[derive(Deserialize)]
pub struct UploadFileRequest {
    hash: String,
}

/// Upload file response
#[derive(Serialize)]
pub struct UploadFileResponse {
    success: bool,
    file_id: i32,
    message: String,
}

/// Upload file handler
pub async fn upload_file(
    Extension(state): Extension<Arc<AppState>>,
    mut multipart: axum::extract::Multipart,
) -> Json<UploadFileResponse> {
    let db = &state.conn;
    
    // Get the file from multipart
    if let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let filename = field.file_name().unwrap_or("unknown").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        let data = field.bytes().await.unwrap_or_default();
        let size = data.len() as i64;
        
        // Get the hash from form field
        let hash = multipart
            .next_field()
            .await
            .unwrap_or(None)
            .and_then(|f| f.text().ok())
            .unwrap_or_default();
        
        // Check if hash already exists
        let existing = entity::file_contents::Entity::find()
            .filter(entity::file_contents::Column::Hash.eq(&hash))
            .one(db)
            .await
            .unwrap_or(None);
        
        if existing.is_some() {
            return Json(UploadFileResponse {
                success: true,
                file_id: existing.unwrap().id,
                message: "File already exists".to_string(),
            });
        }
        
        // Create new file record
        let file_content = entity::file_contents::ActiveModel {
            hash: Set(hash),
            size: Set(size),
            mime_type: Set(content_type),
            file_name: Set(filename),
            created_at: Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        };
        
        let result = entity::file_contents::Entity::insert(file_content)
            .exec(db)
            .await
            .unwrap_or_default();
        
        // TODO: Save file data to storage (S3, local, etc.)
        // For now, just save metadata to database
        
        Json(UploadFileResponse {
            success: true,
            file_id: result.last_insert_id,
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
