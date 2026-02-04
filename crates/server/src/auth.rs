use crate::error::AppError;
use axum::{extract::Request, middleware::Next, response::Response};
use serde::{Deserialize, Serialize};

/// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    pub exp: u64,
    pub iat: u64,
}

/// 认证扩展 - 从 JWT 中提取用户 ID
#[derive(Clone, Debug)]
pub struct Auth(pub i64);

/// 从请求头中提取 JWT token
fn extract_token_from_header(auth_header: &str) -> Option<&str> {
    if let Some(token) = auth_header.strip_prefix("Bearer ") {
        Some(token)
    } else {
        None
    }
}

/// JWT 认证中间件
pub async fn auth_middleware(req: Request, next: Next) -> Result<Response, AppError> {
    let token = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|auth_header| extract_token_from_header(auth_header))
        .ok_or(AppError::InvalidCredentials)?;

    // 使用 services 的全局密钥
    let secret = services::get_jwt_secret();
    let decoding_key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());

    let validation = jsonwebtoken::Validation::default();

    let token_data: jsonwebtoken::TokenData<JwtClaims> =
        jsonwebtoken::decode(token, &decoding_key, &validation)
            .map_err(|_| AppError::InvalidCredentials)?;

    // 将用户 ID 存入请求扩展
    let mut req = req;
    req.extensions_mut().insert(Auth(token_data.claims.sub));

    Ok(next.run(req).await)
}

/// 从请求中获取当前用户 ID
pub fn get_current_user_id(req: &Request) -> Option<i64> {
    req.extensions().get::<Auth>().map(|a| a.0)
}
