#[cfg(test)]
mod unit_tests {
    use ai_model_service::handlers::health::HealthResponse;
    use ai_model_service::models::ml_model::{LinearRegressionModel, PredictionRequest};

    #[test]
    fn test_linear_regression_model_creation() {
        let model = LinearRegressionModel::new();
        // Model should be created successfully
        // Weight length is tested in the model's own tests
        assert!(std::mem::size_of_val(&model) > 0);
    }

    #[test]
    fn test_prediction_with_valid_input() {
        let model = LinearRegressionModel::new();
        let features = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

        let result = model.predict(&features);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.confidence >= 0.85 && response.confidence <= 1.0);
        assert_eq!(response.model_version, "v1.0.0");
        assert!(response.prediction.is_finite());
    }

    #[test]
    fn test_prediction_with_invalid_feature_count() {
        let model = LinearRegressionModel::new();
        let features = vec![1.0, 2.0, 3.0]; // Only 3 features instead of 10

        let result = model.predict(&features);
        assert!(result.is_err());
    }

    #[test]
    fn test_prediction_with_infinite_values() {
        let model = LinearRegressionModel::new();
        let mut features = vec![1.0; 10];
        features[0] = f64::INFINITY;

        let result = model.predict(&features);
        assert!(result.is_err());
    }

    #[test]
    fn test_prediction_with_nan_values() {
        let model = LinearRegressionModel::new();
        let mut features = vec![1.0; 10];
        features[5] = f64::NAN;

        let result = model.predict(&features);
        assert!(result.is_err());
    }

    #[test]
    fn test_prediction_request_deserialization() {
        let json = r#"{"features": [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]}"#;
        let request: Result<PredictionRequest, _> = serde_json::from_str(json);

        assert!(request.is_ok());
        let req = request.unwrap();
        assert_eq!(req.features.len(), 10);
    }

    #[test]
    fn test_confidence_score_range() {
        let model = LinearRegressionModel::new();
        let test_cases = vec![
            vec![0.0; 10],
            vec![1.0; 10],
            vec![-1.0; 10],
            vec![10.0; 10],
            vec![-10.0; 10],
        ];

        for features in test_cases {
            let result = model.predict(&features);
            assert!(result.is_ok());
            let response = result.unwrap();
            assert!(
                response.confidence >= 0.85 && response.confidence <= 1.0,
                "Confidence {} not in range [0.85, 1.0]",
                response.confidence
            );
        }
    }

    #[test]
    fn test_prediction_deterministic() {
        let model = LinearRegressionModel::new();
        let features = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

        let result1 = model.predict(&features).unwrap();
        let result2 = model.predict(&features).unwrap();

        assert_eq!(result1.prediction, result2.prediction);
        assert_eq!(result1.confidence, result2.confidence);
        assert_eq!(result1.model_version, result2.model_version);
    }

    #[test]
    fn test_model_version_constant() {
        let model = LinearRegressionModel::new();
        let features = vec![1.0; 10];
        let result = model.predict(&features).unwrap();

        assert_eq!(result.model_version, "v1.0.0");
    }

    #[tokio::test]
    async fn test_health_response_structure() {
        // Test that HealthResponse can be serialized
        let health = HealthResponse {
            status: "healthy".to_string(),
            service: "ai-model-service".to_string(),
            version: "0.1.0".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let json_result = serde_json::to_string(&health);
        assert!(json_result.is_ok());

        let json = json_result.unwrap();
        assert!(json.contains("healthy"));
        assert!(json.contains("ai-model-service"));
        assert!(json.contains("0.1.0"));
    }
}
