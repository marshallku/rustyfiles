use dotenv::dotenv;
use std::sync::Arc;

use crate::{
    constants::CDN_ROOT,
    storage::{local::LocalStorage, s3::S3Storage, SharedStorage},
};

use super::app::{Env, StorageBackend};

#[derive(Clone)]
pub struct AppState {
    pub host: String,
    pub port: u16,
    pub address: String,
    pub allowed_hosts: Vec<String>,
    pub storage: SharedStorage,
}

impl AppState {
    pub async fn from_env() -> Self {
        dotenv().ok();

        let env = Env::new();

        let storage: SharedStorage = match env.storage_backend {
            StorageBackend::Local => Arc::new(LocalStorage::new(CDN_ROOT)),
            StorageBackend::S3 => {
                let s3_env = env
                    .s3
                    .clone()
                    .expect("S3 env must be set when STORAGE_BACKEND=s3");
                let config = crate::storage::s3::S3Config {
                    endpoint: s3_env.endpoint,
                    region: s3_env.region,
                    bucket: s3_env.bucket,
                    access_key_id: s3_env.access_key_id,
                    secret_access_key: s3_env.secret_access_key,
                    force_path_style: s3_env.force_path_style,
                };
                Arc::new(S3Storage::new(config).await)
            }
        };

        Self {
            host: env.host.into_owned(),
            port: env.port,
            address: env.address.into_owned(),
            allowed_hosts: env.allowed_hosts,
            storage,
        }
    }

    pub fn is_host_allowed(&self, host: &str) -> bool {
        if self.allowed_hosts.is_empty() {
            true
        } else {
            self.allowed_hosts.iter().any(|allowed| allowed == host)
        }
    }
}
