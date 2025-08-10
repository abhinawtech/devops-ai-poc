use axum::{http::StatusCode, response::Json};
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub timestamp: DateTime<Utc>,
}

/// Health check endpoint
///
/// Returns service status, version, and current timestamp
pub async fn health_check() -> Result<Json<HealthResponse>, StatusCode> {
    let health_response = HealthResponse {
        status: "healthy".to_string(),
        service: "ai-model-service-production".to_string(), // Updated for production deployment test
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: Utc::now(),
    };

    tracing::info!("Health check requested");
    Ok(Json(health_response))
}
