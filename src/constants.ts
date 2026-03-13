// Dracolich v12 Constants
// Single source of truth for all configuration values

import { GovernanceConfig, ClaudeModel } from './types.js';

export const TIMEOUTS = {
  CLI_IDLE_MS: 600_000,        // 10 min idle timeout
  CLI_MAX_MS: 10_800_000,      // 3 hour max timeout
  ACTIVITY_CHECK_MS: 10_000,   // Check every 10s
  GOV_IDLE_MS: 300_000,        // 5 min idle for governance agents
  GOV_MAX_MS: 1_800_000,       // 30 min max for governance
  DECOMPOSER_IDLE_MS: 600_000, // 10 min idle for decomposer
  DECOMPOSER_MAX_MS: 10_800_000, // 3 hour max for decomposer
} as const;

export const CONCURRENCY = {
  MAX_AGENTS: 5,
} as const;

/**
 * Retry configuration for execWithRetry()
 *
 * These values are based on:
 * - MAX_ATTEMPTS: 3 is standard for idempotent operations (AWS SDK default)
 * - BASE_DELAY_MS: 1000ms allows transient network issues to resolve
 * - MAX_DELAY_MS: 10000ms caps delay to avoid excessive wait times
 *
 * Latency tradeoff: Exponential backoff (1s + 2s + 4s = 7s worst-case)
 *
 * Error classification for retryable errors:
 * - "rate limit": API throttling, retry after backoff
 * - "network": Transient connectivity issues
 * - "timeout": Process took too long, may succeed on retry
 * - "ECONNRESET": Connection was reset by peer
 * - "ETIMEDOUT": Connection timed out
 *
 * Non-retryable errors (semantic failures):
 * - "invalid prompt": Bad input, retry won't help
 * - "authentication": Credentials issue
 * - Exit codes from application logic
 */
export const RETRY = {
  MAX_ATTEMPTS: 3,
  BASE_DELAY_MS: 1000,
  MAX_DELAY_MS: 10000,
  /** Error substrings that indicate a transient, retryable failure */
  RETRYABLE_PATTERNS: [
    'rate limit',
    'network',
    'timeout',
    'ECONNRESET',
    'ETIMEDOUT',
    'ENOTFOUND',
    'socket hang up',
  ] as readonly string[],
} as const;

export const GOVERNANCE_DEFAULTS: GovernanceConfig = {
  enabled: true,
  arbiterEnabled: true,
  reaperEnabled: true,
  model: ClaudeModel.Sonnet,
  maxIterations: 10,
};

export const GRACEFUL_SHUTDOWN_DELAY_MS = 1000;
export const FORCE_KILL_DELAY_MS = 1500;

export const FILE_LIMITS = {
  MAX_DIR_DEPTH: 100,
  MAX_FILE_SIZE_FOR_REVIEW: 50_000,
  MAX_FILE_PREVIEW_CHARS: 5_000,
  MAX_FILE_SIZE: 100 * 1024 * 1024, // 100MB
} as const;

export const CONTEXT_LIMITS = {
  // Max task length to prevent accidental file pastes (1MB safety limit)
  TASK_CHARS: 1_000_000,

  // Token budget for REAPER review (prevents truncation mid-analysis)
  REAPER_OUTPUT_CHARS: 15_000,

  // Token budget for REVISER (includes original output + critique)
  REVISER_OUTPUT_CHARS: 25_000,

  // Token budget for FIXER (file operations need less context)
  FIXER_OUTPUT_CHARS: 20_000,
} as const;
