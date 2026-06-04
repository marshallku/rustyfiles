#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::response::Response;
    use bytes::Bytes;
    use reqwest::StatusCode;

    use crate::{
        env::{
            app::{ImageLimits, OriginMode},
            state::AppState,
        },
        services::{file::process_file_request, image::process_image_request},
        storage::{Storage, StorageError},
    };

    /// Storage that holds nothing, so every lookup is a miss. `put_bytes`
    /// panics: in bucket-origin mode a miss must short-circuit to 404 *before*
    /// any fetch-and-cache, so a write here means the short-circuit failed.
    struct EmptyStorage;

    #[async_trait]
    impl Storage for EmptyStorage {
        async fn exists(&self, _key: &str) -> Result<bool, StorageError> {
            Ok(false)
        }

        async fn get_bytes(&self, _key: &str) -> Result<Bytes, StorageError> {
            Err(StorageError::NotFound)
        }

        async fn put_bytes(&self, _key: &str, _bytes: Bytes) -> Result<(), StorageError> {
            panic!("put_bytes must not be called on a bucket-origin miss");
        }

        async fn serve(&self, _key: &str) -> Result<Response, StorageError> {
            Err(StorageError::NotFound)
        }
    }

    fn bucket_origin_state() -> AppState {
        AppState {
            host: "https://example.com/".to_string(),
            port: 0,
            address: "127.0.0.1".to_string(),
            allowed_hosts: Vec::new(),
            origin_mode: OriginMode::Bucket,
            storage: Arc::new(EmptyStorage),
            image_limits: ImageLimits::default(),
        }
    }

    #[tokio::test]
    async fn file_request_404s_on_bucket_miss_without_remote_fetch() {
        let state = bucket_origin_state();
        let result = process_file_request(&state, None, "/some/file.txt").await;
        assert_eq!(result.err(), Some(StatusCode::NOT_FOUND));
    }

    #[tokio::test]
    async fn image_request_404s_on_bucket_miss_without_remote_fetch() {
        let state = bucket_origin_state();
        let result = process_image_request(&state, None, "/some/image.png").await;
        assert_eq!(result.err(), Some(StatusCode::NOT_FOUND));
    }
}
