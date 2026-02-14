//! 自定义请求提取器
//!
//! 提供自动验证的提取器，使 handler 代码更简洁

use axum::{
    Json,
    extract::{FromRequest, Request, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::de::DeserializeOwned;
use thiserror::Error;
use validator::Validate;

/// 自动验证的 JSON 提取器
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}

/// 自动验证的 Query 提取器
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedQuery<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let axum::extract::Query(value) =
            axum::extract::Query::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedQuery(value))
    }
}

/// 服务器错误类型
#[derive(Debug, Error)]
pub enum ServerError {
    /// 验证错误
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),

    /// JSON 解析错误
    #[error(transparent)]
    JsonRejection(#[from] JsonRejection),

    /// Query 解析错误
    #[error("Failed to parse query parameters: {0}")]
    QueryRejection(String),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            ServerError::ValidationError(errors) => {
                let error_messages: Vec<String> = errors
                    .field_errors()
                    .into_iter()
                    .flat_map(|(field, errs)| {
                        errs.iter().map(move |e| {
                            format!(
                                "{}: {}",
                                field,
                                e.message
                                    .as_ref()
                                    .map(|m| m.to_string())
                                    .unwrap_or_else(|| "无效值".to_string())
                            )
                        })
                    })
                    .collect();

                let message = format!("验证失败: {}", error_messages.join("; "));
                (
                    StatusCode::BAD_REQUEST,
                    crate::error::AppError::ValidationError(message),
                )
                    .into_response()
            }
            ServerError::JsonRejection(rejection) => (
                StatusCode::BAD_REQUEST,
                crate::error::AppError::InvalidInput(rejection.body_text()),
            )
                .into_response(),
            ServerError::QueryRejection(msg) => (
                StatusCode::BAD_REQUEST,
                crate::error::AppError::InvalidInput(msg),
            )
                .into_response(),
        }
    }
}

impl From<axum::extract::rejection::QueryRejection> for ServerError {
    fn from(rejection: axum::extract::rejection::QueryRejection) -> Self {
        ServerError::QueryRejection(rejection.body_text())
    }
}
