use thiserror::Error;

/// 存储错误类型
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    Sled(#[from] sled::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Key not found")]
    NotFound,

    #[error("Invalid data format")]
    InvalidData,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// 事务错误
#[derive(Debug, Error)]
pub enum TransactionError {
    #[error("Transaction failed: {0}")]
    Abort(String),
}
