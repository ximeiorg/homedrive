use rand::RngCore;
use std::fs;

/// 获取 JWT 密钥文件路径 (~/.homedrive/.jwt_secret)
fn get_secret_file_path() -> std::path::PathBuf {
    let mut path = std::env::home_dir().unwrap_or(std::path::PathBuf::from("."));
    path.push(".homedrive");
    fs::create_dir_all(&path).ok();
    path.push(".jwt_secret");
    path
}

/// 加载 JWT 密钥
/// 如果环境变量设置了 JWT_SECRET，则使用环境变量
/// 否则从文件读取，如果文件不存在则生成新密钥并保存
pub fn load_jwt_secret() -> String {
    // 1. 首先检查环境变量
    if let Ok(secret) = std::env::var("HOMEDRIVE_JWT_SECRET")
        && !secret.is_empty()
    {
        tracing::info!("Using JWT secret from environment variable");
        return secret;
    }

    // 2. 获取密钥文件路径
    let secret_path = get_secret_file_path();

    // 检查密钥文件
    if secret_path.exists() {
        match fs::read_to_string(&secret_path) {
            Ok(secret) if !secret.is_empty() => {
                tracing::info!("Loaded JWT secret from file");
                return secret.trim().to_string();
            }
            _ => {
                tracing::warn!("Invalid secret file, regenerating...");
            }
        }
    }

    // 3. 生成新密钥并保存
    let secret = generate_jwt_secret();
    if let Err(e) = fs::write(&secret_path, &secret) {
        tracing::error!("Failed to save JWT secret: {:?}", e);
    } else {
        tracing::info!("Generated new JWT secret and saved to {:?}", secret_path);
    }

    secret
}

/// 生成随机密钥（32字节 base64编码）
pub fn generate_jwt_secret() -> String {
    let mut rng = rand::thread_rng();
    let mut key = [0u8; 32];
    rng.fill_bytes(&mut key);
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, key)
}
