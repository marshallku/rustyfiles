use axum::{
    body::Body,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use mime_guess;
use std::path::PathBuf;
use tokio_util::io::ReaderStream;

const YEAR_TO_SECONDS: u32 = 31536000;

pub fn get_cache_header(age: u32) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let cache_age = if age == 0 {
        "no-cache".to_string()
    } else {
        format!("public, max-age={}", age)
    };

    headers.insert("Cache-Control", cache_age.parse().unwrap());

    headers
}

pub fn response_error(status_code: StatusCode) -> Response {
    (status_code, get_cache_header(0)).into_response()
}

pub async fn response_file(file_path: &PathBuf) -> Response {
    let file = match tokio::fs::File::open(file_path).await {
        Ok(file) => file,
        Err(_) => {
            return response_error(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let mut headers = get_cache_header(YEAR_TO_SECONDS);

    let mime_type = mime_guess::from_path(file_path).first_or_octet_stream();
    headers.insert("Content-Type", mime_type.to_string().parse().unwrap());

    (headers, body).into_response()
}
