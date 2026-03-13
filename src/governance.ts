// Dracolich v12 Governance

import { readProjectFiles } from './utils/file-operations.js';
import { parseGovernanceJson } from './utils/json-parser.js';
import { format } from './utils/format-utils.js';
import { CONTEXT_LIMITS, TIMEOUTS } from './constants.js';
import { ARBITER_PROMPT, FIXER_PROMPT, REAPER_PROMPT, REVISER_PROMPT } from './prompts.js';
import { execWithRetry } from './pool.js';
import {
  ArbiterDecision,
  ArbiterVerdict,
  CliResult,
  ClaudeModel,
  ExecutionPool,
  GovernanceConfig,
  GovernanceResult,
  ReaperReview,
  ReaperVerdict,
  SwarmDesign,
  getCliError,
} from './types.js';

// NEW: Extracted fallback helpers for consistency
function createArbiterFallback(errorMsg: string): ArbiterDecision {
  return {
    decision: ArbiterVerdict.Revise,
    findings: [],
    summary: `Review failed: ${errorMsg}`
  };
}

function createReaperFallback(errorMsg: string): ReaperReview {
  return {
    decision: ReaperVerdict.SendBack,
    criticalFindings: 1,
    majorFindings: 0,
    minorFindings: 0,
    steelmanSurvives: false,
    adjustedConfidence: 'unknown',
    summary: `Review failed: ${errorMsg}`,
    fullReview: `EXECUTION ERROR: ${errorMsg}`
  };
}

function createDisabledReaperResult(): ReaperReview {
  return {
    decision: ReaperVerdict.Approve,
    criticalFindings: 0,
    majorFindings: 0,
    minorFindings: 0,
    steelmanSurvives: true,
    adjustedConfidence: 'unknown',
    summary: 'Governance disabled',
    fullReview: '',
  };
}

export async function validateSwarmDesign(
  design: SwarmDesign,
  task: string,
  config: GovernanceConfig,
  pool: ExecutionPool
): Promise<ArbiterDecision> {
  if (!config.enabled || !config.arbiterEnabled) {
    return { decision: ArbiterVerdict.Approve, findings: [], summary: 'Governance disabled' };
  }

  console.log('[ARBITER] Reviewing swarm design...');

  const lines: string[] = [`Reasoning: ${design.reasoning}`, '', `Agents (${design.agents.length}):`];
  for (const agent of design.agents) {
    lines.push(`  - ${agent.name}: ${agent.role}`);
  }
  lines.push('', `Execution Groups (${design.groups.length}):`);
  for (let i = 0; i < design.groups.length; i++) {
    const group = design.groups[i]!;
    const parallel = group.length > 1 ? 'PARALLEL' : 'SEQUENTIAL';
    lines.push(`  Group ${i + 1} [${parallel}]:`);
    for (const t of group) {
      const deps = t.dependsOn.length > 0 ? ` (depends: ${t.dependsOn.join(', ')})` : '';
      lines.push(`    - ${t.id} [${t.agent}]: ${t.description}${deps}`);
    }
  }

  const prompt = ARBITER_PROMPT.replace('{DESIGN}', lines.join('\n')).replace('{TASK}', task);
  const model = config.model === ClaudeModel.Haiku ? ClaudeModel.Opus : (config.model ?? ClaudeModel.Opus);

  // NEW: Use execWithRetry for robustness
  const result = await execWithRetry(pool, prompt, {
    model,
    maxTimeoutMs: TIMEOUTS.GOV_MAX_MS,
    printOnly: true,
  }, 'ARBITER');

  if (!result.success) {
    const errorMsg = getCliError(result);
    console.warn('[ARBITER] Review failed - using safe fallback');
    if (errorMsg) console.error(`[ARBITER] ${errorMsg}`);
    return createArbiterFallback(errorMsg);
  }

  const decision = parseGovernanceJson<ArbiterDecision>(
    result.output, 'ARBITER',
    createArbiterFallback('Parse failed - manual review needed'),
    ['decision', 'findings', 'summary']
  );

  const icon = decision.decision === ArbiterVerdict.Approve ? '✓' :
               decision.decision === ArbiterVerdict.Revise ? '⚠' : '✗';
  console.log(`[ARBITER] ${icon} ${decision.decision}: ${decision.summary}`);

  if (decision.findings.length > 0) {
    for (const f of decision.findings) console.log(`  [${f.severity}] ${f.issue}`);
  }

  return decision;
}

export async function reviewDeliverables(
  task: string,
  finalOutput: string,
  config: GovernanceConfig,
  pool: ExecutionPool,
  projectDir?: string
): Promise<ReaperReview> {
  if (!config.enabled || !config.reaperEnabled) {
    return createDisabledReaperResult();
  }

  console.log('[REAPER] Adversarial review of deliverables...');

  const filesContent = projectDir ? readProjectFiles(projectDir) : '(No project directory)';

  const prompt = REAPER_PROMPT
    .replace('{TASK}', task)
    .replace('{FILES}', filesContent)
    .replace('{OUTPUT}', finalOutput.substring(0, CONTEXT_LIMITS.REAPER_OUTPUT_CHARS));

  // NEW: Use execWithRetry for robustness
  const result = await execWithRetry(pool, prompt, {
    model: ClaudeModel.Opus,
    maxTimeoutMs: TIMEOUTS.GOV_MAX_MS,
    printOnly: true,
  }, 'REAPER');

  if (!result.success) {
    const errorMsg = getCliError(result);
    console.warn('[REAPER] Review failed - using safe fallback');
    if (errorMsg) console.error(`[REAPER] ${errorMsg}`);
    return createReaperFallback(errorMsg);
  }

  const parsed = parseGovernanceJson<Omit<ReaperReview, 'fullReview'>>(
    result.output, 'REAPER',
    {
      decision: ReaperVerdict.SendBack,
      criticalFindings: 1,
      majorFindings: 0,
      minorFindings: 0,
      steelmanSurvives: false,
      adjustedConfidence: 'unknown',
      summary: 'Parse failed - see full review',
    },
    ['decision', 'criticalFindings', 'majorFindings', 'minorFindings']
  );

  const review = { ...parsed, fullReview: result.output };

  const icon = review.decision === ReaperVerdict.Approve ? '✓' : '⚠';
  console.log(`[REAPER] ${icon} ${review.decision}`);
  console.log(`  Critical: ${review.criticalFindings}, Major: ${review.majorFindings}, Minor: ${review.minorFindings}`);
  console.log(`  Steelman survives: ${review.steelmanSurvives}`);

  return review;
}

async function iterativelyRefine(
  mode: 'REVISER' | 'FIXER',
  task: string,
  critique: string,
  pool: ExecutionPool,
  options: { currentOutput?: string; projectDir?: string; iteration: number }
): Promise<CliResult> {
  const MAX_CRITIQUE = 15_000;

  console.log(`[${mode}] Iteration ${options.iteration}: Starting...`);

  const prompt = mode === 'REVISER'
    ? REVISER_PROMPT
        .replace('{TASK}', task)
        .replace('{OUTPUT}', (options.currentOutput || '').substring(0, CONTEXT_LIMITS.REVISER_OUTPUT_CHARS))
        .replace('{CRITIQUE}', critique.substring(0, MAX_CRITIQUE))
    : FIXER_PROMPT
        .replace('{TASK}', task)
        .replace('{CRITIQUE}', critique.substring(0, CONTEXT_LIMITS.FIXER_OUTPUT_CHARS));

  // NEW: Use execWithRetry for robustness
  const result = await execWithRetry(pool, prompt, {
    model: ClaudeModel.Opus,
    workingDir: mode === 'FIXER' ? options.projectDir : undefined,
    maxTimeoutMs: TIMEOUTS.GOV_MAX_MS,
    printOnly: mode === 'REVISER',
  }, mode);

  if (!result.success) {
    console.error(`[${mode}] Failed: ${getCliError(result)}`);
  } else {
    console.log(`[${mode}] ✓ Complete: ${format.fileSize(result.output.length)}, ${format.duration(result.durationMs)}`);
  }

  return result;
}

// NEW: Extracted governance loop from orchestrator for separation of concerns
export async function runGovernanceLoop(
  task: string,
  initialOutput: string,
  config: GovernanceConfig,
  context: { projectDir: string; hasBuildArtifacts: boolean; pool: ExecutionPool }
): Promise<GovernanceResult> {
  if (!config.enabled || !config.reaperEnabled) {
    return {
      finalOutput: initialOutput,
      reaperDecision: createDisabledReaperResult(),
      iterations: 0,
      maxIterationsReached: false,
    };
  }

  let currentOutput = initialOutput;

  for (let iteration = 0; iteration < config.maxIterations; iteration++) {
    const review = await reviewDeliverables(task, currentOutput, config, context.pool, context.projectDir);

    if (review.decision === ReaperVerdict.Approve) {
      return {
        finalOutput: currentOutput,
        reaperDecision: review,
        iterations: iteration,
        maxIterationsReached: false
      };
    }

    console.log(`\n🔄 REAPER iteration ${iteration + 1}/${config.maxIterations}`);

    const mode = context.hasBuildArtifacts ? 'FIXER' : 'REVISER';
    const improved = await iterativelyRefine(
      mode,
      task,
      review.fullReview,
      context.pool,
      { currentOutput, projectDir: context.projectDir, iteration: iteration + 1 }
    );

    if (!improved.success) {
      console.warn('[GOVERNANCE] Improvement failed, accepting current output');
      return {
        finalOutput: currentOutput,
        reaperDecision: review,
        iterations: iteration + 1,
        maxIterationsReached: false
      };
    }

    currentOutput = improved.output;
  }

  const finalReview = await reviewDeliverables(task, currentOutput, config, context.pool, context.projectDir);
  console.warn(`[GOVERNANCE] Max iterations (${config.maxIterations}) reached`);

  return {
    finalOutput: currentOutput,
    reaperDecision: finalReview,
    iterations: config.maxIterations,
    maxIterationsReached: true
  };
}
