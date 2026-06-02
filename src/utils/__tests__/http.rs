#[cfg(test)]
mod tests {
    use reqwest::StatusCode;

    use crate::utils::http::{get_cache_header, response_error};

    #[test]
    fn test_get_cache_header() {
        let headers = get_cache_header(100);
        let cache_control = headers.get("Cache-Control").unwrap().to_str().unwrap();

        assert_eq!(cache_control, "public, max-age=100");
    }

    #[test]
    fn test_get_cache_header_with_falsy_age() {
        let headers = get_cache_header(0);
        let cache_control = headers.get("Cache-Control").unwrap().to_str().unwrap();

        assert_eq!(cache_control, "no-cache");
    }

    #[test]
    fn test_response_error() {
        let response = response_error(StatusCode::NOT_FOUND);
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_response_error_with_cache() {
        let response = response_error(StatusCode::NOT_FOUND);
        let headers = response.headers();
        let cache_control = headers.get("Cache-Control").unwrap().to_str().unwrap();

        assert_eq!(cache_control, "no-cache");
    }
}
