#!/usr/bin/env npx tsx
/**
 * Dracolich v12 - Test Suite
 *
 * Test Categories:
 * 1. Unit tests - Pure function testing with no external dependencies
 * 2. Mock integration tests - Tests with MockPool (validates logic, NOT subprocess behavior)
 * 3. Real CLI integration test - Validates actual claude subprocess works (requires claude CLI)
 *
 * IMPORTANT: MockPool tests validate the LOGIC of retry/governance code but do NOT
 * verify actual subprocess spawning, stdio handling, or Claude CLI behavior.
 * The real CLI integration test addresses this gap.
 *
 * Coverage notes:
 * - JSON parsing: Comprehensive (7 tests)
 * - Retry logic: Comprehensive (4 tests with MockPool)
 * - Governance: Basic disabled-path tests (3 tests)
 * - Path utils: Comprehensive (4 tests)
 * - Real CLI: Smoke test only (1 test)
 *
 * For production: Add more real CLI tests for error scenarios.
 */

import { parseGovernanceJson } from './utils/json-parser.js';
import { ensurePathWithinBase, generateProjectName } from './utils/path-utils.js';
import { execWithRetry, CliPool, isRetryableError } from './pool.js';
import { validateSwarmDesign, reviewDeliverables, runGovernanceLoop } from './governance.js';
import {
  CliResult, CliOptions, ExecutionPool, ClaudeModel, SwarmDesign,
  ArbiterVerdict, ReaperVerdict, GovernanceConfig, getCliError
} from './types.js';
import { RETRY } from './constants.js';

interface TestResult {
  name: string;
  passed: boolean;
  error?: string;
}

const results: TestResult[] = [];

async function test(name: string, fn: () => void | Promise<void>): Promise<void> {
  try {
    await fn();
    results.push({ name, passed: true });
    console.log(`[TEST] ✓ ${name}`);
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error);
    results.push({ name, passed: false, error: errorMsg });
    console.log(`[TEST] ✗ ${name}: ${errorMsg}`);
  }
}

function assertEqual<T>(actual: T, expected: T, msg = ''): void {
  if (actual !== expected) {
    throw new Error(`${msg ? msg + ': ' : ''}Expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
}

function assertThrows(fn: () => void, msg = ''): void {
  let threw = false;
  try { fn(); } catch { threw = true; }
  if (!threw) throw new Error(msg || 'Expected function to throw');
}

// ==================== Mock Pool for Integration Tests ====================
/**
 * MockPool provides deterministic responses for testing retry logic and governance.
 *
 * LIMITATIONS (important for understanding test coverage):
 * - Does NOT spawn real subprocesses (no stdio, no process lifecycle)
 * - Does NOT validate actual Claude CLI behavior or output format
 * - Does NOT test network/rate-limit handling in production scenarios
 *
 * Use MockPool for: Testing retry LOGIC, governance decision paths, error handling
 * Use Real CLI for: Validating subprocess spawning, integration with claude binary
 */
class MockPool implements ExecutionPool {
  public calls: Array<{ prompt: string; options: CliOptions }> = [];
  private responses: CliResult[] = [];
  private responseIndex = 0;

  setResponses(responses: CliResult[]) {
    this.responses = responses;
    this.responseIndex = 0;
  }

  async exec(prompt: string, options: CliOptions): Promise<CliResult> {
    this.calls.push({ prompt, options });
    const response = this.responses[this.responseIndex++];
    return response ?? { success: true, output: 'mock output', durationMs: 100 };
  }

  async shutdown(): Promise<void> {}
}

// ==================== Run All Tests ====================

async function runAllTests() {
  // JSON Utils Tests
  await test('parseGovernanceJson: code block JSON', () => {
    const input = 'Some text ```json\n{"key": "value"}\n``` more text';
    const result = parseGovernanceJson<{ key: string }>(input, 'TEST', { key: 'fallback' }, ['key']);
    assertEqual(result.key, 'value');
  });

  await test('parseGovernanceJson: raw JSON', () => {
    const input = 'Here is the result: {"foo": 123, "bar": true}';
    const result = parseGovernanceJson<{ foo: number; bar: boolean }>(input, 'TEST', { foo: 0, bar: false }, ['foo', 'bar']);
    assertEqual(result.foo, 123);
    assertEqual(result.bar, true);
  });

  await test('parseGovernanceJson: nested JSON', () => {
    const input = '{"outer": {"inner": {"deep": 42}}}';
    const result = parseGovernanceJson<{ outer: { inner: { deep: number } } }>(
      input, 'TEST', { outer: { inner: { deep: 0 } } }, ['outer']
    );
    assertEqual(result.outer.inner.deep, 42);
  });

  await test('parseGovernanceJson: JSON with strings containing braces', () => {
    const input = '{"text": "Hello {world}"}';
    const result = parseGovernanceJson<{ text: string }>(input, 'TEST', { text: 'fallback' }, ['text']);
    assertEqual(result.text, 'Hello {world}');
  });

  await test('parseGovernanceJson: falls back on no JSON', () => {
    const result = parseGovernanceJson<{ key: string }>(
      'no json here', 'TEST', { key: 'fallback' }, ['key']
    );
    assertEqual(result.key, 'fallback');
  });

  await test('parseGovernanceJson: valid input with all fields', () => {
    const input = '```json\n{"decision": "APPROVE", "findings": [], "summary": "All good"}\n```';
    const result = parseGovernanceJson<{ decision: string; findings: string[]; summary: string }>(
      input, 'TEST',
      { decision: 'FALLBACK', findings: [], summary: '' },
      ['decision', 'findings', 'summary']
    );
    assertEqual(result.decision, 'APPROVE');
    assertEqual(result.summary, 'All good');
  });

  await test('parseGovernanceJson: falls back on missing required field', () => {
    const input = '{"decision": "APPROVE"}';
    const fallback = { decision: 'FALLBACK', findings: ['default'], summary: 'fallback used' };
    const result = parseGovernanceJson<typeof fallback>(input, 'TEST', fallback, ['decision', 'findings', 'summary']);
    assertEqual(result.decision, 'FALLBACK');
    assertEqual(result.summary, 'fallback used');
  });

  // Pool Tests
  await test('execWithRetry: returns on first success', async () => {
    const pool = new MockPool();
    pool.setResponses([{ success: true, output: 'done', durationMs: 50 }]);
    const result = await execWithRetry(pool, 'test prompt', { model: ClaudeModel.Sonnet });
    assertEqual(result.success, true);
    assertEqual(pool.calls.length, 1);
  });

  await test('execWithRetry: retries on rate limit error', async () => {
    const pool = new MockPool();
    pool.setResponses([
      { success: false, output: '', error: 'rate limit exceeded', durationMs: 10 },
      { success: true, output: 'success after retry', durationMs: 50 }
    ]);
    const result = await execWithRetry(pool, 'test prompt', { model: ClaudeModel.Sonnet });
    assertEqual(result.success, true);
    assertEqual(pool.calls.length, 2);
  });

  await test('execWithRetry: does not retry on non-retryable error', async () => {
    const pool = new MockPool();
    pool.setResponses([
      { success: false, output: '', error: 'invalid prompt', durationMs: 10 }
    ]);
    const result = await execWithRetry(pool, 'test prompt', { model: ClaudeModel.Sonnet });
    assertEqual(result.success, false);
    assertEqual(pool.calls.length, 1);
  });

  await test('execWithRetry: gives up after max attempts', async () => {
    const pool = new MockPool();
    const failures = Array(RETRY.MAX_ATTEMPTS).fill(null).map(() =>
      ({ success: false as const, output: '', error: 'rate limit exceeded', durationMs: 10 })
    );
    pool.setResponses(failures);
    const result = await execWithRetry(pool, 'test prompt', { model: ClaudeModel.Sonnet });
    assertEqual(result.success, false);
    assertEqual(pool.calls.length, RETRY.MAX_ATTEMPTS);
  });

  // Governance Tests
  await test('validateSwarmDesign: returns approve when governance disabled', async () => {
    const pool = new MockPool();
    const design: SwarmDesign = {
      agents: [{ name: 'TEST', role: 'tester', systemPrompt: 'test' }],
      groups: [[{ id: 't1', description: 'test task', agent: 'TEST', dependsOn: [] }]],
      reasoning: 'test design'
    };
    const config: GovernanceConfig = {
      enabled: false,
      arbiterEnabled: true,
      reaperEnabled: true,
      model: ClaudeModel.Sonnet,
      maxIterations: 3
    };
    const decision = await validateSwarmDesign(design, 'test task', config, pool);
    assertEqual(decision.decision, ArbiterVerdict.Approve);
    assertEqual(pool.calls.length, 0);
  });

  await test('reviewDeliverables: returns approve when governance disabled', async () => {
    const pool = new MockPool();
    const config: GovernanceConfig = {
      enabled: true,
      arbiterEnabled: true,
      reaperEnabled: false,
      model: ClaudeModel.Sonnet,
      maxIterations: 3
    };
    const review = await reviewDeliverables('test task', 'test output', config, pool);
    assertEqual(review.decision, ReaperVerdict.Approve);
    assertEqual(review.summary, 'Governance disabled');
    assertEqual(pool.calls.length, 0);
  });

  await test('runGovernanceLoop: returns immediately when disabled', async () => {
    const pool = new MockPool();
    const config: GovernanceConfig = {
      enabled: false,
      arbiterEnabled: true,
      reaperEnabled: true,
      model: ClaudeModel.Sonnet,
      maxIterations: 3
    };
    const result = await runGovernanceLoop('test', 'initial output', config, {
      projectDir: '/tmp',
      hasBuildArtifacts: false,
      pool
    });
    assertEqual(result.finalOutput, 'initial output');
    assertEqual(result.iterations, 0);
    assertEqual(result.maxIterationsReached, false);
    assertEqual(pool.calls.length, 0);
  });

  // Constants Tests
  await test('RETRY constants are valid', () => {
    if (RETRY.MAX_ATTEMPTS < 1) throw new Error('MAX_ATTEMPTS must be >= 1');
    if (RETRY.BASE_DELAY_MS < 0) throw new Error('BASE_DELAY_MS must be >= 0');
    if (RETRY.MAX_DELAY_MS < RETRY.BASE_DELAY_MS) throw new Error('MAX_DELAY_MS must be >= BASE_DELAY_MS');
    if (RETRY.RETRYABLE_PATTERNS.length === 0) throw new Error('RETRYABLE_PATTERNS must have at least one pattern');
  });

  // isRetryableError Tests (validates the extracted error classification)
  await test('isRetryableError: returns true for rate limit errors', () => {
    assertEqual(isRetryableError('rate limit exceeded'), true);
    assertEqual(isRetryableError('Rate Limit hit'), true); // Case insensitive
  });

  await test('isRetryableError: returns true for network errors', () => {
    assertEqual(isRetryableError('network error occurred'), true);
    assertEqual(isRetryableError('ECONNRESET'), true);
    assertEqual(isRetryableError('socket hang up'), true);
  });

  await test('isRetryableError: returns false for semantic errors', () => {
    assertEqual(isRetryableError('invalid prompt'), false);
    assertEqual(isRetryableError('authentication failed'), false);
    assertEqual(isRetryableError('Exit code 1'), false);
  });

  // Path Utils Tests
  await test('generateProjectName: creates valid names', () => {
    const name = generateProjectName('Test task with spaces & symbols!');
    if (name.includes(' ')) throw new Error('Name should not contain spaces');
    if (name.length === 0) throw new Error('Name should not be empty');
    if (name.length >= 100) throw new Error('Name should be reasonably short');
  });

  await test('ensurePathWithinBase: rejects path traversal', () => {
    assertThrows(() => ensurePathWithinBase('/tmp', '../etc/passwd'), 'Should reject path traversal');
  });

  await test('ensurePathWithinBase: accepts valid paths', () => {
    assertEqual(ensurePathWithinBase('/tmp', 'valid-project'), '/tmp/valid-project');
  });

  await test('ensurePathWithinBase: rejects slash in project name', () => {
    assertThrows(() => ensurePathWithinBase('/tmp', 'invalid/path'), 'Should reject slash in project name');
  });

  // ==================== Real CLI Integration Test ====================
  // This test validates actual subprocess spawning works.
  // Skip if claude CLI is not installed (CI environments without claude).

  await test('INTEGRATION: CliPool spawns real claude process (smoke test)', async () => {
    const pool = new CliPool();
    try {
      // Minimal prompt that should complete quickly with haiku
      const result = await pool.exec('Reply with exactly: PING', {
        model: ClaudeModel.Haiku,
        printOnly: true,
        maxTimeoutMs: 30000, // 30 second timeout for simple task
      });

      // We expect either:
      // 1. success: true - claude CLI worked
      // 2. success: false with specific error about missing CLI - expected in CI
      if (!result.success) {
        const errorMsg = getCliError(result);
        // If claude isn't installed, that's okay for CI - mark as expected
        if (errorMsg.includes('spawn claude') || errorMsg.includes('ENOENT') || errorMsg.includes('not found')) {
          console.log('  [INTEGRATION] claude CLI not installed - skipping (expected in CI)');
          return; // Test passes - we validated spawn handling works
        }
        // Any other error is unexpected
        throw new Error(`Unexpected CLI error: ${errorMsg}`);
      }

      // If we got here, claude responded
      const hasOutput = result.output && result.output.length > 0;
      if (!hasOutput) {
        throw new Error('claude returned empty output');
      }

      console.log(`  [INTEGRATION] Real CLI responded: ${result.output.substring(0, 50)}...`);
    } finally {
      await pool.shutdown();
    }
  });

  // Print results
  console.log('\n' + '='.repeat(60));
  console.log('TEST RESULTS');
  console.log('='.repeat(60));

  const passed = results.filter(r => r.passed).length;
  const failed = results.filter(r => !r.passed).length;

  console.log(`\nPassed: ${passed}/${results.length}`);
  console.log(`Failed: ${failed}/${results.length}`);

  if (failed > 0) {
    console.log('\nFailed tests:');
    for (const r of results.filter(r => !r.passed)) console.log(`  - ${r.name}: ${r.error}`);
    process.exit(1);
  } else {
    console.log('\nAll tests passed!');
    process.exit(0);
  }
}

runAllTests().catch(error => {
  console.error('Test runner failed:', error);
  process.exit(1);
});
