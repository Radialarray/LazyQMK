# Vite Dev Mode Preview Rendering Fixes

## Problem Statement

When running `pnpm dev:web` on cold start, the keyboard preview would not render without a manual refresh. This was caused by:

1. **Fixed 2-second sleep in dev.mjs**: The script waited a fixed 2 seconds for the backend to start, but Rust compilation could take longer on cold start.
2. **No retry logic in loadGeometry**: If the frontend loaded before the backend was fully ready, geometry API calls would fail with no retry attempt.

## Solution

### 1. Health Check Polling in dev.mjs

**File:** `web/dev.mjs`

**Changes:**
- Replaced fixed `setTimeout(2000)` with health check polling
- Added `waitForBackend()` function that:
  - Polls `http://localhost:3001/health` every 1 second
  - Attempts up to 30 times (30 seconds total)
  - Logs progress every 5 attempts to avoid console spam
  - Exits with clear error message if backend never becomes healthy
  - Only starts Vite after backend health check succeeds

**Benefits:**
- Works reliably on cold start (first compile takes 10-15 seconds)
- Provides clear feedback during startup
- Fails fast with helpful error message if backend crashes
- No wasted wait time on warm starts (backend ready in <2 seconds)

### 2. Retry Logic in loadGeometry

**File:** `web/src/routes/layouts/[name]/+page.svelte`

**Changes:**
- Added retry mechanism with 3 attempts and 500ms delay between retries
- Added console logging for debugging:
  - Logs each attempt with attempt number
  - Logs success with key count
  - Logs warnings on transient failures
  - Logs final error only after all retries exhausted
- Only logs "Backend may still be starting" message on first retry to avoid spam

**Benefits:**
- Handles race conditions where frontend loads slightly before backend is fully ready
- Provides visibility into fetch operations via console logs
- Graceful degradation with clear error messages
- No impact on successful loads (single attempt succeeds immediately)

## Configuration

Both mechanisms use conservative defaults that can be adjusted:

**dev.mjs:**
```javascript
await waitForBackend(maxAttempts = 30, intervalMs = 1000)
```

**+page.svelte:**
```javascript
const maxRetries = 3;
const retryDelayMs = 500;
```

## Testing

### Unit Tests
```bash
pnpm test:unit      # 121 tests passed
pnpm test:dev-script # 14 tests passed
```

### E2E Tests
```bash
pnpm test:e2e       # 84 tests passed, 6 skipped
```

### Manual Testing

**Cold start (backend not compiled):**
```bash
# Clean state
cd .. && cargo clean && cd web

# Start dev environment
pnpm dev:web

# Expected behavior:
# 1. Backend starts compiling
# 2. Health check polls with progress updates every 5 seconds
# 3. Once backend is healthy, Vite starts
# 4. Frontend loads and geometry API succeeds on first try
```

**Warm start (backend already compiled):**
```bash
pnpm dev:web

# Expected behavior:
# 1. Backend starts quickly (<2 seconds)
# 2. Health check succeeds immediately
# 3. Vite starts without delay
# 4. Frontend loads normally
```

**Backend failure scenario:**
```bash
# Simulate port conflict
lsof -ti:3001 # If port is in use

pnpm dev:web

# Expected behavior:
# 1. Backend fails to bind to port
# 2. Health check fails 30 times with progress updates
# 3. Script exits with error message:
#    "Backend never became healthy. Check logs above for errors."
```

## Acceptance Criteria

✅ **Running `pnpm dev:web` on cold start consistently renders keyboard preview without manual refresh**
- Verified with health check polling that waits for backend to be ready

✅ **If backend never comes up, dev script exits with clear message**
- Implemented timeout with helpful error message after 30 seconds

✅ **No flakiness introduced in existing e2e tests**
- All 84 e2e tests pass
- All 121 unit tests pass
- All 14 dev script tests pass

## Related Files

- `web/dev.mjs` - Dev script with health check polling
- `web/src/routes/layouts/[name]/+page.svelte` - Layout page with retry logic
- `web/DEV_SCRIPT.md` - Updated documentation
- `web/vite.config.ts` - Already had `/health` proxy configured (no changes needed)

## Future Improvements

1. **Configurable timeouts via environment variables:**
   ```bash
   BACKEND_TIMEOUT=60 pnpm dev:web  # Wait up to 60 seconds
   ```

2. **Exponential backoff for retries:**
   ```javascript
   const delay = baseDelay * Math.pow(2, attempt - 1);
   ```

3. **WebSocket health monitoring:**
   - Keep connection alive for instant failure detection
   - Automatically restart frontend if backend crashes

4. **Unified logging framework:**
   - Consistent format across dev.mjs and Svelte components
   - Toggle verbose mode for debugging
