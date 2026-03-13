#!/usr/bin/env node
// Dracolich v12 CLI Entry Point

import { execSync } from 'child_process';
import { existsSync, readFileSync } from 'fs';
import { resolve } from 'path';
import { CONTEXT_LIMITS } from './constants.js';
import { orchestrateSwarm } from './orchestrator.js';
import { CliPool } from './pool.js';
import { format } from './utils/format-utils.js';

let pool: CliPool | null = null;
let shuttingDown = false;

const shutdown = async (signal: string) => {
  if (shuttingDown) return;
  shuttingDown = true;
  console.log(`\nReceived ${signal}, shutting down gracefully...`);
  if (pool) await pool.shutdown();
  process.exit(0);
};

process.on('SIGINT', () => shutdown('SIGINT'));
process.on('SIGTERM', () => shutdown('SIGTERM'));

async function main() {
  const args = process.argv.slice(2);
  const flags = {
    governanceDisabled: args.includes('--no-governance'),
    verboseReview: args.includes('--verbose'),
    abortOnFirstFailure: args.includes('--fail-fast'),
  };

  const fileIndex = args.findIndex(a => a === '--file' || a === '-f');
  let task: string;

  if (fileIndex !== -1 && args[fileIndex + 1]) {
    const filePath = resolve(args[fileIndex + 1]!);
    if (!existsSync(filePath)) {
      console.error(`✗ File not found: ${filePath}`);
      process.exit(1);
    }
    task = readFileSync(filePath, 'utf-8').trim();
    if (!task) {
      console.error(`✗ File is empty: ${filePath}`);
      process.exit(1);
    }
    console.log(`Reading task from: ${filePath}`);
    console.log(`Task length: ${task.length} characters\n`);
  } else {
    const filteredArgs = args.filter((a, i) => {
      if (a.startsWith('--') || a === '-f') return false;
      if (i > 0 && (args[i - 1] === '--file' || args[i - 1] === '-f')) return false;
      return true;
    });
    task = filteredArgs.join(' ');
  }

  if (!task) {
    console.log('Dracolich v12 — Self-Designing Build Swarm');
    console.log('');
    console.log('Agents CREATE FILES. Output is a project directory with deliverables.');
    console.log('');
    console.log('Usage:');
    console.log('  npx tsx src/v12/index.ts [options] "Your task here"');
    console.log('  npx tsx src/v12/index.ts [options] --file <path>');
    console.log('');
    console.log('Options:');
    console.log('  --file, -f <path>  Read task/prompt from a file');
    console.log('  --no-governance    Disable ARBITER and REAPER governance gates');
    console.log('  --fail-fast        Abort on first task failure');
    console.log('  --verbose          Show full REAPER review output');
    console.log('');
    console.log('Output: ./output/<timestamp>-<task>/');
    process.exit(1);
  }

  if (task.length > CONTEXT_LIMITS.TASK_CHARS) {
    console.error(`✗ Task is too long (${format.fileSize(task.length)}). Max: ${format.fileSize(CONTEXT_LIMITS.TASK_CHARS)}`);
    process.exit(1);
  }

  try {
    execSync('claude --version', { stdio: 'ignore' });
  } catch {
    console.error('✗ claude CLI not found in PATH');
    console.error('Install: npm install -g @anthropic-ai/claude-code');
    process.exit(1);
  }

  try {
    pool = new CliPool();

    const result = await orchestrateSwarm(task, {
      governance: flags.governanceDisabled ? { enabled: false } : undefined,
      pool,
      failFast: flags.abortOnFirstFailure,
    });

    console.log('\n' + '='.repeat(60));
    console.log('SUMMARY');
    console.log('='.repeat(60));
    console.log(`Status: ${result.success ? 'SUCCESS' : 'PARTIAL FAILURE'}`);
    console.log(`Agents: ${result.outputs.length} executed`);
    console.log(`Duration: ${format.duration(result.totalDurationMs)}`);
    console.log(`Project: ${result.projectDir}`);
    console.log(`Files created: ${result.filesCreated.length}`);

    if (result.filesCreated.length > 0) {
      for (const file of result.filesCreated.slice(0, 10)) console.log(`  - ${file}`);
      if (result.filesCreated.length > 10) console.log(`  ... and ${result.filesCreated.length - 10} more`);
    }

    console.log(`Governance: ARBITER ${result.governance.arbiter.decision}, REAPER ${result.governance.reaper.decision}`);

    if (result.governance.iterations > 0) {
      console.log(`Iterations: ${result.governance.iterations} revision(s)${result.governance.maxIterationsReached ? ' (max reached)' : ''}`);
    }

    console.log('\n' + '='.repeat(60));
    console.log('DELIVERABLES');
    console.log('='.repeat(60));
    console.log(`Project directory: ${result.projectDir}`);

    if (result.filesCreated.length > 0) {
      console.log('\nFiles:');
      for (const file of result.filesCreated) console.log(`  ${file}`);
      console.log(`\nOpen: ${result.projectDir}/index.html (if web project)`);
    } else {
      console.log('\nNo files were created. Agent outputs:');
      console.log(result.finalOutput.substring(0, 2000));
      if (result.finalOutput.length > 2000) console.log(`\n... (${format.fileSize(result.finalOutput.length)} total output)`);
    }

    if (result.governance.reaper.fullReview && flags.verboseReview) {
      console.log('\n' + '='.repeat(60));
      console.log('REAPER ADVERSARIAL REVIEW');
      console.log('='.repeat(60));
      console.log(result.governance.reaper.fullReview);
    }

    process.exit(result.success ? 0 : 1);
  } catch (error) {
    console.error(`✗ Fatal error: ${error}`);
    process.exit(1);
  }
}

main();
