// Dracolich v12 Agent Prompt Loader

import { existsSync, readFileSync } from 'fs';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const AGENTS_DIR = join(__dirname, '../agents');

function loadAgentPrompt(agentName: string): string {
  const filePath = join(AGENTS_DIR, `${agentName}.md`);
  if (!existsSync(filePath)) {
    throw new Error(`Missing required agent prompt: ${filePath}`);
  }

  try {
    return readFileSync(filePath, 'utf-8');
  } catch (error) {
    throw new Error(`Failed to load ${agentName} prompt: ${(error as Error).message}`);
  }
}

export const ARBITER_PROMPT = loadAgentPrompt('ARBITER');
export const REAPER_PROMPT = loadAgentPrompt('REAPER');
export const FIXER_PROMPT = loadAgentPrompt('FIXER');
export const REVISER_PROMPT = loadAgentPrompt('REVISER');
export const META_DECOMPOSER_PROMPT = loadAgentPrompt('META_DECOMPOSER');
