use axum::{Router, routing::get};
use axum_maintenance::{MaintenanceState, MaintenanceLayer};
use std::time::Duration;

#[tokio::main]
async fn main() {
    // create new maintenance state
    let state = MaintenanceState::new();

    // enable maintenance mode after 15 seconds
    let toggle = state.clone();

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(15)).await;
        toggle.enable().await;
    });

    // create maintenance layer
    let layer = MaintenanceLayer::new(state);

    // create app
    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .layer(layer);

    let addr = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    // serve!
    axum::serve(addr, app.into_make_service())
        .await
        .unwrap();
}