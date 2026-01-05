# LazyQMK Web Editor Setup

This document covers setup for the LazyQMK web editor, including:
- Local development (recommended)
- Production usage
- Docker deployment (optional)
- Tauri desktop application (optional)

## Quick Start

### Recommended: Local Development

**Single command** (starts both backend and frontend):
```bash
cd web
pnpm install              # or npm install
pnpm dev:web              # or npm run dev:web
```

This starts:
- Rust backend on http://localhost:3001
- Vite dev server on http://localhost:5173 (with hot-reload)

Visit http://localhost:5173

### Alternative: Production Mode

For out-of-the-box usage without dev tools:
```bash
lazyqmk --web
```
Then open http://localhost:3001 in your browser.

**Custom configuration:**
```bash
# Custom workspace directory
lazyqmk --web --workspace ~/my-layouts

# Custom port
lazyqmk --web --port 8080

# Bind to all interfaces
lazyqmk --web --host 0.0.0.0
```

### Optional: Docker Deployment

Docker is **not required** for development. Use Docker only for containerized production deployments:

```bash
# Production build
docker compose up

# Development with hot reloading
docker compose --profile dev up
```

See Docker Deployment section below for details.

### Optional: Desktop App (Tauri)

```bash
cd web
npm install
npm run tauri:dev
```

---

## Local Development

### Prerequisites

- **Rust**: 1.91.1 or later (for backend)
- **Node.js**: 20.x or later (for frontend)
- **pnpm** or npm: 10.x or later

### Single-Command Development (Recommended)

```bash
cd web
pnpm install
pnpm dev:web
```

This automatically starts:
1. Rust backend (port 3001) - API server
2. Vite dev server (port 5173) - Frontend with hot-reload

The frontend proxies API calls to the backend automatically.

### Manual Two-Terminal Setup

If you prefer running processes separately:

```bash
# Terminal 1: Backend
cd .. && cargo run --features web --bin lazyqmk-web

# Terminal 2: Frontend
cd web && pnpm dev
```

### Backend Options

```bash
cargo run --features web --bin lazyqmk-web -- \
  --port 3001 \
  --host 127.0.0.1 \
  --workspace ~/my-layouts \
  --verbose
```

**Options:**
- `--port`: Port to listen on (default: 3001)
- `--host`: Host to bind to (default: 127.0.0.1)
- `--workspace`: Directory for layout files (default: platform-specific, see below)
- `--verbose`: Enable debug logging

### Default Workspace Directory

The backend stores layout files in a workspace directory:

- **Linux**: `~/.config/LazyQMK/layouts/`
- **macOS**: `~/Library/Application Support/LazyQMK/layouts/`
- **Windows**: `%APPDATA%\LazyQMK\layouts\`

This directory is created automatically on first run. Override with `--workspace` flag.

### Changing the Backend URL

The frontend proxies to `http://localhost:3001` by default (configured in `vite.config.ts`).

To change:
```typescript
// vite.config.ts
server: {
  proxy: {
    '/api': {
      target: 'http://localhost:3001',  // Change port here
      changeOrigin: true
    }
  }
}
```

---

## Docker Deployment (Optional)

**Note:** Docker is **optional** and only needed for containerized deployments. The recommended development workflow uses native tools (see Local Development above).

### Directory Structure

```
LazyQMK/
├── Dockerfile           # Backend production image
├── Dockerfile.dev       # Backend development image
├── docker-compose.yml   # Orchestration
└── web/
    └── Dockerfile       # Frontend production image
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `LAZYQMK_WORKSPACE` | `./examples` | Path to layout files directory |
| `QMK_FIRMWARE_PATH` | `./qmk_firmware` | Path to QMK firmware (for keyboard geometry) |
| `LAZYQMK_CONFIG_DIR` | Named volume | Directory for app configuration |

### Production Deployment

```bash
# Build and start all services
docker compose up --build

# Run in background
docker compose up -d

# View logs
docker compose logs -f

# Stop services
docker compose down
```

**Access:**
- Frontend: http://localhost:5173
- Backend API: http://localhost:3001

### Development with Docker

Use the `dev` profile for hot reloading:

```bash
# Start development containers
docker compose --profile dev up

# Only backend (for frontend on host)
docker compose --profile dev up backend-dev
```

### Volume Mounts

The docker-compose.yml mounts several directories:

1. **Workspace** (`/app/workspace`): Your layout markdown files
   ```bash
   export LAZYQMK_WORKSPACE=/path/to/my/layouts
   docker compose up
   ```

2. **QMK Firmware** (`/app/qmk_firmware`): Required for keyboard geometry
   ```bash
   export QMK_FIRMWARE_PATH=/path/to/qmk_firmware
   docker compose up
   ```

3. **Config** (`/home/lazyqmk/.config/LazyQMK`): Persistent configuration

### Building Individual Images

```bash
# Backend only
docker build -t lazyqmk-backend .

# Frontend only
docker build -t lazyqmk-frontend -f web/Dockerfile .
```

### Custom docker-compose.override.yml

For local customization, create `docker-compose.override.yml`:

```yaml
services:
  backend:
    volumes:
      - /my/custom/layouts:/app/workspace:rw
      - /my/qmk_firmware:/app/qmk_firmware:ro
    environment:
      - RUST_LOG=debug
```

---

## Tauri Desktop Application

The Tauri app wraps the web frontend in a native window and can spawn/manage the backend process.

### Prerequisites

- **Rust**: 1.70 or later
- **Node.js**: 20.x or later
- **Platform-specific dependencies**:
  - **macOS**: Xcode Command Line Tools
  - **Linux**: `libwebkit2gtk-4.1-dev`, `libssl-dev`, `libayatana-appindicator3-dev`
  - **Windows**: Visual Studio Build Tools, WebView2

### Development

```bash
cd web
npm install

# Start in development mode
npm run tauri:dev
```

This will:
1. Start the Vite dev server
2. Build and run the Tauri app
3. Open devtools automatically

### Building for Production

```bash
# Build the backend first
cargo build --release --features web --bin lazyqmk-web

# Build the desktop app
cd web
npm run tauri:build
```

Output locations:
- **macOS**: `web/src-tauri/target/release/bundle/dmg/`
- **Linux**: `web/src-tauri/target/release/bundle/deb/` and `appimage/`
- **Windows**: `web/src-tauri/target/release/bundle/msi/`

### Backend Connection

The Tauri app can connect to the backend in two ways:

1. **Spawn Backend**: The app spawns `lazyqmk-web` as a child process
   - Uses Tauri commands: `start_backend`, `stop_backend`
   - Backend is bundled with the app or found in PATH

2. **External Backend**: Connect to an existing backend
   - Set `PUBLIC_API_URL` environment variable
   - Useful when running backend via Docker

### Frontend Tauri Integration

Use the Tauri API in Svelte components:

```typescript
import { invoke } from '@tauri-apps/api/core';

// Start the backend
const result = await invoke('start_backend', { 
  workspacePath: '/path/to/layouts' 
});

// Get backend URL
const url = await invoke('get_backend_url');

// Check if running
const isRunning = await invoke('is_backend_running');

// Stop backend
await invoke('stop_backend');
```

### Icons

Place app icons in `web/src-tauri/icons/`:
- `32x32.png`
- `128x128.png`
- `128x128@2x.png`
- `icon.icns` (macOS)
- `icon.ico` (Windows)

Generate icons from a source image:
```bash
npm run tauri icon /path/to/icon.png
```

---

## Testing

### Unit Tests

```bash
cd web
npm run test              # Run once
npm run test:watch        # Watch mode
npm run test:ui           # Open UI
```

### E2E Tests

```bash
npm run test:e2e          # Run tests
npm run test:e2e:ui       # Open UI
```

E2E tests mock the API, so they work without a running backend.

### Backend Tests

```bash
cargo test --features web
```

---

## Project Structure

```
web/
├── src/
│   ├── lib/
│   │   ├── api/              # API client
│   │   ├── components/       # UI components
│   │   └── utils/            # Utilities
│   ├── routes/               # SvelteKit routes
│   ├── app.html              # HTML template
│   └── app.css               # Global styles
├── src-tauri/                # Tauri desktop app
│   ├── src/
│   │   ├── main.rs           # Entry point
│   │   ├── lib.rs            # Tauri setup
│   │   └── backend.rs        # Backend spawning
│   ├── tauri.conf.json       # Tauri config
│   └── Cargo.toml            # Rust dependencies
├── e2e/                      # E2E tests
├── Dockerfile                # Frontend Docker image
├── package.json              # Node dependencies
├── svelte.config.js          # SvelteKit config
├── vite.config.ts            # Vite config
└── tailwind.config.js        # Tailwind config
```

---

## Troubleshooting

### Docker: "Cannot connect to backend"

1. Check backend is running: `docker compose ps`
2. Check backend logs: `docker compose logs backend`
3. Verify health check: `curl http://localhost:3001/health`

### Docker: Volume permissions

If you see permission errors:

```bash
# Fix ownership (Linux)
sudo chown -R 1000:1000 /path/to/workspace

# Or run as root (not recommended for production)
docker compose up --user root
```

### Tauri: "Cannot find lazyqmk-web binary"

Build the backend first:
```bash
cargo build --release --features web --bin lazyqmk-web
```

Or install it:
```bash
cargo install --path . --features web --bin lazyqmk-web
```

### Tauri: WebView not found (Linux)

Install WebKitGTK:
```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-dev

# Fedora
sudo dnf install webkit2gtk4.1-devel
```

### Frontend: "Cannot find module 'vite'"

```bash
cd web
rm -rf node_modules package-lock.json
npm install
```

---

## API Reference

All endpoints are documented in [`ARCHITECTURE.md`](../docs/ARCHITECTURE.md).

Quick reference:
- `GET /health` - Health check
- `GET /api/layouts` - List layouts
- `GET /api/layouts/{filename}` - Get layout
- `PUT /api/layouts/{filename}` - Save layout
- `GET /api/keycodes` - Search keycodes
- `GET /api/keycodes/categories` - List categories
- `GET /api/config` - Get config
- `PUT /api/config` - Update config
- `GET /api/keyboards/{keyboard}/geometry/{layout}` - Get geometry
