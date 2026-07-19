import { readFile, stat } from 'fs/promises';
import { existsSync } from 'fs';
import { join } from 'path';
import { exec as execCb } from 'child_process';
import { promisify } from 'util';
import type { ProjectContext } from './types.js';

const exec = promisify(execCb);

const LIBRARY_PATH = join(process.cwd(), '.claude', 'library');

// --- Rules Cache (in-memory, mtime-based invalidation) ---
const rulesCache = new Map<string, { content: string; mtime: number }>();

async function loadRuleCached(file: string): Promise<string> {
  const fullPath = join(LIBRARY_PATH, file);
  try {
    const fileStat = await stat(fullPath);
    const cached = rulesCache.get(file);
    if (cached && cached.mtime >= fileStat.mtimeMs) return cached.content;
    const content = await readFile(fullPath, 'utf-8');
    rulesCache.set(file, { content, mtime: fileStat.mtimeMs });
    return content;
  } catch {
    return `[NOT FOUND: ${file}]`;
  }
}

export async function loadRules(files: string[]): Promise<{ text: string; lines: number }> {
  const results = await Promise.all(
    files.map(async (f) => {
      const content = await loadRuleCached(f);
      return { file: f, content };
    })
  );

  const contents: string[] = [];
  let totalLines = 0;
  for (const { file, content } of results) {
    if (!content.startsWith('[NOT FOUND')) {
      contents.push(`# --- ${file} ---\n${content}`);
      totalLines += content.split('\n').length;
    } else {
      contents.push(`# --- ${file} --- ${content}`);
    }
  }

  return { text: contents.join('\n\n'), lines: totalLines };
}

// --- Git Log Cache (TTL-based) ---
let gitLogCache = { text: '', timestamp: 0 };
const GIT_CACHE_TTL = 30_000; // 30 seconds

async function getGitLogCached(): Promise<string> {
  const now = Date.now();
  if (now - gitLogCache.timestamp < GIT_CACHE_TTL) return gitLogCache.text;
  try {
    const { stdout } = await exec('git log --oneline -5', { timeout: 3000 });
    gitLogCache = { text: stdout.trim(), timestamp: now };
    return gitLogCache.text;
  } catch {
    return '';
  }
}

// --- Project Context ---
export async function getProjectContext(keywords: string): Promise<ProjectContext> {
  const [lessons, research, gitLog, currentTask, toolRegistry, ecosystem] = await Promise.all([
    grepFile('tasks/lessons.md', keywords),
    readIfExists('tasks/.research-cache.md'),
    getGitLogCached(),
    readIfExists('tasks/current.md', 30),
    grepFile('_reference/tool-registry.md', keywords),
    grepFile('ecosystem.md', keywords)
  ]);

  return { lessons, research, gitLog, currentTask, toolRegistry, ecosystem };
}

async function grepFile(filePath: string, keywords: string): Promise<string> {
  if (!existsSync(filePath) || !keywords) return '';

  try {
    const content = await readFile(filePath, 'utf-8');
    const keywordList = keywords.toLowerCase().split(/\s+/).filter(k => k.length > 1);
    if (keywordList.length === 0) return '';

    const lines = content.split('\n');
    const matches: string[] = [];
    const seen = new Set<number>();

    for (let i = 0; i < lines.length; i++) {
      if (seen.has(i)) continue;
      const lineLower = lines[i].toLowerCase();
      if (keywordList.some(k => lineLower.includes(k))) {
        const start = Math.max(0, i - 2);
        const end = Math.min(lines.length, i + 3);
        for (let j = start; j < end; j++) seen.add(j);
        matches.push(lines.slice(start, end).join('\n'));
        matches.push('---');
      }
    }

    return matches.slice(0, 10).join('\n');
  } catch {
    return '';
  }
}

async function readIfExists(filePath: string, maxLines?: number): Promise<string> {
  if (!existsSync(filePath)) return '';
  try {
    const content = await readFile(filePath, 'utf-8');
    if (maxLines) return content.split('\n').slice(0, maxLines).join('\n');
    return content;
  } catch {
    return '';
  }
}
