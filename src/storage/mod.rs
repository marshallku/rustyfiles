mod __tests__;
pub mod local;
pub mod s3;

use async_trait::async_trait;
use axum::response::Response;
use bytes::Bytes;
use std::sync::Arc;

#[derive(Debug)]
pub enum StorageError {
    NotFound,
    Other(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::NotFound => write!(f, "not found"),
            StorageError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for StorageError {}

#[async_trait]
pub trait Storage: Send + Sync + 'static {
    async fn exists(&self, key: &str) -> Result<bool, StorageError>;

    async fn get_bytes(&self, key: &str) -> Result<Bytes, StorageError>;

    async fn put_bytes(&self, key: &str, bytes: Bytes) -> Result<(), StorageError>;

    async fn serve(&self, key: &str) -> Result<Response, StorageError>;
}

pub type SharedStorage = Arc<dyn Storage>;
