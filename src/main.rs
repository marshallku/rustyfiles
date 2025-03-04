mod constants;
mod controllers;
mod env;
mod services;
mod utils;

use controllers::app::app;
use env::state::AppState;
use tokio::net::TcpListener;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{info, Level};
use utils::log::trace_layer_on_request;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let state = AppState::from_env();
    let addr = format!("{}:{}", state.address, state.port);
    let app = app()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
                .on_request(trace_layer_on_request),
        )
        .with_state(state);

    let listener = TcpListener::bind(addr.as_str()).await.unwrap();

    info!("Server running at http://{}", addr);

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap()
}
