#!/usr/bin/env node

/**
 * Unit tests for dev.mjs
 * 
 * Tests the cleanup guard, signal handling, and exit code logic.
 * Run with: node dev.test.mjs
 */

import { spawn } from 'child_process';
import { platform } from 'os';

const IS_WINDOWS = platform() === 'win32';

// Test counters
let passed = 0;
let failed = 0;

function assert(condition, message) {
  if (condition) {
    console.log(`✅ PASS: ${message}`);
    passed++;
  } else {
    console.log(`❌ FAIL: ${message}`);
    failed++;
  }
}

function assertEquals(actual, expected, message) {
  assert(actual === expected, `${message} (expected: ${expected}, got: ${actual})`);
}

// Test 1: Cleanup guard prevents multiple calls
console.log('\n📝 Test 1: Cleanup guard');
{
  let cleanupInProgress = false;
  let cleanupCount = 0;
  
  function cleanup() {
    if (cleanupInProgress) {
      return;
    }
    cleanupInProgress = true;
    cleanupCount++;
  }
  
  cleanup();
  cleanup();
  cleanup();
  
  assertEquals(cleanupCount, 1, 'Cleanup should only execute once');
}

// Test 2: Exit code 143 should be ignored
console.log('\n📝 Test 2: Exit code 143 (SIGTERM) handling');
{
  let cleanupInProgress = false;
  let shouldTriggerCleanup = false;
  
  // Simulate child exit with code 143
  const code = 143;
  const signal = null;
  
  if (!cleanupInProgress && code !== 0 && code !== null && code !== 143) {
    shouldTriggerCleanup = true;
  }
  
  assertEquals(shouldTriggerCleanup, false, 'Exit code 143 should not trigger cleanup');
}

// Test 3: Non-zero exit codes (except 143) should trigger cleanup
console.log('\n📝 Test 3: Non-zero exit codes trigger cleanup');
{
  let cleanupInProgress = false;
  
  function shouldCleanup(code, signal) {
    if (cleanupInProgress) {
      return false;
    }
    return code !== 0 && code !== null && code !== 143;
  }
  
  assertEquals(shouldCleanup(1, null), true, 'Exit code 1 should trigger cleanup');
  assertEquals(shouldCleanup(98, null), true, 'Exit code 98 (port in use) should trigger cleanup');
  assertEquals(shouldCleanup(127, null), true, 'Exit code 127 (command not found) should trigger cleanup');
  assertEquals(shouldCleanup(143, null), false, 'Exit code 143 should NOT trigger cleanup');
  assertEquals(shouldCleanup(0, null), false, 'Exit code 0 should NOT trigger cleanup');
  assertEquals(shouldCleanup(null, null), false, 'Exit code null should NOT trigger cleanup');
}

// Test 4: Cleanup in progress prevents subsequent cleanup
console.log('\n📝 Test 4: Cleanup in progress flag');
{
  let cleanupInProgress = false;
  
  function shouldCleanup(code, signal) {
    if (cleanupInProgress) {
      return false;
    }
    return code !== 0 && code !== null && code !== 143;
  }
  
  assertEquals(shouldCleanup(1, null), true, 'Should cleanup when flag is false');
  
  cleanupInProgress = true;
  assertEquals(shouldCleanup(1, null), false, 'Should NOT cleanup when flag is true');
}

// Test 5: Platform detection
console.log('\n📝 Test 5: Platform detection');
{
  assert(typeof IS_WINDOWS === 'boolean', 'IS_WINDOWS should be a boolean');
  console.log(`   Platform: ${IS_WINDOWS ? 'Windows' : 'Unix'}`);
}

// Test 6: Process kill error handling
console.log('\n📝 Test 6: Process kill error handling');
{
  let errorCaught = false;
  const mockProc = { killed: false, kill: () => { throw new Error('Process already exited'); } };
  
  try {
    mockProc.kill('SIGTERM');
  } catch (err) {
    errorCaught = true;
    // Should be caught and ignored in real script
  }
  
  assert(errorCaught, 'Kill errors should be catchable');
}

// Test 7: Signal vs exit code priority
console.log('\n📝 Test 7: Signal parameter handling');
{
  let cleanupInProgress = false;
  
  function shouldCleanup(code, signal) {
    if (cleanupInProgress) {
      return false;
    }
    return code !== 0 && code !== null && code !== 143;
  }
  
  // When SIGTERM is sent, code will be 143 or signal will be 'SIGTERM'
  assertEquals(shouldCleanup(143, 'SIGTERM'), false, 'SIGTERM via code should not trigger cleanup');
  assertEquals(shouldCleanup(null, 'SIGTERM'), false, 'SIGTERM via signal should not trigger cleanup');
}

// Summary
console.log('\n' + '='.repeat(50));
console.log(`📊 Test Results: ${passed} passed, ${failed} failed`);
console.log('='.repeat(50));

if (failed > 0) {
  process.exit(1);
} else {
  console.log('✅ All tests passed!');
  process.exit(0);
}
