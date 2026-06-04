use async_trait::async_trait;
use aws_config::{BehaviorVersion, Region};
use aws_credential_types::Credentials;
use aws_sdk_s3::{
    config::Builder as S3ConfigBuilder,
    error::SdkError,
    operation::{get_object::GetObjectError, head_object::HeadObjectError, put_object::PutObjectError},
    primitives::ByteStream,
    Client,
};
use axum::{
    body::Body,
    http::HeaderValue,
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use tokio_util::io::ReaderStream;

use crate::utils::http::get_cache_header;

use super::{Storage, StorageError};

const YEAR_TO_SECONDS: u32 = 31536000;
const CREDENTIALS_PROVIDER: &str = "rustyfiles-env";

pub struct S3Config {
    pub endpoint: Option<String>,
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub force_path_style: bool,
}

pub struct S3Storage {
    client: Client,
    bucket: String,
}

impl S3Storage {
    pub async fn new(config: S3Config) -> Self {
        let credentials = Credentials::new(
            config.access_key_id,
            config.secret_access_key,
            None,
            None,
            CREDENTIALS_PROVIDER,
        );

        let mut builder = S3ConfigBuilder::new()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(config.region))
            .credentials_provider(credentials)
            .force_path_style(config.force_path_style);

        if let Some(endpoint) = config.endpoint {
            builder = builder.endpoint_url(endpoint);
        }

        let client = Client::from_conf(builder.build());

        Self {
            client,
            bucket: config.bucket,
        }
    }
}

#[async_trait]
impl Storage for S3Storage {
    async fn exists(&self, key: &str) -> Result<bool, StorageError> {
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(normalize_key(key))
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(SdkError::ServiceError(err)) if matches!(err.err(), HeadObjectError::NotFound(_)) => {
                Ok(false)
            }
            Err(err) => Err(StorageError::Other(format!("{:?}", err))),
        }
    }

    async fn get_bytes(&self, key: &str) -> Result<Bytes, StorageError> {
        let output = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(normalize_key(key))
            .send()
            .await
            .map_err(map_get_err)?;

        let data = output
            .body
            .collect()
            .await
            .map_err(|err| StorageError::Other(err.to_string()))?;
        Ok(data.into_bytes())
    }

    async fn put_bytes(&self, key: &str, bytes: Bytes) -> Result<(), StorageError> {
        let mime_type = mime_guess::from_path(key)
            .first_or_octet_stream()
            .to_string();

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(normalize_key(key))
            .body(ByteStream::from(bytes))
            .content_type(mime_type)
            .send()
            .await
            .map_err(map_put_err)?;
        Ok(())
    }

    async fn serve(&self, key: &str) -> Result<Response, StorageError> {
        let output = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(normalize_key(key))
            .send()
            .await
            .map_err(map_get_err)?;

        let content_type = output.content_type().map(|s| s.to_string()).unwrap_or_else(|| {
            mime_guess::from_path(key)
                .first_or_octet_stream()
                .to_string()
        });

        let reader = output.body.into_async_read();
        let stream = ReaderStream::new(reader);
        let body = Body::from_stream(stream);

        let mut headers = get_cache_header(YEAR_TO_SECONDS);
        // `content_type` is upstream/S3-controlled, so it may not be a valid
        // header value — fall back instead of panicking.
        let content_type = HeaderValue::from_str(&content_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));
        headers.insert("Content-Type", content_type);

        Ok((headers, body).into_response())
    }
}

fn normalize_key(key: &str) -> String {
    key.trim_start_matches('/').to_string()
}

fn map_get_err<R>(err: SdkError<GetObjectError, R>) -> StorageError
where
    R: std::fmt::Debug,
{
    if let SdkError::ServiceError(ref service_err) = err {
        if matches!(service_err.err(), GetObjectError::NoSuchKey(_)) {
            return StorageError::NotFound;
        }
    }
    StorageError::Other(format!("{:?}", err))
}

fn map_put_err<R>(err: SdkError<PutObjectError, R>) -> StorageError
where
    R: std::fmt::Debug,
{
    StorageError::Other(format!("{:?}", err))
}
