# Docker Build Configuration

This document explains the Rust version pinning strategy and Docker build configuration for LazyQMK.

## Rust Version Requirements

### Current Configuration

| Component | Rust Version | Rationale |
|-----------|-------------|-----------|
| **AGENTS.md (Minimum)** | 1.91.1+ | CI compatibility, clippy consistency |
| **Dockerfile (Production)** | 1.92 | Stable build in Docker environments |
| **Dockerfile.dev (Development)** | 1.92 | Consistency with production |
| **rust-toolchain.toml** | stable | Local development flexibility |
| **CI/CD (GitHub Actions)** | stable (latest) | Always test against newest stable |

### Why Pin to 1.92 in Docker?

**Problem**: Rust stable moves forward frequently, and newer versions can introduce:
- Stricter type inference requirements
- New clippy lints that fail CI
- Breaking changes in unstable features
- Platform-specific compilation differences

**Solution**: Pin Docker images to Rust 1.92 because:
1. **Verified compatibility**: Builds successfully on all platforms (Linux x86_64, ARM64, macOS, Windows)
2. **Meets minimum requirement**: 1.92 >= 1.91.1 (AGENTS.md requirement)
3. **Stable baseline**: Provides reproducible builds across environments
4. **Tested in production**: Current release (v0.16.0) built and verified with 1.92

**CI uses latest stable** to catch future compatibility issues early, but Docker provides a stable deployment baseline.

### Version Update Strategy

When updating the pinned Rust version:

1. **Test locally first**:
   ```bash
   # Update rust-toolchain.toml temporarily
   rustup override set 1.93.0
   cargo clean
   cargo test --all-features
   cargo clippy --all-features -- -D warnings
   ```

2. **Test Docker build**:
   ```bash
   # Update Dockerfile and Dockerfile.dev
   docker build -f Dockerfile -t lazyqmk-backend:test .
   docker build -f Dockerfile.dev -t lazyqmk-backend-dev:test .
   ```

3. **Test on all platforms** (via CI or manually):
   - Linux x86_64
   - Linux ARM64
   - macOS ARM64 (Apple Silicon)
   - Windows x86_64

4. **Update documentation**:
   - This file (DOCKER_BUILD.md)
   - AGENTS.md if changing minimum version
   - Release notes if user-facing impact

5. **Document rationale** if staying on older version (e.g., "1.93+ breaks feature X")

## Node.js Installation

### Current Approach

The production Dockerfile installs Node.js 20.x using the official NodeSource setup script:

```dockerfile
RUN curl -fsSL https://deb.nodesource.com/setup_20.x -o /tmp/nodesource_setup.sh \
    && bash /tmp/nodesource_setup.sh \
    && apt-get install -y nodejs \
    && rm /tmp/nodesource_setup.sh
```

### Why Not Multi-Stage Node Build?

**Option A (Current)**: Download and execute setup script
- **Pros**: Official NodeSource method, reliable, well-maintained
- **Cons**: Downloads external script (security consideration)
- **Mitigation**: Script is downloaded over HTTPS, saved to file for inspection

**Option B**: Multi-stage build with node:20 base image
- **Pros**: No external scripts, uses official Node.js Docker image
- **Cons**: Requires copying Node binary and dependencies, more complex, larger intermediate image
- **Trade-off**: Complexity vs. marginal security improvement

**Decision**: Stay with Option A because:
1. Official recommended method from NodeSource
2. Widely used in production (industry standard)
3. HTTPS download with `-fsSL` flags for security
4. Script saved to file before execution (auditable)
5. Simpler Dockerfile, easier to maintain

### Node.js Security Best Practices

If security concerns arise, consider these alternatives:

1. **Pin script checksum**:
   ```dockerfile
   RUN curl -fsSL https://deb.nodesource.com/setup_20.x -o /tmp/nodesource_setup.sh \
       && echo "EXPECTED_SHA256 /tmp/nodesource_setup.sh" | sha256sum -c - \
       && bash /tmp/nodesource_setup.sh
   ```

2. **Use multi-stage with official Node image**:
   ```dockerfile
   FROM node:20-bookworm-slim AS node-builder
   FROM rust:1.92-bookworm AS builder
   COPY --from=node-builder /usr/local/bin/node /usr/local/bin/
   COPY --from=node-builder /usr/local/lib/node_modules /usr/local/lib/node_modules
   ```

3. **Install from Debian repos** (older version, not recommended):
   ```dockerfile
   RUN apt-get install -y nodejs npm
   # Note: Debian stable has Node.js 18, not 20
   ```

**Current assessment**: The NodeSource setup script method is acceptable for LazyQMK's security posture.

## Build Optimization

### Dependency Caching Strategy

The Dockerfile uses a standard dependency pre-build pattern:

```dockerfile
# Copy dependency manifests
COPY Cargo.toml Cargo.lock ./
COPY rust-toolchain.toml ./
COPY build.rs ./

# Build dependencies (cached layer)
RUN cargo build --release --features web --bin lazyqmk

# Copy source code
COPY src ./src
COPY web ./web

# Build final binary (fast rebuild on source changes)
RUN cargo build --release --features web --bin lazyqmk
```

### Why Not Use Dummy Build?

Previous versions used a "dummy build" pattern:
1. Create src/main.rs with `fn main() {}`
2. Build to cache dependencies
3. Remove dummy artifacts
4. Copy real source and rebuild

**Problem**: Fragile, easy to break, caused build failures in testing.

**Current approach**: Simpler, more reliable, slightly slower on first build but cached effectively.

### Alternative: cargo-chef

For faster CI builds, consider [cargo-chef](https://github.com/LukeMathWalker/cargo-chef):

```dockerfile
FROM rust:1.92-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --features web --recipe-path recipe.json
COPY . .
RUN cargo build --release --features web --bin lazyqmk
```

**Trade-off**: More complex Dockerfile, requires cargo-chef maintenance, marginal speed improvement for typical development workflow.

**Decision**: Not implemented yet, can be added if CI build times become problematic.

## Multi-Platform Builds

### Current Support

LazyQMK Docker images support:
- linux/amd64 (x86_64)
- linux/arm64 (ARM64, including Apple Silicon via emulation)

### Building for Multiple Platforms

```bash
# Build for current platform
docker build -t lazyqmk-backend:latest .

# Build for multiple platforms (requires buildx)
docker buildx build --platform linux/amd64,linux/arm64 -t lazyqmk-backend:latest .

# Build and push to registry
docker buildx build --platform linux/amd64,linux/arm64 \
  -t ghcr.io/radialarray/lazyqmk:latest \
  --push .
```

### Platform-Specific Notes

- **ARM64 on Apple Silicon**: Works via Docker Desktop's Rosetta emulation
- **Cross-compilation**: Not currently used (native builds on each platform)
- **QMK firmware submodule**: Platform-independent (metadata only, no compilation in Docker)

## Testing Docker Builds

### Pre-Commit Testing

Before committing Dockerfile changes:

```bash
# 1. Build production image
docker build -f Dockerfile -t lazyqmk-backend:test .

# 2. Build development image
docker build -f Dockerfile.dev -t lazyqmk-backend-dev:test .

# 3. Test production image runs
docker run --rm lazyqmk-backend:test lazyqmk --version

# 4. Test with Docker Compose
docker compose build
docker compose up -d
docker compose exec backend lazyqmk --version
docker compose down
```

### CI Testing

GitHub Actions automatically tests Docker builds on tag push (see `.github/workflows/release.yml`).

Manual CI testing:

```bash
# Trigger manual workflow
gh workflow run release.yml --ref main
```

### Integration Testing

See [tmp/docker-test-checklist.md](../tmp/docker-test-checklist.md) for full manual test suite.

## Troubleshooting

### Build fails with "type annotations needed"

**Problem**: Rust compilation error E0282 in Docker but works locally.

**Solution**:
1. Verify Rust version matches between Dockerfile and local (`rustc --version`)
2. Clean build: `docker build --no-cache -f Dockerfile .`
3. Check Cargo.lock is committed and up-to-date
4. Try building locally with same Rust version: `rustup override set 1.92.0`

### Node.js version mismatch

**Problem**: Web frontend build fails due to Node.js version differences.

**Solution**:
1. Update setup script version in Dockerfile: `setup_20.x` → `setup_22.x`
2. Update web/Dockerfile to match: `FROM node:20-slim` → `FROM node:22-slim`
3. Test locally: `cd web && nvm use 22 && npm test`

### Docker build is slow

**Problem**: Every build takes 10+ minutes.

**Solutions**:
1. Use BuildKit (enabled by default in modern Docker): `export DOCKER_BUILDKIT=1`
2. Ensure Cargo.lock is committed (enables better layer caching)
3. Don't modify Cargo.toml unless necessary (breaks dependency cache)
4. Use docker-compose build with cache: `docker compose build --parallel`

### Platform-specific build failures

**Problem**: Builds on x86_64 but fails on ARM64.

**Solutions**:
1. Check for platform-specific dependencies in Cargo.toml
2. Test on target platform: `docker buildx build --platform linux/arm64 .`
3. Use multi-platform base images: `rust:1.92-bookworm` (not `rust:1.92-bullseye`)

## Related Documentation

- [AGENTS.md](../AGENTS.md) - Development guidelines and Rust version policy
- [DOCKER_QMK_SETUP.md](DOCKER_QMK_SETUP.md) - QMK firmware integration with Docker
- [docker-compose.yml](../docker-compose.yml) - Full stack configuration
- [.github/workflows/ci.yml](../.github/workflows/ci.yml) - CI pipeline configuration

## References

- [Rust Docker Official Images](https://hub.docker.com/_/rust)
- [NodeSource Node.js Binary Distributions](https://github.com/nodesource/distributions)
- [Docker Multi-Platform Builds](https://docs.docker.com/build/building/multi-platform/)
- [cargo-chef](https://github.com/LukeMathWalker/cargo-chef) - Dependency caching tool
