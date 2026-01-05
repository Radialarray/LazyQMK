# Manual Testing Guide for dev.mjs

This guide explains how to manually verify that `pnpm dev:web` is stable and handles edge cases correctly.

## Prerequisites

```bash
cd web
pnpm install
```

## Test Scenarios

### 1. Normal Operation (Happy Path)

**Test:** Start dev server and verify both services run continuously

```bash
pnpm dev:web
```

**Expected:**
- ✅ Backend starts on http://localhost:3001
- ✅ Frontend starts on http://localhost:5173
- ✅ Both services keep running (no unexpected exits)
- ✅ No error messages in console
- ✅ Browser opens automatically to frontend

**Manual verification:**
- Wait 30 seconds - both services should still be running
- Open http://localhost:5173 in browser - should load the UI
- Check http://localhost:3001/health - should return 200 OK

### 2. Clean Shutdown (Ctrl+C)

**Test:** Stop services gracefully with Ctrl+C

```bash
pnpm dev:web
# Wait for both services to start
# Press Ctrl+C once
```

**Expected:**
- ✅ See "[CLEANUP] Stopping all processes..." message
- ✅ Both processes terminate within 1 second
- ✅ Script exits cleanly (no errors)
- ✅ Only one cleanup message (no cascading cleanup)
- ✅ No "Exited with code 143" error message

**Verify:**
```bash
# Check ports are free after shutdown
lsof -ti:3001  # Should return nothing
lsof -ti:5173  # Should return nothing
```

### 3. Port Conflict (Backend)

**Test:** Start backend on port 3001 separately, then run dev script

```bash
# Terminal 1: Start backend manually
cd ..
cargo run --features web --bin lazyqmk-web -- --port 3001

# Terminal 2: Try to run dev script
cd web
pnpm dev:web
```

**Expected:**
- ❌ Backend fails to start (port already in use)
- ✅ Clear error message: "Port 3001 may already be in use"
- ✅ Helpful instructions: "Run: lsof -ti:3001 | xargs kill"
- ✅ Script exits cleanly without infinite loop
- ✅ No cascading cleanup messages

**Cleanup:**
```bash
# Stop the manually started backend
lsof -ti:3001 | xargs kill
```

### 4. Port Conflict (Frontend)

**Test:** Start Vite on port 5173 separately, then run dev script

```bash
# Terminal 1: Start frontend manually
cd web
pnpm dev

# Terminal 2: Try to run dev script
pnpm dev:web
```

**Expected:**
- ✅ Backend starts successfully
- ⚠️ Frontend may fail or use alternate port
- ✅ Script handles error gracefully
- ✅ No infinite cleanup loop

**Cleanup:**
```bash
# Stop the manually started frontend
lsof -ti:5173 | xargs kill
```

### 5. Missing Dependencies (cargo)

**Test:** Temporarily rename cargo to simulate missing Rust toolchain

```bash
# Temporarily make cargo unavailable (if safe to do so)
# or just observe behavior if cargo is genuinely not installed

pnpm dev:web
```

**Expected:**
- ❌ Backend fails to start
- ✅ Clear error message: "Failed to start: cargo: command not found"
- ✅ Helpful hint: "Make sure Rust and cargo are installed"
- ✅ Script exits cleanly

### 6. Missing Dependencies (pnpm/npm)

**Test:** This would require removing node_modules, but is covered by error handling

```bash
rm -rf node_modules
pnpm dev:web
```

**Expected:**
- ❌ Frontend fails to start
- ✅ Clear error message about missing dependencies
- ✅ Script exits cleanly

**Cleanup:**
```bash
pnpm install
```

## Automated Tests

The dev.test.mjs file validates internal logic:

```bash
pnpm test:dev-script
```

**Tests:**
- Cleanup guard prevents multiple calls
- Exit code 143 (SIGTERM) is properly ignored
- Non-zero exit codes trigger cleanup
- Platform detection works
- Process kill errors are handled

## Success Criteria

All manual tests should:
- ✅ Show clear, helpful error messages
- ✅ Exit cleanly without infinite loops
- ✅ Not show "Exited with code 143" errors during normal Ctrl+C shutdown
- ✅ Display only one "[CLEANUP]" message per shutdown
- ✅ Free ports after exit (verify with lsof/netstat)

## Platform-Specific Notes

### macOS/Linux
- Uses `lsof -ti:<port>` to check ports
- Uses SIGTERM for graceful shutdown
- Exit code 143 = SIGTERM (128 + 15)

### Windows
- Uses `netstat -ano | findstr :<port>` to check ports
- Uses `taskkill /f /t` for process tree termination
- May have different exit codes for termination

## Troubleshooting

If tests fail:
1. Check `git diff web/dev.mjs` - ensure all fixes are applied
2. Verify `cleanupInProgress` guard is in place
3. Verify exit code 143 is excluded from cleanup triggers
4. Verify `process.on('exit')` handler is removed
5. Check that both backend and frontend exit handlers check `cleanupInProgress`
