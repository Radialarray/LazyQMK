# LazyQMK Web Deployment Guide

This guide explains how to build and deploy the standalone `lazyqmk-web` binary with embedded frontend files.

## Overview

The `lazyqmk-web` binary can be compiled with the frontend assets embedded using `rust-embed`. This creates a single executable that serves both the REST API and the web UI, making deployment simple and portable.

## Building the Standalone Binary

### Prerequisites

- Rust 1.91.1 or newer
- Node.js 18+ and npm/pnpm (for building frontend)

### Step-by-Step Build Process

**1. Build the frontend**

```bash
cd web
npm install           # or: pnpm install
npm run build         # Creates web/build/ directory
```

This creates the production build in `web/build/` with minified HTML, CSS, and JavaScript.

**2. Build the Rust binary with embedded frontend**

```bash
cd ..
cargo build --release --features web --bin lazyqmk-web
```

The `rust-embed` crate will bundle all files from `web/build/` into the binary at compile time.

**3. Run the standalone binary**

```bash
./target/release/lazyqmk-web
```

The server will start on `http://localhost:3001` by default, serving both the API and frontend.

## How It Works

### Static File Embedding

- **Compile-time bundling**: The `#[derive(RustEmbed)]` macro on the `Assets` struct embeds all files from `web/build/` into the binary
- **MIME type detection**: The `mime_guess` crate automatically sets correct `Content-Type` headers based on file extensions
- **Memory efficiency**: Files are embedded as static data, no disk I/O at runtime

### Request Routing

The router handles requests in this order:

1. **API routes** (`/api/*`, `/health`): Handled by API route handlers
2. **Static files** (e.g., `/_app/immutable/entry/app.B4z6suyA.js`): Served from embedded assets
3. **SPA fallback**: Any other GET request serves `index.html` (enables client-side routing)

### SPA (Single Page Application) Support

The frontend uses SvelteKit for client-side routing. When a user navigates to `/layouts/my-layout` and refreshes, the server serves `index.html`, which then handles the routing client-side.

## Development vs. Production

### Development Mode

When developing the frontend with Vite:

```bash
cd web
npm run dev:web
```

- Backend runs on port 3001 (serves API only)
- Vite dev server runs on port 5173 (serves frontend with hot-reload)
- Vite proxies API requests to the backend
- Frontend files are **not** embedded in the binary

### Production Mode

When the binary is built with `--release`:

- Backend serves both API and frontend on a single port
- Frontend files are embedded in the binary
- No separate Vite server needed
- Suitable for deployment

## Configuration Options

```bash
# Custom port
./target/release/lazyqmk-web --port 8080

# Custom host (bind to all interfaces)
./target/release/lazyqmk-web --host 0.0.0.0

# Custom workspace directory
./target/release/lazyqmk-web --workspace ~/my-layouts

# Enable verbose logging
./target/release/lazyqmk-web --verbose
```

## Deployment Scenarios

### Local Machine

```bash
./target/release/lazyqmk-web
# Access at http://localhost:3001
```

### LAN Access

```bash
./target/release/lazyqmk-web --host 0.0.0.0
# Access from other devices at http://<your-ip>:3001
```

### Behind Reverse Proxy (e.g., nginx)

```bash
./target/release/lazyqmk-web --host 127.0.0.1 --port 3001
```

Example nginx config:

```nginx
server {
    listen 80;
    server_name lazyqmk.local;

    location / {
        proxy_pass http://127.0.0.1:3001;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Systemd Service (Linux)

Create `/etc/systemd/system/lazyqmk-web.service`:

```ini
[Unit]
Description=LazyQMK Web Server
After=network.target

[Service]
Type=simple
User=lazyqmk
WorkingDirectory=/home/lazyqmk
ExecStart=/usr/local/bin/lazyqmk-web --workspace /home/lazyqmk/layouts
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable lazyqmk-web
sudo systemctl start lazyqmk-web
```

## Troubleshooting

### Frontend not loading

**Problem**: Browser shows "Frontend not embedded - use Vite dev server"

**Solution**: Build the frontend before building the Rust binary:
```bash
cd web && npm run build && cd .. && cargo build --release --features web --bin lazyqmk-web
```

### Static files returning 404

**Problem**: Files like `/_app/immutable/entry/app.B4z6suyA.js` return 404

**Causes**:
- Frontend wasn't built before compiling the binary
- Frontend build is outdated (file hashes changed)

**Solution**: Rebuild both frontend and backend:
```bash
cd web && npm run build && cd .. && cargo clean && cargo build --release --features web --bin lazyqmk-web
```

### CORS errors in development

**Problem**: API requests fail with CORS errors

**Solution**: Use the development script which configures CORS properly:
```bash
cd web && npm run dev:web
```

## Binary Size

The standalone binary with embedded frontend is larger than the API-only binary:

- **API-only** (`--no-default-features --features web`): ~8-10 MB (stripped)
- **With embedded frontend**: ~10-12 MB (stripped)

The frontend assets (HTML, CSS, JS) add approximately 2-3 MB to the binary size.

## Security Considerations

- **CORS**: The default CORS policy allows all origins. For production, consider restricting to specific domains.
- **HTTPS**: Use a reverse proxy (nginx, Caddy) to add HTTPS termination
- **Firewall**: If binding to `0.0.0.0`, ensure firewall rules restrict access appropriately
- **Workspace path**: The `--workspace` flag determines where layout files are stored. Ensure proper file permissions.

## Future Improvements

Potential enhancements for the deployment story:

- [ ] Add `--no-embed` flag to disable static file serving (API-only mode)
- [ ] Support environment variables for configuration
- [ ] Add Docker image with multi-stage build
- [ ] Add health check endpoint for monitoring
- [ ] Add metrics/telemetry endpoint
- [ ] Support custom frontend builds (e.g., from different directories)

## Related Documentation

- [web/README.md](../web/README.md) - Frontend development guide
- [web/SETUP.md](../web/SETUP.md) - Detailed setup instructions
- [ARCHITECTURE.md](ARCHITECTURE.md) - Overall project architecture
