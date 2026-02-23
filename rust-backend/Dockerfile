# ProtonVPN Tailscale Exit Node - Production Dockerfile
# Multi-stage build for minimal image size

FROM rust:1.93.1-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

# Create app directory
WORKDIR /app

# Copy Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (cached layer)
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage - minimal alpine image
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    curl \
    wget \
    shadow \
    su-exec \
    && rm -rf /var/cache/apk/*

# Create app user
RUN adduser -D -s /bin/sh appuser

# Create necessary directories
RUN mkdir -p /app/config && chown appuser:appuser /app/config

# Copy binary from builder
COPY --from=builder /app/target/release/proton-vpn-api /app/proton-vpn-api

# Set ownership
RUN chown appuser:appuser /app/proton-vpn-api

# Copy entrypoint script that fixes docker socket permissions
COPY docker-entrypoint.sh /app/docker-entrypoint.sh
RUN chmod +x /app/docker-entrypoint.sh

# Expose API port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget -q --spider http://localhost:8080/health || exit 1

# Start as root, entrypoint drops to appuser after fixing permissions
ENTRYPOINT ["/app/docker-entrypoint.sh"]
