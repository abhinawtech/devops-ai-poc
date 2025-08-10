# AI Model Service

A Rust web service for AI model inference with clean architecture, comprehensive testing, and Prometheus metrics.

## 🚀 Features

- **RESTful API** with Axum web framework
- **Mock ML Model** using linear regression with ndarray
- **Health Checks** with service metadata
- **Prometheus Metrics** for observability
- **Structured Logging** with tracing
- **Comprehensive Testing** (unit + integration)
- **Clean Architecture** with proper separation of concerns

## 📋 API Endpoints

### Health Check
```
GET /health
```
Returns service status, version, and timestamp.

**Response Example:**
```json
{
  "status": "healthy",
  "service": "ai-model-service", 
  "version": "0.1.0",
  "timestamp": "2025-01-15T10:30:00Z"
}
```

### Prediction
```
POST /predict
Content-Type: application/json

{
  "features": [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
}
```
Returns ML model prediction with confidence score.

**Response Example:**
```json
{
  "prediction": 15.7, 
  "confidence": 0.92, 
  "model_version": "v1.0.0"
}
```

### Metrics
```
GET /metrics
```
Returns Prometheus-formatted metrics for monitoring.

## 🔧 Setup and Running

### Prerequisites
- Rust 1.75+ (using edition 2021)
- Cargo

### Installation
```bash
git clone <repository>
cd ai-model-service
cargo build
```

### Running the Service
```bash
cargo run
```
The service will start on `http://0.0.0.0:3000`

### Running Tests
```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_tests

# Run with output
cargo test -- --nocapture
```

## 🏗️ Architecture

### Project Structure
```
src/
├── main.rs              # Application entry point and server setup
├── lib.rs               # Library exports for testing
├── handlers/            # HTTP request handlers
│   ├── mod.rs
│   ├── health.rs        # Health check endpoint
│   └── predict.rs       # ML prediction endpoint
├── models/              # ML model implementation
│   ├── mod.rs
│   └── ml_model.rs      # Linear regression model
└── metrics/             # Monitoring and metrics
    ├── mod.rs
    └── prometheus.rs    # Prometheus metrics setup
tests/
├── unit_tests.rs        # Unit tests
└── integration_tests.rs # Integration tests
```

### Key Components

**ML Model (`models/ml_model.rs`)**
- 10-dimensional feature vector input
- Linear regression with deterministic weights
- Input validation (NaN, infinite values)
- Confidence scoring (0.85-1.0 range)
- Thread-safe global instance

**Metrics (`metrics/prometheus.rs`)**
- HTTP request counters and latency histograms
- ML prediction metrics
- Service uptime tracking
- Custom business metrics

**Handlers (`handlers/`)**
- Health check with metadata
- ML prediction with error handling
- Structured logging for observability

## 📊 Monitoring

The service exposes the following Prometheus metrics:

- `http_requests_total` - Total HTTP requests by method, endpoint, status
- `http_request_duration_seconds` - Request latency histogram
- `ml_predictions_total` - ML prediction counts by model version and status
- `ml_prediction_confidence` - Distribution of prediction confidence scores
- `active_connections_total` - Current active connections
- `service_uptime_seconds` - Service uptime since start

## 🧪 Testing

The project includes comprehensive testing:

**Unit Tests** (`tests/unit_tests.rs`)
- Model creation and validation
- Prediction accuracy and determinism
- Input validation and error handling
- Confidence score ranges

**Integration Tests** (`tests/integration_tests.rs`) 
- End-to-end HTTP endpoint testing
- JSON serialization/deserialization
- Error scenarios and edge cases
- Metrics endpoint verification

### Test Coverage
- ✅ Model prediction logic
- ✅ HTTP endpoint functionality
- ✅ Error handling and validation
- ✅ Metrics collection
- ✅ Health checks
- ✅ JSON parsing and responses

## 🔍 Usage Examples

### Health Check
```bash
curl http://localhost:3000/health
```

### Make a Prediction
```bash
curl -X POST http://localhost:3000/predict \
  -H "Content-Type: application/json" \
  -d '{
    "features": [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
  }'
```

### View Metrics
```bash
curl http://localhost:3000/metrics
```

## 🛠️ Development

### Code Style
- Uses `#![forbid(unsafe_code)]` principles from Rust ecosystem
- Comprehensive error handling with `anyhow`
- Structured logging with contextual information
- Input validation and sanitization
- Efficient async patterns with Tokio

### Dependencies
- **axum** 0.8 - Modern web framework
- **tokio** 1.45 - Async runtime
- **prometheus** 0.13 - Metrics collection
- **ndarray** 0.16 - Numerical computing
- **serde** 1.0 - Serialization
- **tracing** - Structured logging
- **anyhow** - Error handling

## 🚢 Production Considerations

- Structured logging for observability
- Prometheus metrics for monitoring
- Input validation and error handling
- Thread-safe model instance
- CORS and tracing middleware
- Comprehensive test coverage

## 📄 License

This project demonstrates clean Rust architecture patterns for ML inference services.