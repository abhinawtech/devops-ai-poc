# Multi-stage build for optimal image size
FROM rust:1.88 as builder

WORKDIR /app

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./

# Build deps with a dummy main to leverage cache
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy real sources
COPY src ./src

# Build the actual application (release)
RUN cargo build --release

# ---- Runtime image ----
FROM debian:bookworm-slim

# curl is used by HEALTHCHECK; certs for HTTPS
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

# Non-root user for security
RUN useradd --create-home --shell /bin/bash app
USER app
WORKDIR /home/app

# Copy binary from builder
# 👈 change ai-model-service if your binary name differs
COPY --from=builder /app/target/release/ai-model-service ./ai-model-service

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -fsS http://localhost:3000/health || exit 1

# 👈 change if you renamed the binary above
CMD ["./ai-model-service"]
