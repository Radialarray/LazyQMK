# LazyQMK Backend Dockerfile
# Multi-stage build for minimal production image

# =============================================================================
# Stage 1: Build the Rust backend
# =============================================================================
# Use Rust 1.91.1+ as required by AGENTS.md
# Note: Using 1.92 specifically as it's the stable version that builds successfully
# in both local and Docker environments. See AGENTS.md and docs/DOCKER_BUILD.md.
FROM rust:1.92-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    gnupg \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js 20.x using official setup script with verification
# This is the recommended method from NodeSource for Debian/Ubuntu
RUN curl -fsSL https://deb.nodesource.com/setup_20.x -o /tmp/nodesource_setup.sh \
    && bash /tmp/nodesource_setup.sh \
    && apt-get install -y nodejs \
    && rm /tmp/nodesource_setup.sh \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency files and source code
COPY Cargo.toml Cargo.lock ./
COPY rust-toolchain.toml ./
COPY build.rs ./
COPY src ./src

# Copy web frontend for embedding
COPY web ./web

# Build the binary (build.rs will automatically build the web frontend)
RUN cargo build --release --features web --bin lazyqmk

# =============================================================================
# Stage 2: Runtime image
# =============================================================================
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies including curl for health checks and gosu for privilege dropping
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    gosu \
    && rm -rf /var/lib/apt/lists/*

# Install QMK build toolchain: Python, ARM/AVR compilers, and build tools
# These are required for `qmk compile` to work inside the container
# Note: Flashing tools (avrdude, dfu-util, etc.) are not included because
# LazyQMK only compiles firmware — it does not flash to keyboards.
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 \
    python3-pip \
    build-essential \
    git \
    unzip \
    wget \
    zip \
    diffutils \
    binutils-arm-none-eabi \
    gcc-arm-none-eabi \
    libnewlib-arm-none-eabi \
    binutils-avr \
    gcc-avr \
    avr-libc \
    && rm -rf /var/lib/apt/lists/*

# Install QMK CLI tool and verify installation
# Note: --break-system-packages is required on Debian bookworm (PEP 668)
RUN pip3 install --break-system-packages qmk && \
    qmk --version

# Create non-root user with home directory so config volume mounts work correctly
RUN useradd -r -m -d /home/lazyqmk -s /bin/false lazyqmk

# Copy binary from builder
COPY --from=builder /app/target/release/lazyqmk /usr/local/bin/lazyqmk

# Copy entrypoint script
COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

# Create directories for volume mounts and pre-create config dir with correct
# ownership — Docker will preserve this ownership when mounting named volumes
RUN mkdir -p /app/workspace /app/qmk_firmware \
        /home/lazyqmk/.config/LazyQMK \
        /home/lazyqmk/.config/LazyQMK/templates && \
    chown -R lazyqmk:lazyqmk /app /home/lazyqmk

# Copy keycode database and other data files needed at runtime
COPY --from=builder /app/src/keycode_db /app/src/keycode_db
COPY --from=builder /app/src/data /app/src/data

# NOTE: No USER directive - container starts as root, entrypoint drops to lazyqmk via gosu

# Expose the API port
EXPOSE 3001

# Environment variables for configuration
ENV LAZYQMK_HOST=0.0.0.0
ENV LAZYQMK_PORT=3001

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3001/health || exit 1

# Entrypoint handles permission fixing and privilege dropping
ENTRYPOINT ["docker-entrypoint.sh"]
CMD ["web", "--host", "0.0.0.0", "--port", "3001", "--workspace", "/app/workspace"]
