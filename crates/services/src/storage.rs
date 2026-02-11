use async_trait::async_trait;
use std::path::Path;
use tokio::io::AsyncWriteExt;

/// 存储服务 trait - 定义文件存储的异步接口
#[async_trait]
pub trait StorageService: Send + Sync {
    /// 保存文件
    async fn save(&self, key: &str, content: &[u8])
    -> std::result::Result<(), crate::ServiceError>;

    /// 流式保存文件（支持大文件）
    async fn save_stream(
        &self,
        key: &str,
        stream: std::pin::Pin<
            Box<
                dyn futures::Stream<Item = std::result::Result<bytes::Bytes, std::io::Error>>
                    + Send,
            >,
        >,
    ) -> std::result::Result<u64, crate::ServiceError>;

    /// 删除文件
    async fn delete(&self, key: &str) -> std::result::Result<(), crate::ServiceError>;

    /// 检查文件是否存在
    async fn exists(&self, key: &str) -> std::result::Result<bool, crate::ServiceError>;

    /// 获取文件内容
    async fn get(&self, key: &str) -> std::result::Result<Vec<u8>, crate::ServiceError>;

    /// 获取文件 URL（用于访问文件）
    async fn url(&self, key: &str) -> std::result::Result<String, crate::ServiceError>;

    /// 获取文件大小
    async fn size(&self, key: &str) -> std::result::Result<u64, crate::ServiceError>;
}

/// 存储配置
#[derive(Clone, Debug)]
pub struct StorageConfig {
    /// 存储根目录
    pub root: String,
}

/// 本地文件系统存储实现
#[derive(Clone, Debug)]
pub struct LocalStorage {
    config: StorageConfig,
}

impl LocalStorage {
    /// 创建新的本地存储实例
    pub fn new(config: StorageConfig) -> Self {
        Self { config }
    }

    /// 获取完整文件路径
    fn full_path(&self, key: &str) -> std::path::PathBuf {
        let mut path = Path::new(&self.config.root).to_path_buf();
        // 使用 key 的前两个字符作为子目录（哈希分片）
        if key.len() >= 2 {
            path.push(&key[0..2]);
        }
        path.push(key);
        path
    }
}

#[async_trait]
impl StorageService for LocalStorage {
    async fn save(
        &self,
        key: &str,
        content: &[u8],
    ) -> std::result::Result<(), crate::ServiceError> {
        let path = self.full_path(key);

        // 创建父目录
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| crate::ServiceError::Storage(e.to_string()))?;
        }

        // 写入文件
        tokio::fs::write(&path, content)
            .await
            .map_err(|e| crate::ServiceError::Storage(e.to_string()))?;

        tracing::info!("File saved: {}", key);
        Ok(())
    }

    async fn save_stream(
        &self,
        key: &str,
        mut stream: std::pin::Pin<
            Box<
                dyn futures::Stream<Item = std::result::Result<bytes::Bytes, std::io::Error>>
                    + Send,
            >,
        >,
    ) -> std::result::Result<u64, crate::ServiceError> {
        use futures::StreamExt;
        use tokio::fs::File;
        use tokio::io::BufWriter;

        let path = self.full_path(key);

        // 创建父目录
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| crate::ServiceError::Storage(e.to_string()))?;
        }

        // 创建文件
        let file = File::create(&path)
            .await
            .map_err(|e| crate::ServiceError::Storage(e.to_string()))?;

        let mut writer = BufWriter::new(file);
        let mut total_size: u64 = 0;

        // 流式写入
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| crate::ServiceError::Storage(e.to_string()))?;
            writer
                .write_all(&chunk)
                .await
                .map_err(|e| crate::ServiceError::Storage(e.to_string()))?;
            total_size += chunk.len() as u64;
        }

        // 确保所有数据都写入磁盘
        writer
            .flush()
            .await
            .map_err(|e| crate::ServiceError::Storage(e.to_string()))?;

        tracing::info!("File saved (stream): {} ({} bytes)", key, total_size);
        Ok(total_size)
    }

    async fn delete(&self, key: &str) -> std::result::Result<(), crate::ServiceError> {
        let path = self.full_path(key);

        if path.exists() {
            tokio::fs::remove_file(&path)
                .await
                .map_err(|e| crate::ServiceError::Storage(e.to_string()))?;
            tracing::info!("File deleted: {}", key);
        }

        Ok(())
    }

    async fn exists(&self, key: &str) -> std::result::Result<bool, crate::ServiceError> {
        let path = self.full_path(key);
        Ok(path.exists())
    }

    async fn get(&self, key: &str) -> std::result::Result<Vec<u8>, crate::ServiceError> {
        let path = self.full_path(key);
        if !path.exists() {
            return Err(crate::ServiceError::FileNotFound);
        }
        tokio::fs::read(&path)
            .await
            .map_err(|e| crate::ServiceError::Storage(e.to_string()))
    }

    async fn url(&self, key: &str) -> std::result::Result<String, crate::ServiceError> {
        // 生成可直接访问的 URL 路径
        // 文件存储在: root/{hash_prefix}/{key}
        // 访问路径: /api/files/{storage_tag}/{file_path}
        // 由于 key 的格式是 "{storage_tag}/{file_path}"，直接返回该路径
        Ok(format!("/api/files/{key}"))
    }

    async fn size(&self, key: &str) -> std::result::Result<u64, crate::ServiceError> {
        let path = self.full_path(key);
        let metadata = tokio::fs::metadata(&path)
            .await
            .map_err(|e| crate::ServiceError::Storage(e.to_string()))?;
        Ok(metadata.len())
    }
}
