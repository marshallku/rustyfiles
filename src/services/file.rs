use axum::response::Response;
use reqwest::StatusCode;
use tracing::error;

use crate::{
    env::state::AppState,
    utils::{fetch::fetch_remote, url::get_host_from_url},
};

pub async fn process_file_request(
    state: &AppState,
    host: Option<String>,
    path: &str,
) -> Result<Response, StatusCode> {
    let target_host = host.unwrap_or(state.host.clone());
    let key = format!(
        "files/{}/{}",
        get_host_from_url(&target_host),
        path.trim_start_matches('/')
    );

    match state.storage.exists(&key).await {
        Ok(true) => {
            return state
                .storage
                .serve(&key)
                .await
                .map_err(|err| map_storage_err(err, &key));
        }
        Ok(false) => {}
        Err(err) => return Err(map_storage_err(err, &key)),
    }

    let bytes = match fetch_remote(&target_host, path).await {
        Ok(bytes) => bytes,
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };

    if let Err(err) = state.storage.put_bytes(&key, bytes).await {
        error!("Failed to persist {}: {}", key, err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    state
        .storage
        .serve(&key)
        .await
        .map_err(|err| map_storage_err(err, &key))
}

fn map_storage_err(err: crate::storage::StorageError, key: &str) -> StatusCode {
    match err {
        crate::storage::StorageError::NotFound => StatusCode::NOT_FOUND,
        crate::storage::StorageError::Other(msg) => {
            error!("Storage error on {}: {}", key, msg);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
