use axum::response::Response;
use reqwest::StatusCode;
use std::path::PathBuf;
use tracing::error;

use crate::{
    constants::CDN_ROOT,
    env::state::AppState,
    utils::{fetch::fetch_and_cache, http::response_file, url::get_host_from_url},
};

pub async fn process_file_request(
    state: &AppState,
    host: Option<String>,
    path: &str,
) -> Result<Response, StatusCode> {
    let target_host = host.unwrap_or(state.host.clone());
    let file_path = PathBuf::from(format!(
        "{}/files/{}/{}",
        CDN_ROOT,
        get_host_from_url(&target_host),
        path
    ));

    if file_path.exists() {
        error!("File exists but respond with Rust: {:?}", file_path);
        return Ok(response_file(&file_path).await);
    }

    if fetch_and_cache(target_host, &file_path, path)
        .await
        .is_err()
    {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(response_file(&file_path).await)
}
