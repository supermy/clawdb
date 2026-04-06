use crate::storage::StorageError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClawError {
    #[error("Vector dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("Vector not found: {0}")]
    VectorNotFound(u64),

    #[error("Index not built")]
    IndexNotBuilt,

    #[error("Invalid vector data: {0}")]
    InvalidVectorData(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("RocksDB error: {0}")]
    RocksDb(#[from] rocksdb::Error),

    #[error("Data loader error: {0}")]
    LoaderError(String),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
}

pub type Result<T> = std::result::Result<T, ClawError>;
