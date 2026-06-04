use axum::{
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};

pub fn get_cache_header(age: u32) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let cache_age = if age == 0 {
        "no-cache".to_string()
    } else {
        format!("public, max-age={}", age)
    };

    let value = HeaderValue::from_str(&cache_age)
        .unwrap_or_else(|_| HeaderValue::from_static("no-cache"));
    headers.insert("Cache-Control", value);

    headers
}

pub fn response_error(status_code: StatusCode) -> Response {
    (status_code, get_cache_header(0)).into_response()
}
