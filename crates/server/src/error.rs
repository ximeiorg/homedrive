use axum::{Json, response::IntoResponse};
use hyper::StatusCode;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("member not found")]
    MemberNotFound,

    #[error("member already exists")]
    MemberAlreadyExists,

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("database error")]
    DatabaseError,

    #[error("service error")]
    ServiceError(#[from] services::ServiceError),

    #[error("unknown error")]
    Unknown,
}
pub type Result<T> = std::result::Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            // 服务层错误 - 委托给 ServiceError 处理
            AppError::ServiceError(service_err) => return service_err.into_response(),
            _ => {}
        }

        let (status, message, should_log) = match &self {
            // 业务逻辑错误 - 可以安全地返回给用户
            AppError::MemberNotFound => {
                (StatusCode::NOT_FOUND, "member not found".to_owned(), false)
            }
            AppError::MemberAlreadyExists => (
                StatusCode::CONFLICT,
                "member already exists".to_owned(),
                false,
            ),
            AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg.clone(), false),

            // 系统内部错误 - 返回通用消息，记录详细错误
            AppError::DatabaseError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_owned(),
                true,
            ),
            AppError::Unknown => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_owned(),
                true,
            ),
            AppError::ServiceError(_) => unreachable!("ServiceError handled above"),
        };

        // 记录内部错误的详细信息
        if should_log {
            tracing::error!("Internal server error: {:?}", self);
        }

        let body = Json(json!({
            "code": status.as_u16(),
            "message": message,
            "data":"",
        }));

        (status, body).into_response()
    }
}
