use thiserror::Error;

/// 错误类别 - 用于区分错误处理方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// 业务逻辑错误 - 可以安全地返回给用户
    Business,
    /// 系统内部错误 - 需要记录日志，返回通用消息
    System,
}

#[derive(Debug, Error)]
pub enum ServiceError {
    // ===== 业务错误 (Business Errors) =====
    /// 会员不存在
    #[error("member not found")]
    MemberNotFound,

    /// 用户名已存在
    #[error("username already exists")]
    UsernameExists,

    /// 凭证无效
    #[error("invalid credentials")]
    InvalidCredentials,

    /// 输入参数无效
    #[error("invalid input: {0}")]
    InvalidInput(String),

    // ===== 系统错误 (System Errors) =====
    /// 数据库错误
    #[error("database error: {0}")]
    Database(#[from] store::DbErr),

    /// 存储错误
    #[error("storage error: {0}")]
    Storage(String),

    /// 文件不存在
    #[error("file not found")]
    FileNotFound,

    /// 未知错误
    #[error("unknown error")]
    Unknown,

    /// 其他错误
    #[error("other error: {0}")]
    Other(String),
}

impl ServiceError {
    /// 获取错误的类别
    #[inline]
    pub fn category(&self) -> ErrorCategory {
        match self {
            // 业务错误可以直接返回给用户
            Self::MemberNotFound | Self::UsernameExists | Self::InvalidCredentials | Self::InvalidInput(_) | Self::FileNotFound => ErrorCategory::Business,
            // 系统错误需要记录日志
            Self::Database(_) | Self::Storage(_) | Self::Unknown | Self::Other(_) => ErrorCategory::System,
        }
    }

    /// 判断是否为业务错误
    #[inline]
    pub fn is_business_error(&self) -> bool {
        matches!(self, Self::MemberNotFound | Self::UsernameExists | Self::InvalidCredentials | Self::InvalidInput(_) | Self::FileNotFound)
    }

    /// 判断是否为系统错误
    #[inline]
    pub fn is_system_error(&self) -> bool {
        matches!(self, Self::Database(_) | Self::Storage(_) | Self::Unknown | Self::Other(_))
    }
}

pub type Result<T> = std::result::Result<T, ServiceError>;
