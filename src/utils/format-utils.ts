// Dracolich v12 Formatting Utilities

import { AgentResult } from '../types.js';

export const format = {
  fileSize(bytes: number): string {
    return `${(bytes / 1024).toFixed(1)}KB`;
  },

  duration(ms: number): string {
    return `${(ms / 1000).toFixed(1)}s`;
  },

  agentOutput(result: AgentResult): string {
    const divider = '='.repeat(60);
    return `\n${divider}\n${result.agent}\n${divider}\n${result.output}`;
  },
} as const;
