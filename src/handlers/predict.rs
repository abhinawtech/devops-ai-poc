use axum::{extract::Json, http::StatusCode};
use crate::models::ml_model::{get_model, PredictionRequest, PredictionResponse};
use crate::metrics::prometheus::record_ml_prediction;

/// Prediction endpoint
/// 
/// Accepts a JSON payload with features and returns ML model prediction
pub async fn predict(
    Json(request): Json<PredictionRequest>,
) -> Result<Json<PredictionResponse>, StatusCode> {
    tracing::info!(
        feature_count = request.features.len(),xxff
        "Prediction request received"
    );

    // Get the global model instance dcxdcd
    let model = get_model();

    // Perform prediction
    match model.predict(&request.features) {
        Ok(prediction_response) => {
            // Record successful prediction metrics
            record_ml_prediction(&prediction_response.model_version, prediction_response.confidence, true);
            
            tracing::info!(
                prediction = %prediction_response.prediction,
                confidence = %prediction_response.confidence,
                "Prediction completed successfully"
            );
            Ok(Json(prediction_response))
        }
        Err(e) => {
            // Record failed prediction metrics
            record_ml_prediction("v1.0.0", 0.0, false);
            
            tracing::error!(
                error = %e,
                "Prediction failed"
            );
            // Return bad request for invalid input
            Err(StatusCode::BAD_REQUEST)
        }
    }
}