#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    use crate::{constants::CDN_ROOT, controllers::app::app, env::state::AppState};

    const URI: &str = "/files";

    #[tokio::test]
    async fn test_response_file() {
        let app = app();
        let state = AppState::from_env();
        let file_path = "/images/hpp/ic_wahlberg_product_core_48.png8.png";
        let response = app
            .with_state(state)
            .oneshot(
                Request::builder()
                    .uri(format!("{}{}", URI, file_path))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let local_file_path = PathBuf::from(format!("{}{}{}", CDN_ROOT, URI, file_path));

        assert_eq!(response.status(), StatusCode::OK);
        assert!(local_file_path.exists());
    }

    #[tokio::test]
    async fn test_response_file_with_host() {
        let app = app();
        let state = AppState::from_env();
        let host = "www.google.com";
        let full_host = format!("https://{}", host);
        let file_path: &'static str = "/images/hpp/ic_wahlberg_product_core_48.png8.png";
        let expected_file_path = format!("{}{}/{}{}", CDN_ROOT, URI, host, file_path);

        let response = app
            .with_state(state)
            .oneshot(
                Request::builder()
                    .uri(format!("{}/{}{}", URI, full_host, file_path))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let local_file_path = PathBuf::from(expected_file_path);

        assert_eq!(response.status(), StatusCode::OK);
        assert!(local_file_path.exists());
    }

    #[tokio::test]
    async fn test_response_error() {
        let app = app();
        let state = AppState::from_env();
        let file_path = "/images/you-must-not-exist.png";
        let response = app
            .with_state(state)
            .oneshot(
                Request::builder()
                    .uri(format!("{}{}", URI, file_path))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let local_file_path = PathBuf::from(format!("{}{}{}", CDN_ROOT, URI, file_path));

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert!(!local_file_path.exists());
    }
}
