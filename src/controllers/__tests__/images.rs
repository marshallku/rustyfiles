#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    use crate::{
        constants::CDN_ROOT, controllers::app::app, env::state::AppState,
        utils::url::get_host_from_url,
    };

    const URI: &str = "/images";

    #[tokio::test]
    async fn test_response_file() {
        let app = app();
        let state = AppState::from_env();
        let file_path = "/images/hpp/ic_wahlberg_product_core_48.png8.png";
        let response = app
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri(format!("{}{}", URI, file_path))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let local_file_path = PathBuf::from(format!(
            "{}{}/{}{}",
            CDN_ROOT,
            URI,
            get_host_from_url(&state.host),
            file_path
        ));

        assert_eq!(response.status(), StatusCode::OK);
        assert!(local_file_path.exists());
    }

    #[tokio::test]
    async fn test_response_error() {
        let app = app();
        let state = AppState::from_env();
        let file_path = "/images/you-must-not-exist.png";
        let response = app
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri(format!("{}{}", URI, file_path))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let local_file_path = PathBuf::from(format!(
            "{}{}/{}{}",
            CDN_ROOT,
            URI,
            get_host_from_url(&state.host),
            file_path
        ));

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert!(!local_file_path.exists());
    }

    #[tokio::test]
    async fn test_response_webp_file() {
        let app = app();
        let state = AppState::from_env();
        let file_path = "/images/hpp/ic_wahlberg_product_core_48.png8.png.webp";
        let response = app
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri(format!("{}{}", URI, file_path))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let local_file_path = PathBuf::from(format!(
            "{}{}/{}{}",
            CDN_ROOT,
            URI,
            get_host_from_url(&state.host),
            file_path
        ));

        assert_eq!(response.status(), StatusCode::OK);
        assert!(local_file_path.exists());
    }
}
