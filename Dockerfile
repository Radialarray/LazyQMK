# LazyQMK Backend Dockerfile
# Multi-stage build for minimal production image

# =============================================================================
# Stage 1: Build the Rust backend
# =============================================================================
FROM rust:latest AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy only dependency files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY rust-toolchain.toml ./

# Create dummy src files to build dependencies
RUN mkdir -p src/bin && \
    echo 'fn main() {}' > src/main.rs && \
    echo 'pub fn lib() {}' > src/lib.rs

# Build dependencies only (this layer is cached if dependencies don't change)
RUN cargo build --release --features web --bin lazyqmk 2>/dev/null || true

# Remove dummy files and compiled binary to force rebuild
RUN rm -rf src target/release/lazyqmk target/release/deps/lazyqmk-*

# Copy actual source code
COPY src ./src

# Build the actual binary
RUN cargo build --release --features web --bin lazyqmk

# =============================================================================
# Stage 2: Runtime image
# =============================================================================
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN useradd -r -s /bin/false lazyqmk

# Copy binary from builder
COPY --from=builder /app/target/release/lazyqmk /usr/local/bin/lazyqmk

# Create directories for volume mounts
RUN mkdir -p /app/workspace /app/qmk_firmware && \
    chown -R lazyqmk:lazyqmk /app

# Copy keycode database and other data files needed at runtime
COPY --from=builder /app/src/keycode_db /app/src/keycode_db
COPY --from=builder /app/src/data /app/src/data

USER lazyqmk

# Expose the API port
EXPOSE 3001

# Environment variables for configuration
ENV LAZYQMK_HOST=0.0.0.0
ENV LAZYQMK_PORT=3001

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3001/health || exit 1

# Default command
ENTRYPOINT ["lazyqmk"]
CMD ["--web", "--host", "0.0.0.0", "--port", "3001", "--workspace", "/app/workspace"]
