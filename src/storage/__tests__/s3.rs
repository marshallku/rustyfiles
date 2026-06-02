#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, http::StatusCode};
    use bytes::Bytes;
    use dotenv::dotenv;

    use crate::{
        env::app::{Env, StorageBackend},
        storage::{
            s3::{S3Config, S3Storage},
            Storage, StorageError,
        },
    };

    const PROBE_KEY: &str = "rustyfiles-e2e/roundtrip-probe.txt";
    const MISSING_KEY: &str = "rustyfiles-e2e/this-key-must-not-exist.txt";

    /// Builds the S3 config from the real environment, exercising the same
    /// wiring `AppState::from_env` uses. For Cloudflare R2 we derive the
    /// endpoint from `R2_ACCOUNT_ID` when `S3_ENDPOINT` is not set explicitly.
    fn load_config() -> S3Config {
        dotenv().ok();

        if std::env::var("S3_ENDPOINT")
            .ok()
            .filter(|v| !v.is_empty())
            .is_none()
        {
            if let Ok(account) = std::env::var("R2_ACCOUNT_ID") {
                if !account.is_empty() {
                    std::env::set_var(
                        "S3_ENDPOINT",
                        format!("https://{account}.r2.cloudflarestorage.com"),
                    );
                }
            }
        }
        std::env::set_var("STORAGE_BACKEND", "s3");

        let env = Env::new();
        assert_eq!(
            env.storage_backend,
            StorageBackend::S3,
            "STORAGE_BACKEND should resolve to S3"
        );
        let s3 = env.s3.expect("S3 env must be populated when backend is S3");

        S3Config {
            endpoint: s3.endpoint,
            region: s3.region,
            bucket: s3.bucket,
            access_key_id: s3.access_key_id,
            secret_access_key: s3.secret_access_key,
            force_path_style: s3.force_path_style,
        }
    }

    /// End-to-end check against a real S3/R2 bucket: put -> exists -> get ->
    /// serve -> missing-key handling. Ignored by default since it needs live
    /// credentials and network access. Run with:
    ///   cargo test --bins s3_roundtrip -- --ignored --nocapture
    #[tokio::test]
    #[ignore = "hits a real S3/R2 bucket; run with `cargo test -- --ignored`"]
    async fn s3_roundtrip() {
        let storage = S3Storage::new(load_config()).await;
        let payload = Bytes::from_static(b"rustyfiles s3 e2e roundtrip");

        // 1. put_bytes uploads the object.
        storage
            .put_bytes(PROBE_KEY, payload.clone())
            .await
            .expect("put_bytes should succeed");

        // 2. exists reports the freshly written object.
        assert!(
            storage.exists(PROBE_KEY).await.expect("exists should succeed"),
            "object should exist right after put_bytes"
        );

        // 3. get_bytes round-trips the exact payload.
        let fetched = storage
            .get_bytes(PROBE_KEY)
            .await
            .expect("get_bytes should succeed");
        assert_eq!(fetched, payload, "round-tripped bytes must match");

        // 4. serve returns 200 with the same body.
        let response = storage.serve(PROBE_KEY).await.expect("serve should succeed");
        assert_eq!(response.status(), StatusCode::OK);
        let served = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should collect");
        assert_eq!(served, payload, "served body must match the upload");

        // 5. missing keys are reported as absent / NotFound, not as errors.
        assert!(
            !storage
                .exists(MISSING_KEY)
                .await
                .expect("exists(missing) should succeed"),
            "a non-existent key must not report as existing"
        );
        match storage.get_bytes(MISSING_KEY).await {
            Err(StorageError::NotFound) => {}
            other => panic!("expected StorageError::NotFound for missing key, got {other:?}"),
        }
    }
}
