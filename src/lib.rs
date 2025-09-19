use axum::{
    http::{Request, Response, StatusCode},
    body::Body,
};
use std::{
    task::{Context, Poll},
    sync::Arc,
};
use tokio::sync::RwLock;
use tower_service::Service;
use tower_layer::Layer;

/// A shared state for maintenance mode.
#[derive(Clone, Default)]
pub struct MaintenanceState {
    pub enabled: Arc<RwLock<bool>>,
}

impl MaintenanceState {
    /// Creates a new `MaintenanceState`.
    pub fn new() -> Self {
        Self { enabled: Arc::new(RwLock::new(false)), }
    }

    /// Enables maintenance mode.
    /// While maintenance mode is enabled, the server will automatically respond to requests with
    /// the provided `MaintenanceResponse`.
    pub async fn enable(&self) {
        *self.enabled.write().await = true;
    }

    /// Disables maintenance mode.
    /// Server requests will be handled normally again.
    pub async fn disable(&self) {
        *self.enabled.write().await = false;
    }

    /// Returns the current status of the `MaintenanceState`.
    pub async fn status(&self) -> bool {
        *self.enabled.read().await
    }
}

/// A trait to provide a customizable `MaintenanceResponse`.
pub trait MaintenanceResponse: Send + Sync + 'static {
    fn response(&self) -> Response<Body>;
}

impl<F> MaintenanceResponse for F
where
    F: Fn() -> Response<Body> + Send + Sync + 'static,
{
    fn response(&self) -> Response<Body> {
        (self)()
    }
}

/// A middleware layer for the maintenance mode.
#[derive(Clone)]
pub struct MaintenanceLayer {
    state: MaintenanceState,
    response: Arc<dyn MaintenanceResponse>,
}

impl MaintenanceLayer {
    /// Creates a new `MaintenanceLayer` with a default response.
    pub fn new(state: MaintenanceState) -> Self {
        let response = Arc::new(|| {
            Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .header("Content-Type", "text/html")
                .body(Body::from(
                    "<h1>Maintenance</h1><p>Site is under maintenance. Please try again later.</p>"
                ))
                .unwrap()
        });

        Self { state, response, }
    }

    /// Creates a new `MaintenanceLayer` with a provided response closure.
    pub fn with_response<R>(state: MaintenanceState, response: R) -> Self
    where
        R: MaintenanceResponse
    {
        Self {
            state,
            response: Arc::new(response),
        }
    }
}

impl<S> Layer<S> for MaintenanceLayer {
    type Service = MaintenanceMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MaintenanceMiddleware {
            inner,
            state: self.state.clone(),
            response: self.response.clone(),
        }
    }
}

/// Maintenance mode middleware service.
#[derive(Clone)]
pub struct MaintenanceMiddleware<S> {
    inner: S,
    state: MaintenanceState,
    response: Arc<dyn MaintenanceResponse>,
}

impl<S, ReqBody> Service<Request<ReqBody>> for MaintenanceMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<axum::BoxError>,
    ReqBody: Send + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = futures_util::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        let state = self.state.clone();
        let response = self.response.clone();

        Box::pin(async move {
            if state.status().await == true {
                Ok(response.response())
            } else {
                inner.call(req).await
            }
        })
    }
}