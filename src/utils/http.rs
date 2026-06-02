use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};

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
