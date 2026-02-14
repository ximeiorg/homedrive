use crate::error::AppError;
use axum::extract::FromRequestParts;
use jsonwebtoken::{DecodingKey, EncodingKey, Validation, decode};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::error;

/// Member role enum - must match schema::member::MemberRole
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MemberRole {
    #[default]
    User,
    Admin,
}

impl MemberRole {
    pub fn is_admin(&self) -> bool {
        matches!(self, Self::Admin)
    }
}

impl From<schema::member::MemberRole> for MemberRole {
    fn from(role: schema::member::MemberRole) -> Self {
        match role {
            schema::member::MemberRole::Admin => Self::Admin,
            schema::member::MemberRole::User => Self::User,
        }
    }
}

impl From<MemberRole> for schema::member::MemberRole {
    fn from(role: MemberRole) -> Self {
        match role {
            MemberRole::Admin => Self::Admin,
            MemberRole::User => Self::User,
        }
    }
}

impl From<store::entity::members::MemberRole> for MemberRole {
    fn from(role: store::entity::members::MemberRole) -> Self {
        match role {
            store::entity::members::MemberRole::Admin => Self::Admin,
            store::entity::members::MemberRole::User => Self::User,
        }
    }
}

impl From<MemberRole> for store::entity::members::MemberRole {
    fn from(role: MemberRole) -> Self {
        match role {
            MemberRole::Admin => Self::Admin,
            MemberRole::User => Self::User,
        }
    }
}

/// JWT Claims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtClaims {
    pub sub: i64,
    #[serde(default)]
    pub role: MemberRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    pub exp: u64,
    pub iat: u64,
}

/// Auth 包装器，用于从请求中提取用户 ID
#[derive(Clone)]
pub struct Auth(pub i64);

impl From<JwtClaims> for Auth {
    fn from(claims: JwtClaims) -> Self {
        Auth(claims.sub)
    }
}

/// 已认证用户提取器
/// 自动从 JWT Claims 中提取用户 ID 和角色，handler 只需使用此类型
#[derive(Clone, Debug)]
pub struct Authorized(pub i64, pub MemberRole);

impl Authorized {
    pub fn is_admin(&self) -> bool {
        self.1.is_admin()
    }

    pub fn user_id(&self) -> i64 {
        self.0
    }

    pub fn role(&self) -> &MemberRole {
        &self.1
    }
}

impl<S> FromRequestParts<S> for Authorized
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let claims = JwtClaims::from_request_parts(parts, _state).await?;
        Ok(Authorized(claims.sub, claims.role))
    }
}

/// Admin only extractor - only allows admin users
#[derive(Clone, Debug)]
pub struct AdminOnly(pub i64);

impl<S> FromRequestParts<S> for AdminOnly
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let claims = JwtClaims::from_request_parts(parts, _state).await?;

        if !claims.role.is_admin() {
            return Err(AppError::Forbidden);
        }

        Ok(AdminOnly(claims.sub))
    }
}

/// JWT 密钥管理
pub(crate) static JWT_SECRET_KEY: Lazy<(EncodingKey, DecodingKey)> = Lazy::new(|| {
    let secret = services::get_jwt_secret();
    let secret = secret.as_bytes();
    (
        EncodingKey::from_secret(secret),
        DecodingKey::from_secret(secret),
    )
});

/// 从请求中获取当前用户 ID
pub fn get_current_user_id(req: &axum::http::Request<axum::body::Body>) -> Option<i64> {
    req.extensions().get::<JwtClaims>().map(|c| c.sub)
}

impl<S> FromRequestParts<S> for JwtClaims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // 首先尝试从 Authorization header 获取 token
        let token = parts
            .headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "));

        // 如果 header 中没有 token，尝试从 query 参数获取
        let token = if let Some(token) = token {
            token.to_string()
        } else {
            // 从 query 参数获取 token: ?token=xxx
            let query = parts.uri.query().unwrap_or("");

            query
                .split('&')
                .find(|p| p.starts_with("token="))
                .and_then(|p| p.strip_prefix("token="))
                .map(|p| urlencoding::decode(p).unwrap_or_else(|_| p.to_string()))
                .ok_or(AppError::InvalidCredentials)?
        };

        let decoding_key = &JWT_SECRET_KEY.1;

        // 验证 audience（与编码时一致）
        let mut validation = Validation::default();

        validation.set_audience(&["homedrive"]);

        let token_data: jsonwebtoken::TokenData<JwtClaims> =
            decode(&token, decoding_key, &validation).map_err(|e| {
                error!("JWT decode error: {:?}", e);
                AppError::InvalidCredentials
            })?;

        Ok(token_data.claims)
    }
}
