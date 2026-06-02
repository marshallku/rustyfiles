use async_trait::async_trait;
use axum::{
    body::Body,
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use std::path::PathBuf;
use tokio::{fs, io::AsyncReadExt};
use tokio_util::io::ReaderStream;

use crate::utils::http::get_cache_header;

use super::{Storage, StorageError};

const YEAR_TO_SECONDS: u32 = 31536000;

pub struct LocalStorage {
    root: PathBuf,
}

impl LocalStorage {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn resolve(&self, key: &str) -> PathBuf {
        self.root.join(key.trim_start_matches('/'))
    }
}

#[async_trait]
impl Storage for LocalStorage {
    async fn exists(&self, key: &str) -> Result<bool, StorageError> {
        fs::try_exists(self.resolve(key)).await.map_err(to_other)
    }

    async fn get_bytes(&self, key: &str) -> Result<Bytes, StorageError> {
        let path = self.resolve(key);
        let mut file = fs::File::open(&path).await.map_err(map_io)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await.map_err(to_other)?;
        Ok(Bytes::from(buf))
    }

    async fn put_bytes(&self, key: &str, bytes: Bytes) -> Result<(), StorageError> {
        let path = self.resolve(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(to_other)?;
        }
        fs::write(&path, bytes).await.map_err(to_other)?;
        Ok(())
    }

    async fn serve(&self, key: &str) -> Result<Response, StorageError> {
        let path = self.resolve(key);
        let file = fs::File::open(&path).await.map_err(map_io)?;
        let stream = ReaderStream::new(file);
        let body = Body::from_stream(stream);

        let mut headers = get_cache_header(YEAR_TO_SECONDS);
        let mime_type = mime_guess::from_path(&path).first_or_octet_stream();
        headers.insert("Content-Type", mime_type.to_string().parse().unwrap());

        Ok((headers, body).into_response())
    }
}

fn map_io(err: std::io::Error) -> StorageError {
    if err.kind() == std::io::ErrorKind::NotFound {
        StorageError::NotFound
    } else {
        StorageError::Other(err.to_string())
    }
}

fn to_other<E: std::fmt::Display>(err: E) -> StorageError {
    StorageError::Other(err.to_string())
}
