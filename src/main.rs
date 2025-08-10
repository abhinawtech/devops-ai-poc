use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod metrics;
mod models;

use handlers::{health, predict};
use metrics::prometheus::setup_metrics_recorder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ai_model_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Setup Prometheus metrics
    setup_metrics_recorder()?;

    // Build application router with all routes
    let app = Router::new()
        .route("/health", get(health::health_check))
        .route("/predict", post(predict::predict))
        .route("/metrics", get(metrics::prometheus::metrics_handler))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
        .layer(middleware::from_fn(metrics::middleware::metrics_middleware));

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("🚀 AI Model Service starting on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
