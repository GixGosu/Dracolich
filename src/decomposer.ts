// Dracolich v12 Meta-Decomposer

import { parseGovernanceJson } from './utils/json-parser.js';
import { CliPool } from './pool.js';
import { ClaudeModel, SwarmDesign, TaskDefinition, getCliError } from './types.js';
import { TIMEOUTS } from './constants.js';
import { META_DECOMPOSER_PROMPT } from './prompts.js';

export async function designSwarm(task: string, pool?: CliPool): Promise<SwarmDesign> {
  console.log('[META_DECOMPOSER] Designing swarm architecture...');

  const prompt = `Design a swarm of specialized agents for this task:

TASK: ${task}

Remember:
- Create agents with DISTINCT specialties
- Maximize parallelism where tasks are independent
- Include adversarial/critical perspectives
- Final agent should synthesize or critically review

Return ONLY valid JSON.`;

  const executor = pool ?? new CliPool();
  const result = await executor.exec(prompt, {
    model: ClaudeModel.Opus,
    systemPrompt: META_DECOMPOSER_PROMPT,
    maxTimeoutMs: TIMEOUTS.DECOMPOSER_MAX_MS,
    printOnly: true,
  });

  if (!pool) await executor.shutdown();

  if (!result.success) {
    console.error(`[META_DECOMPOSER] Swarm design failed: ${getCliError(result)}`);
    return createFallbackSwarm(task);
  }

  try {
    const design = parseGovernanceJson<SwarmDesign>(
      result.output, 'META_DECOMPOSER',
      createFallbackSwarm(task),
      ['reasoning', 'agents', 'groups']
    );
    validateSwarmDesign(design);
    console.log(`[META_DECOMPOSER] ✓ Designed ${design.agents.length} agents in ${design.groups.length} groups`);
    return design;
  } catch (error) {
    console.error(`[META_DECOMPOSER] Design failed: ${error}`);
    return createFallbackSwarm(task);
  }
}

export function validateSwarmDesign(design: SwarmDesign): void {
  const { agents, groups } = design;

  if (!Array.isArray(agents) || agents.length === 0) {
    throw new Error('Invalid agents: must be non-empty array');
  }
  if (!Array.isArray(groups) || groups.length === 0) {
    throw new Error('Invalid groups: must be non-empty array');
  }

  const agentNames = new Set<string>();
  for (const agent of agents) {
    if (!agent.name || !agent.systemPrompt) {
      throw new Error(`Invalid agent: missing name or systemPrompt`);
    }
    if (agentNames.has(agent.name)) {
      throw new Error(`Duplicate agent name: ${agent.name}`);
    }
    agentNames.add(agent.name);
  }

  const allTasks: TaskDefinition[] = [];
  const taskIds = new Set<string>();

  groups.forEach((group, idx) => {
    if (!Array.isArray(group) || group.length === 0) {
      throw new Error(`Group ${idx} must be non-empty array`);
    }

    for (const task of group) {
      if (!task.id || !task.agent || !task.description) {
        throw new Error(`Task in group ${idx} missing required field`);
      }
      if (!agentNames.has(task.agent)) {
        throw new Error(`Task ${task.id} references undefined agent: ${task.agent}`);
      }
      if (task.dependsOn.includes(task.id)) {
        throw new Error(`Self-dependency: task ${task.id} depends on itself`);
      }
      if (taskIds.has(task.id)) {
        throw new Error(`Duplicate task ID: ${task.id}`);
      }

      taskIds.add(task.id);
      allTasks.push(task);
    }
  });

  validateDependenciesAndCycles(allTasks);
}

function validateDependenciesAndCycles(allTasks: TaskDefinition[]): void {
  const taskMap = new Map(allTasks.map(t => [t.id, t]));
  const visited = new Set<string>();
  const stack = new Set<string>();

  function visit(taskId: string, path: string[]): void {
    if (stack.has(taskId)) {
      throw new Error(`Circular dependency: ${[...path, taskId].join(' → ')}`);
    }
    if (visited.has(taskId)) return;

    const task = taskMap.get(taskId);
    if (!task) {
      const parent = path.length > 0 ? path[path.length - 1] : 'unknown';
      throw new Error(`Task ${parent} depends on non-existent task ${taskId}`);
    }

    stack.add(taskId);
    for (const depId of task.dependsOn) {
      visit(depId, [...path, taskId]);
    }
    stack.delete(taskId);
    visited.add(taskId);
  }

  for (const task of allTasks) {
    visit(task.id, []);
  }
}

function createFallbackSwarm(task: string): SwarmDesign {
  console.warn('[META_DECOMPOSER] Using fallback swarm design');

  return {
    reasoning: 'Fallback to default research swarm due to meta-decomposer failure',
    agents: [
      {
        name: 'RESEARCHER',
        role: 'General research agent',
        systemPrompt: 'You are a thorough researcher. Find authoritative sources, cite evidence, be specific. Use web search to find real information. CREATE FILES for your findings in markdown format.',
      },
      {
        name: 'ANALYST',
        role: 'Synthesis and analysis',
        systemPrompt: 'You analyze and synthesize research findings. Identify patterns, resolve conflicts, produce coherent analysis. CREATE FILES documenting your analysis.',
      },
      {
        name: 'CRITIC',
        role: 'Adversarial review',
        systemPrompt: 'You are a rigorous critic. Challenge assumptions, find gaps, steelman opposing views, identify weaknesses. CREATE FILES with your critical review.',
      },
    ],
    groups: [
      [{ id: '1', description: `Research: ${task}`, agent: 'RESEARCHER', dependsOn: [] }],
      [{ id: '2', description: 'Synthesize findings', agent: 'ANALYST', dependsOn: ['1'] }],
      [{ id: '3', description: 'Critical review', agent: 'CRITIC', dependsOn: ['2'] }],
    ],
  };
}
