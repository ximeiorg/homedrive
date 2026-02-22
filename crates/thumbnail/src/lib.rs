//! 视频缩略图生成模块
//!
//! 通过调用系统 ffmpeg 生成视频的缩略图（第一帧）

use std::path::Path;
use std::process::Stdio;
use thiserror::Error;
use tokio::fs;
use tokio::process::Command;

/// 生成视频缩略图时发生的错误
#[derive(Debug, Error)]
pub enum ThumbnailError {
    /// ffmpeg 命令执行失败
    #[error("ffmpeg 执行失败: {0}")]
    FfmpegFailed(String),

    /// 输入文件不存在
    #[error("输入文件不存在: {0}")]
    InputFileNotFound(String),

    /// IO 错误
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
}

/// 视频缩略图生成器配置
#[derive(Debug, Clone)]
pub struct ThumbnailConfig {
    /// 缩略图宽度
    pub width: Option<u32>,
    /// 缩略图高度
    pub height: Option<u32>,
    /// 缩略图质量 (1-31, 越小越好)
    pub quality: u32,
    /// 输出格式
    pub format: ImageFormat,
}

impl Default for ThumbnailConfig {
    fn default() -> Self {
        Self {
            width: Some(320),
            height: None,
            quality: 2,
            format: ImageFormat::Jpeg,
        }
    }
}

/// 支持的图片格式
#[derive(Debug, Clone, Copy)]
pub enum ImageFormat {
    Jpeg,
    Png,
}

impl ImageFormat {
    pub fn extension(&self) -> &str {
        match self {
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Png => "png",
        }
    }
}

/// 生成视频缩略图
///
/// # 参数
/// - `video_path`: 视频文件路径
/// - `output_path`: 缩略图输出路径
/// - `config`: 生成配置
///
/// # 示例
/// ```ignore
/// use thumbnail::{generate_thumbnail, ThumbnailConfig};
///
/// async fn main() {
///     let config = ThumbnailConfig::default();
///     generate_thumbnail("/path/to/video.mp4", "/path/to/thumbnail.jpg", config).await;
/// }
/// ```
pub async fn generate_thumbnail(
    video_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    config: ThumbnailConfig,
) -> Result<(), ThumbnailError> {
    let video_path = video_path.as_ref();
    let output_path = output_path.as_ref();

    // 检查输入文件是否存在
    if !video_path.exists() {
        return Err(ThumbnailError::InputFileNotFound(
            video_path.display().to_string(),
        ));
    }

    // 确保输出目录存在
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    // 构建 ffmpeg 命令
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y") // 覆盖输出文件（如果存在）
        .arg("-i")
        .arg(video_path) // 输入文件
        .arg("-ss")
        .arg("00:00:00") // 从第一帧开始
        .arg("-vframes")
        .arg("1"); // 只取一帧

    // 添加视频缩放参数
    if let Some(width) = config.width {
        cmd.arg("-vf").arg(format!("scale={width}:-1"));
    } else if let Some(height) = config.height {
        cmd.arg("-vf").arg(format!("scale=-1:{height}"));
    }

    // 添加质量参数
    let qscale = match config.format {
        ImageFormat::Jpeg => ["-q:v", &config.quality.to_string()],
        ImageFormat::Png => ["-compression_level", &config.quality.to_string()],
    };
    cmd.arg(qscale[0]).arg(qscale[1]);

    // 设置输出路径和格式
    cmd.arg(output_path);

    // 执行命令
    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| ThumbnailError::FfmpegFailed(e.to_string()))?;

    if output.status.success() {
        tracing::info!("成功生成缩略图: {}", output_path.display());
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!("ffmpeg 错误: {}", stderr);
        Err(ThumbnailError::FfmpegFailed(stderr.to_string()))
    }
}

/// 检查系统是否安装了 ffmpeg
pub async fn check_ffmpeg() -> Result<bool, ThumbnailError> {
    let output = Command::new("ffmpeg").arg("-version").output().await;

    Ok(output.is_ok())
}

/// 生成视频缩略图的便捷函数
///
/// 使用默认配置生成缩略图
pub async fn generate_thumbnail_default(
    video_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<(), ThumbnailError> {
    generate_thumbnail(video_path, output_path, ThumbnailConfig::default()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ThumbnailConfig::default();
        assert_eq!(config.width, Some(320));
        assert_eq!(config.quality, 2);
    }

    #[test]
    fn test_image_format_extension() {
        assert_eq!(ImageFormat::Jpeg.extension(), "jpg");
        assert_eq!(ImageFormat::Png.extension(), "png");
    }
}
