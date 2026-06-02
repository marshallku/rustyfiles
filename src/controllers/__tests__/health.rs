#[cfg(test)]
mod tests {

    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    use crate::{controllers::health, env::state::AppState};

    #[tokio::test]
    async fn should_be_healthy() {
        let app: Router<AppState> = Router::new().route("/", get(health::get));
        let state = AppState::from_env().await;
        let response = app
            .with_state(state)
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
