# LazyQMK Workspace Configuration

This document explains how the LazyQMK web backend handles workspace directories for storing layout files.

## Quick Start

**Default usage (recommended):**
```bash
lazyqmk --web
```
Uses platform-specific default workspace (see below).

**Custom workspace:**
```bash
lazyqmk --web --workspace ~/my-layouts
```

**Development:**
```bash
cd web && pnpm dev:web
```
Backend automatically uses default workspace.

## Default Workspace Behavior

When starting the web backend without the `--workspace` flag, it uses a platform-specific default directory:

- **Linux**: `~/.config/LazyQMK/layouts/`
- **macOS**: `~/Library/Application Support/LazyQMK/layouts/`
- **Windows**: `%APPDATA%\LazyQMK\layouts\`

This directory is **created automatically** on first run if it doesn't exist.

## Starting the Backend

### Quick Start

**Out-of-the-box (production):**
```bash
lazyqmk --web
```
Opens web UI at http://localhost:3001

**Development (with hot-reload):**
```bash
cd web && pnpm dev:web
```
Opens dev UI at http://localhost:5173

### Command-Line Options

```bash
# Custom workspace
lazyqmk --web --workspace ~/my-layouts

# Custom port and host
lazyqmk --web --port 8080 --host 0.0.0.0

# Combine options
lazyqmk --web --port 8080 --workspace ~/my-layouts
```

### What Happens

The backend will:
1. Determine the workspace directory (default or from `--workspace`)
2. Create the directory if it doesn't exist
3. Serve the API at the specified host and port
4. Return the workspace path via `GET /api/config`

### Legacy: Using lazyqmk-web Binary

The `lazyqmk-web` binary is the dedicated backend server. You can also start it directly:

```bash
# Same as: lazyqmk --web
cargo run --features web --bin lazyqmk-web

# With options
cargo run --features web --bin lazyqmk-web -- --workspace ~/my-layouts --port 3001
```

**Note:** The recommended way is `lazyqmk --web`, which provides the same functionality.

## Frontend Integration

The frontend Settings page displays the current workspace root by fetching `GET /api/config`.

### How It Works

- **Backend running**: Shows the actual workspace path (e.g., `/Users/user/Library/Application Support/LazyQMK/layouts/`)
- **Backend not running**: Shows error with instructions

### Starting the UI

**Production:**
```bash
lazyqmk --web
```
Visit http://localhost:3001

**Development:**
```bash
cd web && pnpm dev:web
```
Visit http://localhost:5173

## API Endpoint

### GET /api/config

Returns current configuration including workspace root:

```json
{
  "qmk_firmware_path": "/path/to/qmk_firmware",
  "output_dir": "/Users/user/Library/Application Support/LazyQMK/builds",
  "workspace_root": "/Users/user/Library/Application Support/LazyQMK/layouts"
}
```

**Note**: `workspace_root` is always present in the response (never null or empty).

## Testing

Tests verify the default workspace behavior:

```rust
#[tokio::test]
async fn test_default_workspace_created_and_returned() {
    // Verifies workspace_root is:
    // - Always present in ConfigResponse
    // - Never null or empty
    // - Points to an existing directory
}
```

Run with:

```bash
cargo test --features web test_default_workspace_created_and_returned
```

## Troubleshooting

### Issue: Frontend shows workspace error

**Symptom:** Cannot see workspace directory in Settings page

**Solution:** Start the backend:
```bash
lazyqmk --web
```

### Issue: Want to use a different workspace

**Solution:** Specify custom workspace:
```bash
lazyqmk --web --workspace ~/my-custom-layouts
```

### Issue: Workspace directory doesn't exist

**Solution:** The backend creates it automatically. If permissions prevent creation, check directory permissions or use a different path.

## Implementation Details

### Backend (Rust)

- **Binary**: `src/bin/lazyqmk-web.rs`
- **Default function**: `get_default_layouts_dir()` - Gets platform-specific path and creates directory
- **Config module**: `src/config.rs` - `Config::config_dir()` returns platform-specific base config directory
- **API handler**: `src/web/mod.rs` - `get_config()` returns `ConfigResponse` with workspace root

### Frontend (SvelteKit)

- **Settings page**: `web/src/routes/settings/+page.svelte`
- **API client**: `web/src/lib/api/client.ts`
- **Type definitions**: `web/src/lib/api/types.ts`

### Platform-Specific Directories

The backend uses the `dirs` crate to determine platform-specific directories:

```rust
// Linux: ~/.config/LazyQMK/
// macOS: ~/Library/Application Support/LazyQMK/
// Windows: %APPDATA%\LazyQMK\
let config_dir = dirs::config_dir()?.join("LazyQMK");

// Add layouts subdirectory
let workspace = config_dir.join("layouts");
```

## Backwards Compatibility

The implementation is fully backwards compatible:

- Existing users with custom `--workspace` flags continue to work unchanged
- New users get a sensible default without configuration
- Tests verify both default and custom workspace scenarios
