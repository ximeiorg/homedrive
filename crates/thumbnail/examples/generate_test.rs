//! 测试本地 ffmpeg 缩略图生成

use thumbnail::{generate_thumbnail, ThumbnailConfig};

#[tokio::main]
async fn main() {
    let video_path = "uploads/test/VID20260122221850.mp4";
    let output_path = "uploads/test/thumbnail.jpg";
    
    println!("测试视频缩略图生成 (本地 ffmpeg)...");
    println!("输入文件: {}", video_path);
    println!("输出文件: {}", output_path);
    
    let config = ThumbnailConfig::default();
    
    match generate_thumbnail(video_path, output_path, config).await {
        Ok(_) => {
            println!("✓ 缩略图生成成功!");
            println!("输出文件: {}", output_path);
        }
        Err(e) => {
            eprintln!("✗ 缩略图生成失败: {}", e);
            std::process::exit(1);
        }
    }
}
