use async_trait::async_trait;

#[async_trait]
pub trait AsyncEnv: Send + Sync {
    async fn read_at(&self, path: &str, offset: u64, len: usize) -> std::io::Result<Vec<u8>>;

    async fn write_at(&self, path: &str, offset: u64, data: &[u8]) -> std::io::Result<()>;

    async fn create_file(&self, path: &str) -> std::io::Result<()>;

    async fn delete_file(&self, path: &str) -> std::io::Result<()>;
}
