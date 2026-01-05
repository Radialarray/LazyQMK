# Development Script

The `dev.mjs` script provides a **cross-platform** way to start both the Rust backend and Vite frontend with a single command.

## Usage

```bash
# From web/ directory
pnpm dev:web
# or
npm run dev:web

# Or run directly
node dev.mjs
```

## Testing

The dev script includes unit tests to ensure stability and correct signal handling:

```bash
# Run all tests (including dev script tests)
pnpm test

# Run only dev script tests
pnpm test:dev-script

# Run only vitest unit tests
pnpm test:unit
```

The tests validate:
- ✅ Cleanup guard prevents multiple cleanup calls
- ✅ Exit code 143 (SIGTERM) is properly ignored
- ✅ Non-zero exit codes trigger cleanup
- ✅ Cleanup in progress flag prevents cascading cleanup
- ✅ Platform detection works correctly
- ✅ Process kill errors are handled gracefully

## What It Does

1. **Starts Rust backend** on `http://localhost:3001`
   - Runs: `cargo run --features web --bin lazyqmk-web -- --port 3001`
   - Working directory: `../` (project root)

2. **Waits for backend to be ready**
   - Polls `/health` endpoint every second (up to 30 seconds)
   - Provides progress updates every 5 attempts
   - Exits with error if backend doesn't start within timeout

3. **Starts Vite dev server** on `http://localhost:5173`
   - Runs: `pnpm dev` (or `npm run dev`)
   - Proxies `/api` and `/health` requests to backend (configured in `vite.config.ts`)
   - Only starts after backend health check succeeds

4. **Handles cleanup** on exit (Ctrl+C)
   - Gracefully terminates both processes
   - Works on Windows (taskkill) and Unix (SIGTERM)

## Platform Support

- ✅ **macOS** - Tested on Apple Silicon and Intel
- ✅ **Linux** - Works on all distributions with Node.js + Rust
- ✅ **Windows** - Uses `taskkill` for proper process cleanup

## Requirements

- **Node.js** 18+ (for running the script and Vite)
- **Rust** 1.91.1+ (for backend compilation)
- **pnpm** or **npm** (for installing dependencies)

## Troubleshooting

### Backend fails to start

**Error:** `Failed to start: cargo: command not found`

**Solution:** Install Rust and add `cargo` to PATH:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Backend takes too long to start

**Error:** `Backend never became healthy. Check logs above for errors.`

**Possible causes:**
- Backend compilation is slow (first run or after clean)
- Port 3001 is already in use
- Cargo configuration issues

**Solution:** Check backend logs above the error. Common fixes:
```bash
# Clean and rebuild
cd ..
cargo clean
cargo build --features web --bin lazyqmk-web

# Or manually start backend to see detailed errors
cargo run --features web --bin lazyqmk-web -- --port 3001
```

### Frontend fails to start

**Error:** `Failed to start: spawn npm ENOENT`

**Solution:** Install Node.js dependencies first:
```bash
cd web
pnpm install  # or npm install
```

### Port already in use

**Error:** `Address already in use (os error 48)`

**Solution:** Kill existing processes on ports 3001 or 5173:
```bash
# macOS/Linux
lsof -ti:3001 | xargs kill
lsof -ti:5173 | xargs kill

# Windows
netstat -ano | findstr :3001
taskkill /PID <PID> /F
```

## Alternative: Manual Startup

If you prefer to run processes separately:

```bash
# Terminal 1: Backend
cd ..
cargo run --features web --bin lazyqmk-web

# Terminal 2: Frontend
cd web
pnpm dev
```

## Implementation Details

The script uses Node.js `child_process.spawn` to launch both services:

- **Backend:** Spawns `cargo run` in parent directory (`../`)
- **Health check:** Polls `GET /health` every 1 second (max 30 attempts) before starting frontend
- **Frontend:** Spawns `pnpm dev` (or `npm run dev`) in current directory (only after backend is healthy)
- **Process management:** Tracks all child processes and cleans up on exit
- **Platform detection:** Uses `os.platform()` to handle Windows vs. Unix differences
- **Color output:** ANSI escape codes for clear, colored terminal output
- **Cleanup guard:** Prevents cascading cleanup calls when child processes exit due to SIGTERM
- **Exit code 143:** Properly handled (SIGTERM exit code) to avoid triggering cleanup loops
- **Signal handling:** Responds to SIGINT (Ctrl+C) and SIGTERM, but not the 'exit' event to prevent cascading

No external dependencies required - uses only Node.js built-in modules (`child_process`, `os`, `fetch`).

## Behavior Notes

### Graceful Shutdown
- Press Ctrl+C once to stop both services
- The script sends SIGTERM to child processes and waits 1 second before exiting
- Exit code 143 (SIGTERM) is treated as normal shutdown, not an error

### Error Handling
- If backend fails to start, the script displays a clear error and exits cleanly
- Port conflict detection provides platform-specific instructions
- Cleanup guard prevents infinite loops if multiple exit events fire

### Cross-Platform Compatibility
- **Unix (macOS/Linux):** Uses `proc.kill('SIGTERM')` for graceful termination
- **Windows:** Uses `taskkill /f /t` to terminate process tree
- **Exit codes:** Handles platform differences (e.g., Unix exit 143 for SIGTERM)
