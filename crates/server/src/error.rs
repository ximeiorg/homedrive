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

    #[error("forbidden")]
    Forbidden,

    #[error("not found")]
    NotFound,

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("validation error: {0}")]
    ValidationError(String),

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
            Self::Forbidden => "forbidden".into(),
            Self::NotFound => "not found".into(),
            Self::InvalidInput(msg) => msg.clone(),
            Self::ValidationError(msg) => msg.clone(),
            // 系统错误只返回通用消息
            Self::DatabaseError | Self::Unknown => {
                tracing::error!(category = "system", "Internal server error");
                "internal server error".into()
            }
            Self::ServiceError(e) => match e.category() {
                services::ErrorCategory::Business => e.to_string(),
                services::ErrorCategory::System => {
                    tracing::error!(
                        error = ?e,
                        error_message = %e,
                        category = "system",
                        "Internal server error"
                    );
                    "internal server error".into()
                }
            },
        }
    }

    /// 判断是否需要记录日志
    pub fn should_log(&self) -> bool {
        match self {
            // 业务错误通常不需要记录 error 级别日志
            Self::MemberNotFound
            | Self::MemberAlreadyExists
            | Self::InvalidCredentials
            | Self::Forbidden
            | Self::NotFound
            | Self::ValidationError(_) => false,
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

        let status = match &self {
            // 业务逻辑错误 - 返回对应的 HTTP 状态码
            Self::MemberNotFound => StatusCode::NOT_FOUND,
            Self::MemberAlreadyExists => StatusCode::CONFLICT,
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::InvalidInput(_) => StatusCode::BAD_REQUEST,
            Self::ValidationError(_) => StatusCode::BAD_REQUEST,

            // 系统内部错误 - 返回 500
            Self::DatabaseError => {
                tracing::error!(category = "system", "Database error occurred");
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Self::Unknown => {
                tracing::error!(category = "system", "Unknown error occurred");
                StatusCode::INTERNAL_SERVER_ERROR
            }
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
                    services::ErrorCategory::System => {
                        tracing::error!(
                            error = ?e,
                            error_message = %e,
                            category = "system",
                            "Service system error"
                        );
                        StatusCode::INTERNAL_SERVER_ERROR
                    }
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

/// 从 validator 验证错误创建 AppError
impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let error_messages: Vec<String> = errors
            .field_errors()
            .into_iter()
            .flat_map(|(field, errs)| {
                errs.iter().map(move |e| {
                    format!(
                        "{}: {}",
                        field,
                        e.message.as_ref().map(|m| m.to_string()).unwrap_or_else(|| "invalid value".to_string())
                    )
                })
            })
            .collect();
        
        AppError::ValidationError(error_messages.join("; "))
    }
}
