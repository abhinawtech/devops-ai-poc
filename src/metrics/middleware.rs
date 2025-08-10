use crate::metrics::prometheus::record_http_request;
use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

/// Middleware to record HTTP request metrics
pub async fn metrics_middleware(request: Request, next: Next) -> Response {
    let start_time = Instant::now();
    let method = request.method().to_string();
    let uri = request.uri().path().to_string();

    // Process the request
    let response = next.run(request).await;

    // Calculate duration and record metrics
    let duration = start_time.elapsed().as_secs_f64();
    let status = response.status().as_u16();

    record_http_request(&method, &uri, status, duration);

    response
}
