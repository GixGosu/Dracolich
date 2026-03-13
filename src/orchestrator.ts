// Dracolich v12 Main Orchestrator

import { mkdirSync, writeFileSync } from 'fs';
import {
  AgentResult, ArbiterVerdict, ClaudeModel, ExecutionPool, getCliError,
  ReaperVerdict, RunOptions, RunResult, SwarmDesign, TaskDefinition,
} from './types.js';
import { CliPool, execWithRetry } from './pool.js';
import { ensurePathWithinBase, generateProjectName } from './utils/path-utils.js';
import { listFilesRecursive } from './utils/file-operations.js';
import { format } from './utils/format-utils.js';
import { GOVERNANCE_DEFAULTS } from './constants.js';
import { runGovernanceLoop, validateSwarmDesign as arbiterValidate } from './governance.js';
import { designSwarm } from './decomposer.js';

// NEW: Extracted prompt building logic
function buildAgentPrompt(
  mainTask: string,
  agentTask: TaskDefinition,
  projectDir: string,
  priorResults: AgentResult[]
): string {
  let prompt = `# Main Task
${mainTask}

# Working Directory
You are working in: ${projectDir}
Create all files in this directory. Use relative paths.

# Your Assignment
${agentTask.description}`;

  if (agentTask.dependsOn.length > 0) {
    prompt += '\n\n# Input From Prior Agents\n';
    for (const depId of agentTask.dependsOn) {
      const prior = priorResults.find(r => r.id === depId);
      if (!prior) throw new Error(`Missing dependency: ${agentTask.id} depends on ${depId}`);
      if (!prior.success) throw new Error(`Blocked by failed dependency: ${depId}`);
      if (!prior.output?.trim()) {
        console.warn(`[${agentTask.id}] Dependency ${depId} succeeded but produced no output`);
      }
      prompt += `\n## ${prior.agent} (Task ${depId})\n${prior.output}`;
    }
  }

  return prompt;
}

async function designAndSaveSwarm(task: string, pool: ExecutionPool, projectDir: string): Promise<SwarmDesign> {
  const design = await designSwarm(task, pool as CliPool);

  const designJson = { task, timestamp: new Date().toISOString(), ...design };

  try {
    writeFileSync(`${projectDir}/swarm-design.json`, JSON.stringify(designJson, null, 2), 'utf-8');
    writeFileSync(`${projectDir}/SWARM.md`, generateSwarmMarkdown(task, design), 'utf-8');
  } catch (error) {
    throw new Error(`Failed to write design files: ${(error as Error).message}`);
  }

  console.log('\nSwarm Design:');
  console.log(`  Reasoning: ${design.reasoning}`);
  console.log('  Agents:');
  for (const agent of design.agents) console.log(`    - ${agent.name}: ${agent.role}`);
  console.log(`  Saved to: ${projectDir}/SWARM.md\n`);

  return design;
}

function generateSwarmMarkdown(task: string, design: SwarmDesign): string {
  const lines = [
    '# Swarm Design', '', `**Task:** ${task}`, '',
    `**Generated:** ${new Date().toISOString()}`, '',
    '## Reasoning', '', design.reasoning, '', '## Agents', '',
  ];

  for (const agent of design.agents) {
    lines.push(`### ${agent.name}`, '', `**Role:** ${agent.role}`, '', '**System Prompt:**', '```', agent.systemPrompt, '```', '');
  }

  lines.push('## Execution Groups', '');
  for (let i = 0; i < design.groups.length; i++) {
    const group = design.groups[i]!;
    const parallel = group.length > 1 ? '(parallel)' : '(sequential)';
    lines.push(`### Group ${i + 1} ${parallel}`, '');
    for (const t of group) {
      const deps = t.dependsOn.length > 0 ? ` ← depends on: ${t.dependsOn.join(', ')}` : '';
      lines.push(`- **${t.agent}** (${t.id}): ${t.description}${deps}`);
    }
    lines.push('');
  }

  return lines.join('\n');
}

async function runAgentTask(
  agentTask: TaskDefinition,
  prompt: string,
  systemPrompt: string,
  model: ClaudeModel,
  projectDir: string,
  pool: ExecutionPool
): Promise<AgentResult> {
  const baseResult = { id: agentTask.id, agent: agentTask.agent };

  // NEW: Use execWithRetry instead of inline retry logic
  const result = await execWithRetry(
    pool,
    prompt,
    { model, systemPrompt, workingDir: projectDir },
    agentTask.agent
  );

  if (result.success) {
    return { ...baseResult, success: true, output: result.output, durationMs: result.durationMs };
  } else {
    const errorMsg = getCliError(result);
    return { ...baseResult, success: false, output: errorMsg, durationMs: result.durationMs };
  }
}

async function executeSwarm(
  design: SwarmDesign,
  mainTask: string,
  pool: ExecutionPool,
  projectDir: string,
  failFast: boolean = false
): Promise<{ success: boolean; outputs: AgentResult[]; finalOutput: string; totalDurationMs: number }> {
  const startTime = Date.now();
  const allResults: AgentResult[] = [];

  const totalAgents = design.groups.flat().length;
  console.log(`\n[Orchestrator] Executing swarm: ${design.agents.length} agent types, ${totalAgents} tasks, ${design.groups.length} groups`);

  for (let i = 0; i < design.groups.length; i++) {
    const group = design.groups[i]!;
    const isLastGroup = i === design.groups.length - 1;
    console.log(`\n--- Group ${i + 1}/${design.groups.length} (${group.length > 1 ? `${group.length} parallel` : 'sequential'}) ---`);

    for (const agentTask of group) {
      for (const depId of agentTask.dependsOn) {
        const dep = allResults.find(r => r.id === depId);
        if (!dep) throw new Error(`Missing dependency: ${agentTask.id} requires ${depId}`);
        if (!dep.success) throw new Error(`Blocked by failed dependency: ${depId}`);
      }
    }

    const promises = group.map(async (agentTask) => {
      const systemPrompt = design.agents.find(a => a.name === agentTask.agent)?.systemPrompt || '';

      // NEW: Use extracted prompt building function
      const taskPrompt = buildAgentPrompt(mainTask, agentTask, projectDir, allResults);

      console.log(`  [${agentTask.agent}] Starting...`);

      const model = isLastGroup ? ClaudeModel.Opus : ClaudeModel.Sonnet;
      const agentResult = await runAgentTask(agentTask, taskPrompt, systemPrompt, model, projectDir, pool);
      allResults.push(agentResult);

      console.log(`  [${agentTask.agent}] ${agentResult.success ? '✓' : '✗'} ${format.duration(agentResult.durationMs)}, ${format.fileSize(agentResult.output.length)}`);
      return agentResult;
    });

    const groupResults = await Promise.allSettled(promises);

    // Check for agent crashes (rejections, not just failures)
    for (const result of groupResults) {
      if (result.status === 'rejected') {
        console.error(`[Orchestrator] Agent crashed: ${result.reason}`);
        if (failFast) throw new Error(`Agent crashed in group ${i + 1}`);
      }
    }

    if (failFast && groupResults.some(r => r.status === 'fulfilled' && !r.value.success)) {
      throw new Error(`Task failed in group ${i + 1}, aborting with --fail-fast`);
    }
  }

  const finalParts = allResults
    .filter(r => r.success)
    .map(format.agentOutput);

  return {
    success: allResults.every(r => r.success),
    outputs: allResults,
    finalOutput: finalParts.join('\n'),
    totalDurationMs: Date.now() - startTime,
  };
}

export async function orchestrateSwarm(task: string, options: RunOptions = {}): Promise<RunResult> {
  console.log('='.repeat(60));
  console.log('DRACOLICH v12 — Self-Designing Swarm');
  console.log('='.repeat(60));
  console.log(`\nTask: ${task}\n`);

  const outputBase = options.outputDir ?? './output';
  const projectName = generateProjectName(task);
  const projectDir = ensurePathWithinBase(outputBase, projectName);

  try {
    mkdirSync(outputBase, { recursive: true });
    mkdirSync(projectDir, { recursive: true });
  } catch (error) {
    throw new Error(`Failed to create directory ${projectDir}: ${(error as Error).message}`);
  }

  const pool = options.pool ?? new CliPool();
  const govConfig = { ...GOVERNANCE_DEFAULTS, ...options.governance };

  console.log(`[Orchestrator] Project directory: ${projectDir}\n`);

  const design = await designAndSaveSwarm(task, pool, projectDir);
  const arbiterDecision = await arbiterValidate(design, task, govConfig, pool);

  if (arbiterDecision.decision === ArbiterVerdict.Reject) {
    console.error('[ARBITER] ✗ Design REJECTED - aborting execution');
    return {
      success: false,
      design,
      outputs: [],
      finalOutput: `Execution aborted: ${arbiterDecision.summary}`,
      totalDurationMs: 0,
      projectDir,
      filesCreated: [],
      governance: {
        arbiter: arbiterDecision,
        reaper: {
          decision: ReaperVerdict.SendBack,
          criticalFindings: 0, majorFindings: 0, minorFindings: 0,
          steelmanSurvives: false, adjustedConfidence: 'N/A',
          summary: 'Not executed due to ARBITER rejection', fullReview: '',
        },
        iterations: 0,
        maxIterationsReached: false,
      },
    };
  }

  const execResult = await executeSwarm(design, task, pool, projectDir, options.failFast);
  const filesCreated = listFilesRecursive(projectDir);
  const isUserDeliverable = (f: string) => !f.endsWith('.json') && f !== 'SWARM.md';
  const hasBuildArtifacts = filesCreated.some(isUserDeliverable);

  // NEW: Use extracted governance loop
  const govResult = await runGovernanceLoop(task, execResult.finalOutput, govConfig, {
    projectDir,
    hasBuildArtifacts,
    pool
  });

  return {
    success: execResult.success,
    design,
    outputs: execResult.outputs,
    finalOutput: govResult.finalOutput,
    totalDurationMs: execResult.totalDurationMs,
    projectDir,
    filesCreated,
    governance: {
      arbiter: arbiterDecision,
      reaper: govResult.reaperDecision,
      iterations: govResult.iterations,
      maxIterationsReached: govResult.maxIterationsReached,
    },
  };
}
