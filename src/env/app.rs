use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq)]
pub enum StorageBackend {
    Local,
    S3,
}

#[derive(Clone, Debug)]
pub struct S3Env {
    pub endpoint: Option<String>,
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub force_path_style: bool,
}

#[derive(Clone, Debug)]
pub struct Env {
    pub address: Cow<'static, str>,
    pub port: u16,
    pub host: Cow<'static, str>,
    pub allowed_hosts: Vec<String>,
    pub storage_backend: StorageBackend,
    pub s3: Option<S3Env>,
}

impl Env {
    pub fn new() -> Self {
        let address = match std::env::var("BIND_ADDRESS") {
            Ok(address) => Cow::Owned(address),
            Err(_) => Cow::Owned("127.0.0.1".to_string()),
        };
        let port = match std::env::var("PORT") {
            Ok(port) => port.parse().unwrap_or(41890),
            Err(_) => 41890,
        };
        let host = match std::env::var("HOST") {
            Ok(host) => Cow::Owned(host),
            Err(_) => Cow::Owned("http://localhost/".to_string()),
        };
        let allowed_hosts = match std::env::var("ALLOWED_HOSTS") {
            Ok(hosts) if !hosts.is_empty() => hosts
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            _ => Vec::new(),
        };

        let storage_backend = match std::env::var("STORAGE_BACKEND")
            .ok()
            .map(|v| v.trim().to_lowercase())
            .filter(|v| !v.is_empty())
            .as_deref()
        {
            None | Some("local") => StorageBackend::Local,
            Some("s3") | Some("r2") => StorageBackend::S3,
            Some(other) => panic!(
                "Invalid STORAGE_BACKEND value {:?}. Expected one of: local, s3, r2",
                other
            ),
        };

        let s3 = if storage_backend == StorageBackend::S3 {
            Some(S3Env {
                endpoint: std::env::var("S3_ENDPOINT").ok().filter(|s| !s.is_empty()),
                region: std::env::var("S3_REGION").unwrap_or_else(|_| "auto".to_string()),
                bucket: std::env::var("S3_BUCKET")
                    .expect("S3_BUCKET must be set when STORAGE_BACKEND=s3"),
                access_key_id: std::env::var("S3_ACCESS_KEY_ID")
                    .expect("S3_ACCESS_KEY_ID must be set when STORAGE_BACKEND=s3"),
                secret_access_key: std::env::var("S3_SECRET_ACCESS_KEY")
                    .expect("S3_SECRET_ACCESS_KEY must be set when STORAGE_BACKEND=s3"),
                force_path_style: std::env::var("S3_FORCE_PATH_STYLE")
                    .map(|v| v.to_lowercase() != "false")
                    .unwrap_or(true),
            })
        } else {
            None
        };

        Self {
            address,
            port,
            host,
            allowed_hosts,
            storage_backend,
            s3,
        }
    }
}
