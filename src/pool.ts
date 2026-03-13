// Dracolich v12 Claude CLI Subprocess Pool

import { spawn, ChildProcess } from 'child_process';
import { CliOptions, CliResult, ClaudeModel, ExecutionPool, getCliError } from './types.js';
import { TIMEOUTS, CONCURRENCY, RETRY } from './constants.js';

function validateModel(model: string): asserts model is ClaudeModel {
  if (!['haiku', 'sonnet', 'opus'].includes(model)) {
    throw new Error(`Invalid model: ${model}. Must be haiku, sonnet, or opus`);
  }
}

async function runClaude(
  prompt: string,
  options: CliOptions
): Promise<CliResult> {
  const start = Date.now();
  validateModel(options.model);

  const args = ['--dangerously-skip-permissions', '--model', options.model];
  if (options.printOnly) args.push('--print');
  args.push('-');

  const fullPrompt = options.systemPrompt
    ? `<system>\n${options.systemPrompt}\n</system>\n\n${prompt}`
    : prompt;

  const workingDir = options.workingDir ?? '/tmp';
  const timeout = options.maxTimeoutMs ?? TIMEOUTS.CLI_MAX_MS;

  let proc: ChildProcess;
  try {
    proc = spawn('claude', args, {
      stdio: ['pipe', 'pipe', 'pipe'],
      cwd: workingDir,
      timeout,
    });
  } catch (error) {
    return {
      success: false,
      output: '',
      error: `Failed to spawn claude: ${(error as Error).message}. Is claude CLI installed?`,
      durationMs: 0,
    };
  }

  if (!proc || !proc.pid || !proc.stdin || !proc.stdout || !proc.stderr) {
    if (proc && proc.pid) {
      try { proc.kill('SIGKILL'); } catch {}
    }
    return {
      success: false,
      output: '',
      error: 'Process spawned but stdio not available',
      durationMs: 0,
    };
  }

  return new Promise((resolve) => {
    let stdout = '';
    let stderr = '';
    let resolved = false;

    const finish = (result: CliResult) => {
      if (resolved) return;
      resolved = true;
      resolve(result);
    };

    proc.stdout?.on('data', (d) => { stdout += d.toString(); });
    proc.stderr?.on('data', (d) => { stderr += d.toString(); });

    proc.on('close', (code) => {
      const durationMs = Date.now() - start;
      if (code === 0) {
        if (stderr.length > 0) console.warn(`[CLI Warning] ${stderr}`);
        finish({ success: true, output: stdout.trim(), durationMs });
      } else {
        finish({ success: false, output: stdout, error: stderr || `Exit code ${code}`, durationMs });
      }
    });

    proc.on('error', (error) => {
      proc.kill('SIGKILL');
      finish({ success: false, output: '', error: `Process error: ${error.message}`, durationMs: Date.now() - start });
    });

    try {
      proc.stdin?.write(fullPrompt);
      proc.stdin?.end();
    } catch (error) {
      proc.kill('SIGKILL');
      finish({ success: false, output: '', error: `Stdin write failed: ${(error as Error).message}`, durationMs: Date.now() - start });
    }
  });
}

/**
 * Determines if an error is retryable based on configured patterns.
 * Uses substring matching against RETRY.RETRYABLE_PATTERNS.
 *
 * @param errorMsg - The error message to classify
 * @returns true if the error is transient and should be retried
 */
export function isRetryableError(errorMsg: string): boolean {
  const lowerMsg = errorMsg.toLowerCase();
  return RETRY.RETRYABLE_PATTERNS.some(pattern => lowerMsg.includes(pattern.toLowerCase()));
}

/**
 * Executes a CLI command with exponential backoff retry on transient failures.
 *
 * Latency tradeoff: Worst-case adds (1+2+4)=7 seconds delay before failure.
 * Configurable via RETRY constants in constants.ts.
 *
 * Error classification is centralized in isRetryableError() for consistency.
 */
export async function execWithRetry(
  pool: ExecutionPool,
  prompt: string,
  options: CliOptions,
  context: string = 'AGENT'
): Promise<CliResult> {
  for (let attempt = 1; attempt <= RETRY.MAX_ATTEMPTS; attempt++) {
    const result = await pool.exec(prompt, options);

    if (result.success) return result;

    const errorMsg = getCliError(result);

    if (!isRetryableError(errorMsg) || attempt === RETRY.MAX_ATTEMPTS) return result;

    const delayMs = Math.min(RETRY.BASE_DELAY_MS * 2 ** (attempt - 1), RETRY.MAX_DELAY_MS);
    console.warn(`[${context}] Retry ${attempt}/${RETRY.MAX_ATTEMPTS} after ${delayMs}ms (${errorMsg.substring(0, 50)})`);
    await new Promise(r => setTimeout(r, delayMs));
  }

  throw new Error('Unreachable');
}

export class CliPool {
  private running = 0;
  private queue: Array<() => void> = [];
  private shuttingDown = false;

  async exec(prompt: string, options: CliOptions): Promise<CliResult> {
    if (this.shuttingDown) {
      return { success: false, output: '', error: 'Pool is shutting down', durationMs: 0 };
    }

    if (this.running >= CONCURRENCY.MAX_AGENTS) {
      await new Promise<void>(resolve => this.queue.push(resolve));
    }

    this.running++;
    try {
      return await runClaude(prompt, options);
    } finally {
      this.running--;
      this.queue.shift()?.();
    }
  }

  async shutdown(): Promise<void> {
    this.shuttingDown = true;
    const queuedCount = this.queue.length;
    this.queue = [];

    console.log(`[Pool] Shutting down (${this.running} active, ${queuedCount} queued abandoned)`);

    const start = Date.now();
    while (this.running > 0 && Date.now() - start < 5000) {
      await new Promise(resolve => setTimeout(resolve, 100));
    }

    if (this.running > 0) {
      console.warn(`[Pool] ${this.running} processes did not exit cleanly`);
    }
  }
}
