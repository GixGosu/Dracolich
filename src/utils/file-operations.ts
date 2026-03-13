// Dracolich v12 File Operations

import { existsSync, readdirSync, readFileSync, statSync } from 'fs';
import { join, relative } from 'path';
import { FILE_LIMITS } from '../constants.js';

export function listFilesRecursive(
  currentDir: string,
  rootDir: string = currentDir,
  currentDepth: number = 0
): string[] {
  const files: string[] = [];
  if (!existsSync(currentDir)) return files;
  if (currentDepth > FILE_LIMITS.MAX_DIR_DEPTH) {
    console.warn(`[FileOps] Max directory depth reached: ${currentDir}`);
    return files;
  }

  const entries = readdirSync(currentDir);
  for (const entry of entries) {
    const fullPath = join(currentDir, entry);
    const relativePath = relative(rootDir, fullPath);

    try {
      const stat = statSync(fullPath);
      if (stat.isDirectory()) {
        files.push(...listFilesRecursive(fullPath, rootDir, currentDepth + 1));
      } else {
        files.push(relativePath);
      }
    } catch (error) {
      console.warn(`[FileOps] Cannot stat ${entry}: ${(error as Error).message}`);
    }
  }
  return files;
}

export function readProjectFiles(projectDir: string): string {
  const lines: string[] = [];

  try {
    const entries = readdirSync(projectDir);
    for (const entry of entries) {
      if (entry === 'swarm-design.json' || entry === 'SWARM.md') continue;

      const filePath = join(projectDir, entry);
      try {
        const stat = statSync(filePath);

        if (stat.isDirectory()) {
          lines.push(`### ${entry}/ (directory)`);
          continue;
        }

        if (!stat.isFile() || stat.size >= FILE_LIMITS.MAX_FILE_SIZE_FOR_REVIEW) continue;

        const content = readFileSync(filePath, 'utf-8');
        const preview = content.substring(0, FILE_LIMITS.MAX_FILE_PREVIEW_CHARS);
        const truncated = content.length > FILE_LIMITS.MAX_FILE_PREVIEW_CHARS
          ? `\n... (truncated, ${content.length} bytes total)`
          : '';

        lines.push(`### ${entry}\n\`\`\`\n${preview}${truncated}\n\`\`\`\n`);
      } catch (error) {
        const reason = (error as Error).message;
        console.warn(`[FileOps] Cannot read ${entry}: ${reason}`);
        lines.push(`### ${entry} (unreadable: ${reason})`);
      }
    }
  } catch {
    lines.push('(Could not read project directory)');
  }

  return lines.length > 0 ? lines.join('\n') : '(No files created)';
}
