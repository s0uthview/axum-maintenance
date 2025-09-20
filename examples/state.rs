use axum::{
    routing::{get, post},
    response::{Html, IntoResponse},
    http::StatusCode,
    extract::State,
    Router,
};
use axum_maintenance::{MaintenanceState, MaintenanceLayer};

// a simple route to toggle maintenance mode
async fn toggle(State(state): State<MaintenanceState>) -> impl IntoResponse {
    if state.status().await == false {
        state.enable().await;
    } else {
        state.disable().await;
    }

    (StatusCode::OK, "toggled")
}

// maintenance page
fn maintenance_page() -> impl IntoResponse {
    let html = r#"
    <h1>Maintenance</h1>
    <p>This is a custom maintenance page.</p>
    <p>Please try again later.</p>
    "#;

    (StatusCode::SERVICE_UNAVAILABLE, Html(html))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = MaintenanceState::new();
    let layer = MaintenanceLayer::with_response(state.clone(), || {
        maintenance_page().into_response()
    });

    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" })
        .layer(layer))
        .route("/toggle", post(toggle))
        .with_state(state);

    let addr = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    axum::serve(addr, app.into_make_service()).await?;

    Ok(())
}