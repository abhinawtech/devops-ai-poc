#[cfg(test)]
mod integration_tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
    };
    use serde_json::{json, Value};
    use tower::util::ServiceExt;

    // Helper function to create the test app
    async fn create_test_app() -> Router {
        use ai_model_service::handlers::{health, predict};
        use ai_model_service::metrics::prometheus;
        use axum::routing::{get, post};

        // Initialize metrics (required for the app to work)
        prometheus::setup_metrics_recorder().expect("Failed to setup metrics");

        Router::new()
            .route("/health", get(health::health_check))
            .route("/predict", post(predict::predict))
            .route("/metrics", get(prometheus::metrics_handler))
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = create_test_app().await;

        let request = Request::builder()
            .method("GET")
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = std::str::from_utf8(&body).unwrap();
        
        let health_response: Value = serde_json::from_str(body_str).unwrap();
        
        assert_eq!(health_response["status"], "healthy");
        assert_eq!(health_response["service"], "ai-model-service");
        assert!(health_response["version"].is_string());
        assert!(health_response["timestamp"].is_string());
    }

    #[tokio::test]
    async fn test_predict_endpoint_valid_input() {
        let app = create_test_app().await;

        let request_body = json!({
            "features": [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
        });

        let request = Request::builder()
            .method("POST")
            .uri("/predict")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = std::str::from_utf8(&body).unwrap();
        
        let prediction_response: Value = serde_json::from_str(body_str).unwrap();
        
        assert!(prediction_response["prediction"].is_number());
        assert!(prediction_response["confidence"].is_number());
        assert_eq!(prediction_response["model_version"], "v1.0.0");

        let confidence = prediction_response["confidence"].as_f64().unwrap();
        assert!(confidence >= 0.85 && confidence <= 1.0);
    }

    #[tokio::test]
    async fn test_predict_endpoint_invalid_feature_count() {
        let app = create_test_app().await;

        let request_body = json!({
            "features": [1.0, 2.0, 3.0] // Only 3 features instead of 10
        });

        let request = Request::builder()
            .method("POST")
            .uri("/predict")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_predict_endpoint_invalid_json() {
        let app = create_test_app().await;

        let request = Request::builder()
            .method("POST")
            .uri("/predict")
            .header("content-type", "application/json")
            .body(Body::from("invalid json"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_predict_endpoint_missing_features_field() {
        let app = create_test_app().await;

        let request_body = json!({
            "invalid_field": [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
        });

        let request = Request::builder()
            .method("POST")
            .uri("/predict")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let app = create_test_app().await;

        let request = Request::builder()
            .method("GET")
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let content_type = response.headers().get("content-type").unwrap();
        assert!(content_type.to_str().unwrap().starts_with("text/plain"));

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = std::str::from_utf8(&body).unwrap();
        
        // Check that metrics response is not empty and contains essential metrics
        assert!(!body_str.is_empty());
        assert!(body_str.contains("service_uptime_seconds"));
    }

    #[tokio::test]
    async fn test_predict_endpoint_with_nan_values() {
        let app = create_test_app().await;

        let request_body = json!({
            "features": [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, null] // null will become NaN
        });

        let request = Request::builder()
            .method("POST")
            .uri("/predict")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should fail during deserialization or validation
        assert!(response.status().is_client_error());
    }

    #[tokio::test]
    async fn test_multiple_predict_requests() {
        let app = create_test_app().await;

        let request_body = json!({
            "features": [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
        });

        // Make multiple requests to test consistency
        for _ in 0..3 {
            let request = Request::builder()
                .method("POST")
                .uri("/predict")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);

            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let body_str = std::str::from_utf8(&body).unwrap();
            
            let prediction_response: Value = serde_json::from_str(body_str).unwrap();
            assert!(prediction_response["prediction"].is_number());
            assert!(prediction_response["confidence"].is_number());
            assert_eq!(prediction_response["model_version"], "v1.0.0");
        }
    }

    #[tokio::test]
    async fn test_nonexistent_endpoint() {
        let app = create_test_app().await;

        let request = Request::builder()
            .method("GET")
            .uri("/nonexistent")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}