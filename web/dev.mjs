#!/usr/bin/env node

/**
 * LazyQMK Web Development Script
 * 
 * Starts both Rust backend and Vite frontend for local development.
 * Cross-platform: works on macOS, Linux, and Windows.
 * 
 * Usage:
 *   npm run dev:web      (from project root)
 *   pnpm dev:web         (from project root)
 *   node dev.mjs         (from web/ directory)
 */

import { spawn } from 'child_process';
import { platform } from 'os';

const IS_WINDOWS = platform() === 'win32';

// Colors for terminal output
const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  cyan: '\x1b[36m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  red: '\x1b[31m',
};

function log(color, prefix, message) {
  console.log(`${colors.bright}${color}[${prefix}]${colors.reset} ${message}`);
}

// Track child processes for cleanup
const processes = [];
let cleanupInProgress = false;

function cleanup() {
  // Guard against multiple cleanup calls
  if (cleanupInProgress) {
    return;
  }
  cleanupInProgress = true;
  
  log(colors.yellow, 'CLEANUP', 'Stopping all processes...');
  
  processes.forEach(proc => {
    if (proc && !proc.killed) {
      try {
        if (IS_WINDOWS) {
          // Windows: kill process tree
          spawn('taskkill', ['/pid', proc.pid, '/f', '/t'], { shell: true });
        } else {
          // Unix: send SIGTERM
          proc.kill('SIGTERM');
        }
      } catch (err) {
        // Process may have already exited, ignore errors
      }
    }
  });
  
  setTimeout(() => {
    process.exit(0);
  }, 1000);
}

// Handle cleanup on signals only (not on 'exit' to avoid cascading cleanup)
process.on('SIGINT', cleanup);
process.on('SIGTERM', cleanup);

function startBackend() {
  log(colors.cyan, 'BACKEND', 'Starting Rust backend on http://localhost:3001...');
  
  const backendArgs = [
    'run',
    '--features', 'web',
    '--bin', 'lazyqmk-web',
    '--',
    '--port', '3001'
  ];
  
  const backend = spawn('cargo', backendArgs, {
    cwd: '..',
    stdio: 'inherit',
    shell: IS_WINDOWS,
  });
  
  processes.push(backend);
  
  backend.on('error', (err) => {
    log(colors.red, 'BACKEND', `Failed to start: ${err.message}`);
    log(colors.yellow, 'BACKEND', 'Make sure Rust and cargo are installed');
    cleanup();
  });
  
  backend.on('exit', (code, signal) => {
    // Don't trigger cleanup if process was terminated by our cleanup (SIGTERM = code 143 or signal 'SIGTERM')
    // or if cleanup is already in progress
    if (cleanupInProgress) {
      return;
    }
    
    if (code !== 0 && code !== null && code !== 143) {
      // Check for common errors and provide helpful messages
      log(colors.red, 'BACKEND', `Exited with code ${code}`);
      
      if (code === 98 || (signal === null && code !== null)) {
        // Address already in use (Unix: 98, Windows varies)
        log(colors.yellow, 'BACKEND', 'Port 3001 may already be in use');
        log(colors.yellow, 'BACKEND', 'Run: lsof -ti:3001 | xargs kill (macOS/Linux)');
        log(colors.yellow, 'BACKEND', 'Or check DEV_SCRIPT.md for Windows instructions');
      }
      
      cleanup();
    }
  });
  
  return backend;
}

function startFrontend() {
  log(colors.green, 'FRONTEND', 'Starting Vite dev server on http://localhost:5173...');
  
  // Use pnpm if available, fallback to npm
  const packageManager = process.env.npm_execpath?.includes('pnpm') ? 'pnpm' : 'npm';
  
  const frontend = spawn(packageManager, ['run', 'dev'], {
    stdio: 'inherit',
    shell: IS_WINDOWS,
  });
  
  processes.push(frontend);
  
  frontend.on('error', (err) => {
    log(colors.red, 'FRONTEND', `Failed to start: ${err.message}`);
    log(colors.yellow, 'FRONTEND', 'Make sure dependencies are installed (npm install)');
    cleanup();
  });
  
  frontend.on('exit', (code, signal) => {
    // Don't trigger cleanup if process was terminated by our cleanup (SIGTERM = code 143 or signal 'SIGTERM')
    // or if cleanup is already in progress
    if (cleanupInProgress) {
      return;
    }
    
    if (code !== 0 && code !== null && code !== 143) {
      log(colors.red, 'FRONTEND', `Exited with code ${code}`);
      cleanup();
    }
  });
  
  return frontend;
}

// Main execution
async function main() {
  console.log(`${colors.bright}${colors.cyan}╔════════════════════════════════════════════════╗${colors.reset}`);
  console.log(`${colors.bright}${colors.cyan}║     LazyQMK Web Development Environment        ║${colors.reset}`);
  console.log(`${colors.bright}${colors.cyan}╚════════════════════════════════════════════════╝${colors.reset}`);
  console.log();
  
  log(colors.green, 'INFO', 'Starting backend and frontend...');
  log(colors.green, 'INFO', 'Press Ctrl+C to stop both services');
  console.log();
  
  // Start backend first (frontend depends on it)
  startBackend();
  
  // Wait a bit for backend to start
  log(colors.yellow, 'INFO', 'Waiting for backend to initialize...');
  await new Promise(resolve => setTimeout(resolve, 2000));
  
  // Start frontend
  startFrontend();
  
  console.log();
  log(colors.green, 'READY', 'Development servers starting...');
  log(colors.green, 'READY', 'Backend:  http://localhost:3001');
  log(colors.green, 'READY', 'Frontend: http://localhost:5173 (opens automatically)');
  console.log();
}

main().catch(err => {
  log(colors.red, 'ERROR', err.message);
  cleanup();
});
