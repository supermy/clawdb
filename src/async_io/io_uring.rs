use crate::async_io::env::AsyncEnv;
use async_trait::async_trait;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};

pub struct IoUringEnv;

impl IoUringEnv {
    pub fn new() -> Self {
        Self
    }
}

impl Default for IoUringEnv {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AsyncEnv for IoUringEnv {
    async fn read_at(&self, path: &str, offset: u64, len: usize) -> std::io::Result<Vec<u8>> {
        let path = path.to_string();
        tokio::task::spawn_blocking(move || {
            let mut file = OpenOptions::new().read(true).open(&path)?;
            file.seek(SeekFrom::Start(offset))?;
            let mut buffer = vec
![0u8; len];
            file.read_exact(&mut buffer)?;
            Ok(buffer)
        })
        .await
        .map_err(std::io::Error::other)?
    }

    async fn write_at(&self, path: &str, offset: u64, data: &[u8]) -> std::io::Result<()> {
        let data = data.to_vec();
        let path = path.to_string();

        tokio::task::spawn_blocking(move || {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)?;
            file.seek(SeekFrom::Start(offset))?;
            file.write_all(&data)?;
            Ok(())
        })
        .await
        .map_err(std::io::Error::other)?
    }

    async fn create_file(&self, path: &str) -> std::io::Result<()> {
        let path = path.to_string();
        tokio::task::spawn_blocking(move || {
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)?;
            Ok(())
        })
        .await
        .map_err(std::io::Error::other)?
    }

    async fn delete_file(&self, path: &str) -> std::io::Result<()> {
        let path = path.to_string();
        tokio::task::spawn_blocking(move || {
            std::fs::remove_file(&path)?;
            Ok(())
        })
        .await
        .map_err(std::io::Error::other)?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_io_uring_read_write() {
        let env = IoUringEnv::new();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        let data = b"Hello, io_uring!";
        env.write_at(path, 0, data).await.unwrap();

        let read_data = env.read_at(path, 0, data.len()).await.unwrap();
        assert_eq!(read_data, data);
    }
}
