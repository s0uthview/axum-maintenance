use axum::{
    response::{Html, IntoResponse},
    http::StatusCode,
    routing::get,
    Router,
};
use axum_maintenance::{MaintenanceState, MaintenanceLayer};
use std::time::Duration;

fn maintenance_page() -> impl IntoResponse {
    let html = r#"
    <h1>Maintenance</h1>
    <p>This is a custom maintenance page.</p>
    <p>Please try again later.</p>
    "#;

    (StatusCode::SERVICE_UNAVAILABLE, Html(html))
}

#[tokio::main]
async fn main() {
    let state = MaintenanceState::new();

    // enable maintenance mode after 15 seconds
    let toggle = state.clone();

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(15)).await;
        toggle.enable().await;
    });

    // use our custom handler
    let layer = MaintenanceLayer::with_response(state, || {
        maintenance_page().into_response()
    });

    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .layer(layer);

    let addr = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(addr, app.into_make_service())
        .await
        .unwrap();
}