// Dracolich v12 Types

export enum ClaudeModel {
  Haiku = 'haiku',
  Sonnet = 'sonnet',
  Opus = 'opus',
}

export enum ArbiterVerdict {
  Approve = 'APPROVE',
  Revise = 'REVISE',
  Reject = 'REJECT',
}

export enum ReaperVerdict {
  Approve = 'APPROVE',
  SendBack = 'SEND_BACK',
}

export interface AgentDefinition {
  name: string;
  role: string;
  systemPrompt: string;
}

export interface TaskDefinition {
  id: string;
  description: string;
  agent: string;
  dependsOn: string[];
}

export interface SwarmDesign {
  agents: AgentDefinition[];
  groups: TaskDefinition[][];
  reasoning: string;
}

export interface AgentResult {
  id: string;
  agent: string;
  output: string;
  durationMs: number;
  success: boolean;
}

export type CliResult =
  | {
      success: true;
      output: string;
      durationMs: number;
    }
  | {
      success: false;
      output: string;
      error: string;
      durationMs: number;
    };

export function getCliError(result: CliResult): string {
  if (result.success === false) {
    return result.error;
  }
  return '';
}

export interface ArbiterDecision {
  decision: ArbiterVerdict;
  findings: Array<{
    severity: 'critical' | 'major' | 'minor';
    issue: string;
    fix: string;
  }>;
  summary: string;
}

export interface ReaperReview {
  decision: ReaperVerdict;
  criticalFindings: number;
  majorFindings: number;
  minorFindings: number;
  steelmanSurvives: boolean;
  adjustedConfidence: string;
  summary: string;
  fullReview: string;
}

export interface GovernanceConfig {
  enabled: boolean;
  arbiterEnabled: boolean;
  reaperEnabled: boolean;
  model: ClaudeModel;
  maxIterations: number;
}

export interface GovernanceResult {
  finalOutput: string;
  reaperDecision: ReaperReview;
  iterations: number;
  maxIterationsReached: boolean;
}

export interface CliOptions {
  model: ClaudeModel;
  systemPrompt?: string;
  workingDir?: string;
  printOnly?: boolean;
  maxTimeoutMs?: number;
  enableActivityTimeout?: boolean;
}

export interface RunResult {
  success: boolean;
  design: SwarmDesign;
  outputs: AgentResult[];
  finalOutput: string;
  totalDurationMs: number;
  projectDir: string;
  filesCreated: string[];
  governance: {
    arbiter: ArbiterDecision;
    reaper: ReaperReview;
    iterations: number;
    maxIterationsReached: boolean;
  };
}

export interface ExecutionPool {
  exec(task: string, options: CliOptions): Promise<CliResult>;
  shutdown(): Promise<void>;
}

export interface RunOptions {
  governance?: Partial<GovernanceConfig>;
  outputDir?: string;
  pool?: ExecutionPool;
  failFast?: boolean;
}
