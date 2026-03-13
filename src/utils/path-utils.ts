// Dracolich v12 Path Utilities

import { existsSync, realpathSync } from 'fs';
import { join, resolve, sep } from 'path';

export function generateProjectName(task: string): string {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-').substring(0, 19);
  const slug = task
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .substring(0, 30)
    .replace(/^-+|-+$/g, '')
    || 'untitled';
  return `${timestamp}-${slug}`;
}

export function ensurePathWithinBase(outputBase: string, projectName: string): string {
  if (projectName.includes('..') ||
      projectName.includes('/') ||
      projectName.includes('\\') ||
      projectName.includes('\0') ||
      /^[a-zA-Z]:/.test(projectName) ||  // Drive letter
      projectName.startsWith('\\\\')) {   // UNC path
    throw new Error(`Invalid project name: ${projectName}`);
  }

  const projectDir = join(outputBase, projectName);

  try {
    const safeBase = existsSync(outputBase) ? realpathSync(outputBase) : resolve(outputBase);
    const safeCwd = resolve(projectDir);

    if (!safeCwd.startsWith(safeBase + sep) && safeCwd !== safeBase) {
      throw new Error(`Path traversal detected: ${projectName} resolves outside ${outputBase}`);
    }

    return projectDir;
  } catch (error) {
    const code = (error as NodeJS.ErrnoException).code;
    throw new Error(`Path validation failed: ${code || (error as Error).message}`);
  }
}
