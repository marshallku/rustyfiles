use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    env::state::AppState,
    services::file::process_file_request,
    utils::{http::response_error, path::parse_path},
};

pub async fn get(State(state): State<AppState>, Path(path): Path<String>) -> impl IntoResponse {
    let (host, path) = parse_path(&path);

    if let Some(ref host_str) = host {
        if !state.is_host_allowed(host_str) {
            return response_error(StatusCode::FORBIDDEN);
        }
    }

    match process_file_request(&state, host, &path).await {
        Ok(response) => response,
        Err(status) => response_error(status),
    }
}
