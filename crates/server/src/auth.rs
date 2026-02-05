use crate::error::AppError;
use axum::{
    extract::{FromRequestParts, Request},
    http::request::Parts,
    middleware::Next,
    response::Response,
    RequestPartsExt, 
};
use jsonwebtoken::{decode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tracing::error;

/// JWT Claims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtClaims {
    pub sub: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    pub exp: u64,
    pub iat: u64,
}

/// JWT 密钥管理
pub(crate) static JWT_SECRET_KEY: Lazy<(EncodingKey, DecodingKey)> = Lazy::new(|| {
    let secret = services::get_jwt_secret();
    let secret = secret.as_bytes();
    (EncodingKey::from_secret(secret), DecodingKey::from_secret(secret))
});

/// 从请求中获取当前用户 ID
pub fn get_current_user_id(req: &Request) -> Option<i64> {
    req.extensions().get::<JwtClaims>().map(|c| c.sub)
}


impl<S> FromRequestParts<S> for JwtClaims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token = parts
        .headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::InvalidCredentials)?;

    let decoding_key = &JWT_SECRET_KEY.1;

    // 验证 audience（与编码时一致）
    let mut validation = Validation::default();
    validation.set_audience(&["homedrive"]);

    let token_data: jsonwebtoken::TokenData<JwtClaims> =
        decode(token, decoding_key, &validation).map_err(|e| {
            error!("JWT decode error: {:?}", e);
            AppError::InvalidCredentials
        })?;


        Ok(token_data.claims)
    }
}
