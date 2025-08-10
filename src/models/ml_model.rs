use anyhow::{anyhow, Result};
use ndarray::Array1;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

const EXPECTED_FEATURES: usize = 10;
const MODEL_VERSION: &str = "v1.0.0";

#[derive(Debug, Serialize, Deserialize)]
pub struct PredictionRequest {
    pub features: Vec<f64>,
}

#[derive(Debug, Serialize)]
pub struct PredictionResponse {
    pub prediction: f64,
    pub confidence: f64,
    pub model_version: String,
}

/// Mock ML Model implementing simple linear regression
/// Uses randomly initialized weights for demonstration purposes
pub struct LinearRegressionModel {
    weights: Array1<f64>,
    bias: f64,
}

impl Default for LinearRegressionModel {
    fn default() -> Self {
        Self::new()
    }
}

impl LinearRegressionModel {
    /// Create a new linear regression model with random weights
    pub fn new() -> Self {
        // Initialize with deterministic "random" weights for consistent results
        let weights = Array1::from_vec(vec![
            0.15, -0.23, 0.87, -0.45, 0.67, 0.34, -0.12, 0.89, -0.56, 0.78,
        ]);
        let bias = 2.5;

        Self { weights, bias }
    }

    /// Validate input features
    fn validate_features(&self, features: &[f64]) -> Result<()> {
        if features.len() != EXPECTED_FEATURES {
            return Err(anyhow!(
                "Expected {} features, got {}",
                EXPECTED_FEATURES,
                features.len()
            ));
        }

        // Check for invalid values (NaN, infinite)
        for (i, &value) in features.iter().enumerate() {
            if !value.is_finite() {
                return Err(anyhow!("Invalid feature value at index {}: {}", i, value));
            }
        }

        Ok(())
    }

    /// Generate confidence score based on prediction magnitude
    fn calculate_confidence(&self, prediction: f64) -> f64 {
        // Simple confidence calculation: higher for predictions closer to bias
        let distance_from_bias = (prediction - self.bias).abs();
        let max_distance = 50.0; // Reasonable max distance for normalization
        let normalized_distance = (distance_from_bias / max_distance).min(1.0);

        // Confidence between 0.85 and 1.0
        0.85 + (1.0 - normalized_distance) * 0.15
    }

    /// Perform prediction using linear regression
    pub fn predict(&self, features: &[f64]) -> Result<PredictionResponse> {
        // Validate input
        self.validate_features(features)?;

        // Convert to ndarray for efficient computation
        let feature_array = Array1::from_vec(features.to_vec());

        // Linear regression: prediction = weights * features + bias
        let prediction = self.weights.dot(&feature_array) + self.bias;

        // Calculate confidence score
        let confidence = self.calculate_confidence(prediction);

        tracing::debug!(
            prediction = %prediction,
            confidence = %confidence,
            "Model prediction completed"
        );

        Ok(PredictionResponse {
            prediction,
            confidence,
            model_version: MODEL_VERSION.to_string(),
        })
    }
}

/// Global model instance using OnceLock for thread-safe lazy initialization
static MODEL_INSTANCE: OnceLock<LinearRegressionModel> = OnceLock::new();

/// Get the global model instance
pub fn get_model() -> &'static LinearRegressionModel {
    MODEL_INSTANCE.get_or_init(LinearRegressionModel::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_creation() {
        let model = LinearRegressionModel::new();
        assert_eq!(model.weights.len(), EXPECTED_FEATURES);
        assert_eq!(model.bias, 2.5);
    }

    #[test]
    fn test_valid_prediction() {
        let model = LinearRegressionModel::new();
        let features = vec![1.0; EXPECTED_FEATURES];
        let result = model.predict(&features);

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.confidence >= 0.85 && response.confidence <= 1.0);
        assert_eq!(response.model_version, MODEL_VERSION);
    }

    #[test]
    fn test_invalid_feature_count() {
        let model = LinearRegressionModel::new();
        let features = vec![1.0; 5]; // Wrong number of features
        let result = model.predict(&features);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_feature_values() {
        let model = LinearRegressionModel::new();
        let mut features = vec![1.0; EXPECTED_FEATURES];
        features[0] = f64::NAN;
        let result = model.predict(&features);

        assert!(result.is_err());
    }

    #[test]
    fn test_global_model_instance() {
        let model1 = get_model();
        let model2 = get_model();

        // Should be the same instance
        assert_eq!(std::ptr::addr_of!(*model1), std::ptr::addr_of!(*model2));
    }
}
