# axum-maintenance
`axum-maintenance` is a crate that provides "maintenance mode" functionality to an Axum application. When enabled, the server will automatically respond to requests with a pre-defined or custom page to inform the user that the server is undergoing maintenance.
## Installation
To install this crate, simply add:
```toml
axum-maintenance = { git = "https://github.com/s0uthview/axum-maintenance.git" }
```
to your `Cargo.toml` file. Once added, you should be good to go. This crate requires `axum`, `futures-util`, `tokio`, `tower-http`, and `tower-service`.

You can also clone this repository and run `cargo run --example [example_name]` to run one of the example programs.
## Examples
### Basic
```rust
use axum::{Router, routing::get};
use axum_maintenance::{MaintenanceState, MaintenanceLayer};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let state = MaintenanceState::new();

    // enable maintenance mode after 15 seconds
    let toggle = state.clone();

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(15)).await;
        toggle.enable().await;
    });

    let layer = MaintenanceLayer::new(state);

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
```
When a response is not provided, the default one will look like this:

![A web page stating "Site is under maintenance. Please try again later."](/assets/example1.png "Example Site 1")
### Use your own response
```rust
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
```
Our custom response page looks like:

![A web page stating "This is a custom maintenance page. Please try again later.](/assets/example2.png "Example Site 2")