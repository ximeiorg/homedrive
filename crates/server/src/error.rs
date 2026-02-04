use axum::{Json, response::IntoResponse};
use hyper::StatusCode;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    // 业务逻辑错误 - 可以安全地返回给用户
    #[error("member not found")]
    MemberNotFound,

    #[error("member already exists")]
    MemberAlreadyExists,

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("invalid input: {0}")]
    InvalidInput(String),

    // 系统内部错误 - 返回通用消息，记录详细错误
    #[error("database error")]
    DatabaseError,

    #[error("service error")]
    ServiceError(#[from] services::ServiceError),

    #[error("unknown error")]
    Unknown,
}

impl AppError {
    /// 获取业务友好的错误消息（不暴露内部细节）
    pub fn user_message(&self) -> String {
        match self {
            Self::MemberNotFound => "member not found".into(),
            Self::MemberAlreadyExists => "member already exists".into(),
            Self::InvalidCredentials => "invalid credentials".into(),
            Self::InvalidInput(msg) => msg.clone(),
            // 系统错误只返回通用消息
            Self::DatabaseError | Self::Unknown | Self::ServiceError(_) => "internal server error".into(),
        }
    }

    /// 判断是否需要记录日志
    pub fn should_log(&self) -> bool {
        match self {
            // 业务错误通常不需要记录 error 级别日志
            Self::MemberNotFound | Self::MemberAlreadyExists | Self::InvalidCredentials => false,
            // 系统错误需要记录
            Self::InvalidInput(_) => false, // 也可以设为 true，取决于需求
            Self::DatabaseError | Self::Unknown | Self::ServiceError(_) => true,
        }
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        // 记录内部错误的详细信息
        if self.should_log() {
            tracing::error!(
                error = ?self,
                category = "system",
                "Internal server error"
            );
        }

        let status = match &self {
            // 业务逻辑错误 - 返回对应的 HTTP 状态码
            Self::MemberNotFound => StatusCode::NOT_FOUND,
            Self::MemberAlreadyExists => StatusCode::CONFLICT,
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::InvalidInput(_) => StatusCode::BAD_REQUEST,
            
            // 系统内部错误 - 返回 500
            Self::DatabaseError | Self::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ServiceError(e) => {
                match e.category() {
                    services::ErrorCategory::Business => {
                        // 业务错误使用对应的状态码
                        match e {
                            services::ServiceError::MemberNotFound => StatusCode::NOT_FOUND,
                            services::ServiceError::UsernameExists => StatusCode::CONFLICT,
                            services::ServiceError::InvalidCredentials => StatusCode::UNAUTHORIZED,
                            services::ServiceError::InvalidInput(_) => StatusCode::BAD_REQUEST,
                            _ => StatusCode::INTERNAL_SERVER_ERROR,
                        }
                    }
                    services::ErrorCategory::System => StatusCode::INTERNAL_SERVER_ERROR,
                }
            }
        };

        let body = Json(json!({
            "code": status.as_u16(),
            "message": self.user_message(),
            "data": serde_json::Value::Null,
        }));

        (status, body).into_response()
    }
}
