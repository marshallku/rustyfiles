#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use aws_config::{BehaviorVersion, Region};
    use aws_credential_types::Credentials;
    use aws_sdk_s3::{config::Builder as S3ConfigBuilder, Client};
    use axum::{body::to_bytes, http::StatusCode};
    use bytes::Bytes;
    use dotenv::dotenv;
    use image::{DynamicImage, ImageFormat, RgbaImage};

    use crate::{
        env::{
            app::{Env, OriginMode, StorageBackend},
            state::AppState,
        },
        services::image::process_image_request,
        storage::{
            s3::{S3Config, S3Storage},
            Storage, StorageError,
        },
        utils::img::encode_image,
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

    const ORIGIN_ORIGINAL_KEY: &str = "images/rustyfiles-e2e/pipeline-probe.png";
    const ORIGIN_DERIVATIVE_KEY: &str = "images/rustyfiles-e2e/pipeline-probe.w300.png";

    /// A raw S3 client mirroring `S3Storage`'s config, used only for test
    /// setup/teardown (the `Storage` trait has no delete).
    fn raw_client(config: &S3Config) -> Client {
        let credentials = Credentials::new(
            config.access_key_id.clone(),
            config.secret_access_key.clone(),
            None,
            None,
            "rustyfiles-e2e-test",
        );
        let mut builder = S3ConfigBuilder::new()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(config.region.clone()))
            .credentials_provider(credentials)
            .force_path_style(config.force_path_style);
        if let Some(endpoint) = &config.endpoint {
            builder = builder.endpoint_url(endpoint.clone());
        }
        Client::from_conf(builder.build())
    }

    /// Full bucket-origin happy path against a real bucket: seed an original in
    /// the bucket, request a resized variant, and confirm the service reads the
    /// original from the bucket, generates the derivative, persists it back, and
    /// serves the resized image — with no upstream HTTP fetch.
    #[tokio::test]
    #[ignore = "hits a real S3/R2 bucket; run with `cargo test -- --ignored`"]
    async fn s3_bucket_origin_image_pipeline() {
        let config = load_config();
        let bucket = config.bucket.clone();
        let client = raw_client(&config);
        let storage = S3Storage::new(load_config()).await;

        // Clean slate so the request must read the original and generate the
        // derivative (a leftover derivative would short-circuit the read path).
        for key in [ORIGIN_ORIGINAL_KEY, ORIGIN_DERIVATIVE_KEY] {
            let _ = client.delete_object().bucket(&bucket).key(key).send().await;
        }

        // Seed a 600x400 original directly in the bucket (out-of-band upload).
        let original = DynamicImage::ImageRgba8(RgbaImage::new(600, 400));
        let original_png = encode_image(&original, ImageFormat::Png).expect("encode seed png");
        storage
            .put_bytes(ORIGIN_ORIGINAL_KEY, Bytes::from(original_png))
            .await
            .expect("seeding the original should succeed");

        let state = AppState {
            host: "https://e2e.invalid/".to_string(),
            port: 0,
            address: "127.0.0.1".to_string(),
            allowed_hosts: Vec::new(),
            origin_mode: OriginMode::Bucket,
            storage: Arc::new(S3Storage::new(load_config()).await),
        };

        // Request the resized variant; the derivative does not exist yet.
        let response =
            process_image_request(&state, None, "/rustyfiles-e2e/pipeline-probe.w300.png")
                .await
                .expect("image request should succeed in bucket-origin mode");
        assert_eq!(response.status(), StatusCode::OK);

        let served = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should collect");
        let decoded = image::load_from_memory(&served).expect("served body should be an image");
        assert_eq!(decoded.width(), 300, "served image should be resized to width 300");

        // The generated derivative must now be persisted in the bucket.
        assert!(
            storage
                .exists(ORIGIN_DERIVATIVE_KEY)
                .await
                .expect("exists(derivative) should succeed"),
            "the derivative should be written back to the bucket"
        );

        for key in [ORIGIN_ORIGINAL_KEY, ORIGIN_DERIVATIVE_KEY] {
            let _ = client.delete_object().bucket(&bucket).key(key).send().await;
        }
    }
}
