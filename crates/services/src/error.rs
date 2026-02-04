use axum::{Json, response::IntoResponse};
use hyper::StatusCode;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("member not found")]
    MemberNotFound,

    #[error("username already exists")]
    UsernameExists,

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("unknown error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, ServiceError>;

impl IntoResponse for ServiceError {
    fn into_response(self) -> axum::response::Response {
        let (status, message, should_log) = match &self {
            // 业务逻辑错误 - 可以安全地返回给用户
            ServiceError::MemberNotFound => {
                (StatusCode::NOT_FOUND, "member not found".to_owned(), false)
            }
            ServiceError::UsernameExists => (
                StatusCode::CONFLICT,
                "username already exists".to_owned(),
                false,
            ),
            ServiceError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg.clone(), false),

            // 系统内部错误 - 返回通用消息，记录详细错误
            ServiceError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_owned(),
                true,
            ),
            ServiceError::Unknown => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_owned(),
                true,
            ),
        };

        // 记录内部错误的详细信息
        if should_log {
            tracing::error!("Internal service error: {:?}", self);
        }

        let body = Json(json!({
            "code": status.as_u16(),
            "message": message,
            "data": "",
        }));

        (status, body).into_response()
    }
}
