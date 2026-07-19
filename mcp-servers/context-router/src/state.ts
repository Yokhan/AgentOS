import { writeFile, readFile } from 'fs/promises';
import { existsSync, mkdirSync } from 'fs';
import type { ServerState } from './types.js';

const STATE_FILE = 'tasks/.active-rules';

let state: ServerState = {
  currentModes: [],
  activeRules: [],
  lastRouteTime: '',
  taskDescription: ''
};

export function getState(): ServerState {
  return { ...state };
}

export function updateState(modes: string[], rules: string[], task: string): void {
  state = {
    currentModes: modes,
    activeRules: rules,
    lastRouteTime: new Date().toISOString(),
    taskDescription: task
  };
  persistState().catch((err) => {
    console.error(`state: persist failed — ${err instanceof Error ? err.message : String(err)}`);
  });
}

async function persistState(): Promise<void> {
  if (!existsSync('tasks')) {
    mkdirSync('tasks', { recursive: true });
  }

  const lines = [
    'STATE_VERSION=2',
    `TASK=${state.taskDescription}`,
    `MODES=${state.currentModes.join(' ')}`,
    `COUNT=${state.activeRules.length} files`,
    `ROUTED_AT=${state.lastRouteTime}`,
    '---',
    ...state.activeRules
  ];

  await writeFile(STATE_FILE, lines.join('\n'), 'utf-8');
}

export async function restoreState(): Promise<ServerState | null> {
  if (!existsSync(STATE_FILE)) return null;

  try {
    const content = await readFile(STATE_FILE, 'utf-8');
    const lines = content.split('\n');
    const task = lines.find(l => l.startsWith('TASK='))?.slice(5) || '';
    const modes = (lines.find(l => l.startsWith('MODES='))?.slice(6) || '').split(' ').filter(Boolean);
    const separatorIdx = lines.indexOf('---');
    const rules = separatorIdx >= 0 ? lines.slice(separatorIdx + 1).filter(Boolean) : [];
    const routedAt = lines.find(l => l.startsWith('ROUTED_AT='))?.slice(10) || '';

    state = { currentModes: modes, activeRules: rules, lastRouteTime: routedAt, taskDescription: task };
    return state;
  } catch (err) {
    console.error(`state: restore failed — ${err instanceof Error ? err.message : String(err)}`);
    return null;
  }
}
