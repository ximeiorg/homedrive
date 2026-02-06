use config::{Config, ConfigError, File, FileFormat, Format, Source};
use serde::Deserialize;

/// 应用配置结构体
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    /// 服务配置
    pub server: ServerConfig,
    /// 存储配置
    pub storage: StorageConfig,
    /// JWT 配置
    pub jwt: JwtConfig,
    /// 数据库配置
    pub database: DatabaseConfig,
}

/// 服务器配置
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    /// 服务器端口
    #[serde(default = "default_port")]
    pub port: u16,
    /// 服务器地址
    #[serde(default = "default_host")]
    pub host: String,
    /// CORS 允许来源
    #[serde(default = "default_cors")]
    pub cors_origin: String,
}

fn default_port() -> u16 {
    2300
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_cors() -> String {
    "*".to_string()
}

/// 存储配置
#[derive(Debug, Deserialize, Clone)]
pub struct StorageConfig {
    /// 存储根目录
    #[serde(default = "default_volume")]
    pub volume: String,
}

fn default_volume() -> String {
    "./Homedrive".to_string()
}

/// JWT 配置
#[derive(Debug, Deserialize, Clone)]
pub struct JwtConfig {
    /// JWT 密钥（如果未设置则自动生成）
    pub secret: Option<String>,
    /// JWT 过期时间（小时）
    #[serde(default = "default_jwt_expiry")]
    pub expiry_hours: i64,
}

fn default_jwt_expiry() -> i64 {
    24 * 7 // 7天
}

/// 数据库配置
#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    /// 数据库 URL
    #[serde(default = "default_database_url")]
    pub url: String,
}

fn default_database_url() -> String {
    "sqlite:homedrive.db?mode=rwc".to_string()
}

#[derive(Debug, Clone)]
struct MemorySource<F: Format> {
    content: String,
    format: F,
}

impl MemorySource<FileFormat> {
    pub fn new() -> Self {
        let contents = include_str!("../../../default.toml");
        Self {
            content: contents.to_string(),
            format: FileFormat::Toml,
        }
    }
}

impl<F: Format + Clone + Send + Sync + std::fmt::Debug + 'static> Source for MemorySource<F> {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new((*self).clone())
    }

    fn collect(&self) -> Result<config::Map<String, config::Value>, ConfigError> {
        let contents = self.content.as_str();

        self.format
            .parse(None, contents)
            .map_err(|cause| ConfigError::FileParse { uri: None, cause })
    }
}

impl AppConfig {
    /// 从环境变量和配置文件加载配置
    pub fn load() -> Result<Self, ConfigError> {
        // 尝试加载 .env 文件（如果存在）
        // dotenvy::dotenv() 不会在找不到 .env 时报错
        let _ = dotenvy::dotenv();

        let config = Config::builder()
            .add_source(MemorySource::new())
            // 默认配置文件
            .add_source(File::with_name("config.toml").required(false))
            // 环境变量覆盖
            .add_source(config::Environment::default().prefix("HOMEDRIVE").prefix_separator("_").separator("__"))
            .build()?;

        config.try_deserialize()
    }

    /// 获取 JWT 密钥，如果未配置则自动生成
    pub fn jwt_secret(&self) -> String {
        if let Some(secret) = &self.jwt.secret {
            secret.clone()
        } else {
            // 如果未配置，自动生成并返回
            let secret = crate::secret::generate_jwt_secret();
            secret
        }
    }
}

/// 从环境变量获取存储卷路径
pub fn get_volume_from_env() -> Option<String> {
    std::env::var("HOMEDRIVE_VOLUME").ok()
}

/// 从环境变量获取 JWT 密钥
pub fn get_jwt_secret_from_env() -> Option<String> {
    std::env::var("HOMEDRIVE_JWT_SECRET").ok()
}

/// 从环境变量获取服务器端口
pub fn get_port_from_env() -> Option<u16> {
    std::env::var("HOMEDRIVE_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
}
