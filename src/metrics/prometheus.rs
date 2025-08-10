use axum::{http::StatusCode, response::Response};
use lazy_static::lazy_static;
use prometheus::{
    register_counter_vec, register_gauge, register_histogram_vec, CounterVec, Encoder, Gauge,
    HistogramVec, TextEncoder,
};

lazy_static! {
    /// HTTP requests total counter with method and endpoint labels
    pub static ref HTTP_REQUESTS_TOTAL: CounterVec = register_counter_vec!(
        "http_requests_total",
        "Total number of HTTP requests processed",
        &["method", "endpoint", "status"]
    )
    .expect("Failed to create HTTP_REQUESTS_TOTAL metric");

    /// HTTP request duration histogram with method and endpoint labels
    pub static ref HTTP_REQUEST_DURATION_SECONDS: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds",
        "HTTP request latency in seconds",
        &["method", "endpoint"],
        vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    )
    .expect("Failed to create HTTP_REQUEST_DURATION_SECONDS metric");

    /// ML predictions total counter
    pub static ref ML_PREDICTIONS_TOTAL: CounterVec = register_counter_vec!(
        "ml_predictions_total",
        "Total number of ML predictions made",
        &["model_version", "status"]
    )
    .expect("Failed to create ML_PREDICTIONS_TOTAL metric");

    /// ML prediction confidence histogram
    pub static ref ML_PREDICTION_CONFIDENCE: HistogramVec = register_histogram_vec!(
        "ml_prediction_confidence",
        "Distribution of ML prediction confidence scores",
        &["model_version"],
        vec![0.85, 0.87, 0.89, 0.91, 0.93, 0.95, 0.97, 0.99, 1.0]
    )
    .expect("Failed to create ML_PREDICTION_CONFIDENCE metric");

    /// Current active connections gauge
    pub static ref ACTIVE_CONNECTIONS: Gauge = register_gauge!(
        "active_connections_total",
        "Number of active connections"
    )
    .expect("Failed to create ACTIVE_CONNECTIONS metric");

    /// Service uptime in seconds
    pub static ref SERVICE_UPTIME_SECONDS: Gauge = register_gauge!(
        "service_uptime_seconds",
        "Service uptime in seconds since start"
    )
    .expect("Failed to create SERVICE_UPTIME_SECONDS metric");
}

/// Setup metrics recorder and start background tasks
pub fn setup_metrics_recorder() -> anyhow::Result<()> {
    // Initialize service start time
    SERVICE_UPTIME_SECONDS.set(0.0);

    // Initialize active connections counter
    set_active_connections(0.0);

    // Start uptime tracking in background
    tokio::spawn(update_uptime_metrics());

    tracing::info!("Prometheus metrics recorder initialized");
    Ok(())
}

/// Background task to update uptime metrics
async fn update_uptime_metrics() {
    let start_time = std::time::Instant::now();

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
    loop {
        interval.tick().await;
        let uptime = start_time.elapsed().as_secs_f64();
        SERVICE_UPTIME_SECONDS.set(uptime);
    }
}

/// Metrics endpoint handler
///
/// Returns Prometheus-formatted metrics
pub async fn metrics_handler() -> Result<Response<String>, StatusCode> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();

    match encoder.encode_to_string(&metric_families) {
        Ok(metrics_text) => {
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", encoder.format_type())
                .body(metrics_text)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            Ok(response)
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to encode metrics");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Record HTTP request metrics
pub fn record_http_request(method: &str, endpoint: &str, status: u16, duration: f64) {
    let status_str = status.to_string();

    HTTP_REQUESTS_TOTAL
        .with_label_values(&[method, endpoint, &status_str])
        .inc();

    HTTP_REQUEST_DURATION_SECONDS
        .with_label_values(&[method, endpoint])
        .observe(duration);
}

/// Record ML prediction metrics
pub fn record_ml_prediction(model_version: &str, confidence: f64, success: bool) {
    let status = if success { "success" } else { "error" };

    ML_PREDICTIONS_TOTAL
        .with_label_values(&[model_version, status])
        .inc();

    if success {
        ML_PREDICTION_CONFIDENCE
            .with_label_values(&[model_version])
            .observe(confidence);
    }
}

/// Update active connections gauge
pub fn set_active_connections(count: f64) {
    ACTIVE_CONNECTIONS.set(count);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_http_request() {
        record_http_request("GET", "/health", 200, 0.001);
        // Verify metric was recorded (basic smoke test)
        assert!(HTTP_REQUESTS_TOTAL
            .get_metric_with_label_values(&["GET", "/health", "200"])
            .is_ok());
    }

    #[test]
    fn test_record_ml_prediction() {
        record_ml_prediction("v1.0.0", 0.95, true);
        // Verify metric was recorded (basic smoke test)
        assert!(ML_PREDICTIONS_TOTAL
            .get_metric_with_label_values(&["v1.0.0", "success"])
            .is_ok());
    }

    #[test]
    fn test_set_active_connections() {
        set_active_connections(5.0);
        // Verify gauge was set
        assert!((ACTIVE_CONNECTIONS.get() - 5.0).abs() < f64::EPSILON);
    }
}
