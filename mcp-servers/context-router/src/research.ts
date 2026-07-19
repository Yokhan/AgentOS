import { readFile, readdir, stat } from 'fs/promises';
import { existsSync, readFileSync, readdirSync, statSync } from 'fs';
import { join, basename } from 'path';
import { exec as execCb } from 'child_process';
import { promisify } from 'util';

const exec = promisify(execCb);

/**
 * Safe shell escape — prevents command injection.
 * For use ONLY with exec() when native Node.js alternative isn't available.
 */
function shellEscape(s: string): string {
  return "'" + s.replace(/'/g, "'\\''") + "'";
}

/** Platform-safe grep: uses Node.js readFile + includes() instead of shell grep */
async function nativeGrep(filePath: string, keyword: string, maxLines = 5): Promise<string> {
  if (!existsSync(filePath)) return '';
  try {
    const content = await readFile(filePath, 'utf-8');
    const kw = keyword.toLowerCase();
    const matches = content.split('\n')
      .filter(line => line.toLowerCase().includes(kw))
      .slice(0, maxLines);
    return matches.join('\n');
  } catch {
    return '';
  }
}

/** Platform-safe find importers: uses Node.js fs instead of shell grep -rl */
async function nativeFindImporters(keyword: string, dirs = ['src', 'lib', 'app']): Promise<string[]> {
  const results: string[] = [];
  const kw = keyword.toLowerCase();
  const exts = ['.ts', '.tsx', '.js', '.jsx', '.py', '.go', '.rs', '.vue', '.svelte'];

  for (const dir of dirs) {
    if (!existsSync(dir)) continue;
    try {
      const walk = (d: string) => {
        if (results.length >= 10) return;
        for (const entry of readdirSync(d)) {
          if (entry === 'node_modules' || entry === '.git' || entry === 'dist') continue;
          const full = join(d, entry);
          try {
            const s = statSync(full);
            if (s.isDirectory()) { walk(full); continue; }
            if (!exts.some(e => full.endsWith(e))) continue;
            const content = readFileSync(full, 'utf-8');
            if (content.toLowerCase().includes(kw)) {
              results.push(full);
            }
          } catch { /* skip inaccessible files */ }
        }
      };
      walk(dir);
    } catch { /* dir doesn't exist or inaccessible */ }
  }
  return results;
}

/** Platform-safe find by keyword: uses Node.js fs instead of shell find */
async function nativeFindByKeyword(word: string, dirs = ['src', 'lib', 'app']): Promise<string[]> {
  const results: string[] = [];
  const kw = word.toLowerCase();

  for (const dir of dirs) {
    if (!existsSync(dir)) continue;
    try {
      const walk = (d: string) => {
        if (results.length >= 5) return;
        for (const entry of readdirSync(d)) {
          if (entry === 'node_modules' || entry === '.git') continue;
          const full = join(d, entry);
          try {
            if (statSync(full).isDirectory()) { walk(full); continue; }
            if (entry.toLowerCase().includes(kw)) results.push(full);
          } catch { /* skip */ }
        }
      };
      walk(dir);
    } catch { /* skip */ }
  }
  return results;
}

export async function runResearch(targetPath: string): Promise<string> {
  const sections: string[] = [];
  const keywords = basename(targetPath).replace(/\.[^.]*$/, '');

  sections.push(`=== RESEARCH: ${targetPath} ===`);
  sections.push('');

  // 1. Target files
  sections.push('FILES:');
  if (existsSync(targetPath)) {
    try {
      if (statSync(targetPath).isDirectory()) {
        const files = findSourceFiles(targetPath, 20);
        for (const f of files) {
          sections.push(`  ${f} (${countLines(f)} lines)`);
        }
      } else {
        sections.push(`  ${targetPath} (${countLines(targetPath)} lines)`);
      }
    } catch (err) {
      sections.push(`  ⚠ Cannot read ${targetPath}: ${err instanceof Error ? err.message : String(err)}`);
    }
  } else {
    sections.push(`  Target not found: ${targetPath}`);
  }

  // 2. Importers (Node.js native — no shell)
  sections.push('');
  sections.push('IMPORTERS:');
  try {
    const importers = await nativeFindImporters(keywords);
    if (importers.length) {
      for (const f of importers) sections.push(`  ${f}`);
    } else {
      sections.push('  (none found)');
    }
  } catch (err) {
    sections.push(`  ⚠ Search failed: ${err instanceof Error ? err.message : String(err)}`);
  }

  // 3. Git log (safe — uses shellEscape)
  sections.push('');
  sections.push('RECENT GIT:');
  try {
    const { stdout } = await exec(`git log --oneline -5 -- ${shellEscape(targetPath)}`, { timeout: 3000 });
    sections.push(stdout.trim() || '  (no history)');
  } catch (err) {
    sections.push(`  ⚠ git: ${err instanceof Error ? err.message : 'not a git repo'}`);
  }

  // 4. Lessons (Node.js native — no shell)
  sections.push('');
  sections.push('LESSONS:');
  const lessons = await nativeGrep('tasks/lessons.md', keywords);
  sections.push(lessons || '  (no relevant lessons)');

  // 5. Tool registry
  sections.push('');
  sections.push('REGISTRY:');
  const registry = await nativeGrep('_reference/tool-registry.md', keywords);
  sections.push(registry || '  (no registry match)');

  // 6. Ecosystem
  sections.push('');
  sections.push('ECOSYSTEM:');
  const ecosystem = await nativeGrep('ecosystem.md', keywords);
  sections.push(ecosystem || '  (no ecosystem match)');

  // 7. Research cache
  sections.push('');
  sections.push('CACHE:');
  const cache = await nativeGrep('tasks/.research-cache.md', keywords);
  sections.push(cache || '  (no cached research)');

  return sections.join('\n');
}

export async function runVerify(size: string): Promise<string> {
  const sections: string[] = [];
  let pass = 0, fail = 0, manual = 0;

  sections.push(`=== VERIFICATION (size: ${size}) ===`);
  sections.push('');

  // Modified files (git is safe — no user input)
  let modified: string[] = [];
  try {
    const { stdout } = await exec('git diff --name-only HEAD', { timeout: 3000 });
    const { stdout: staged } = await exec('git diff --cached --name-only', { timeout: 3000 });
    modified = [...new Set([...stdout.trim().split('\n'), ...staged.trim().split('\n')].filter(Boolean))];
  } catch (err) {
    sections.push(`⚠ git: ${err instanceof Error ? err.message : 'not available'}`);
  }

  if (modified.length === 0) {
    sections.push('No modified files detected.');
    return sections.join('\n');
  }

  sections.push(`Modified: ${modified.join(', ')}`);
  sections.push('');

  // Gate 0: File sizes
  sections.push('--- File sizes ---');
  for (const f of modified) {
    if (!existsSync(f)) continue;
    const lines = countLines(f);
    if (lines > 375) {
      sections.push(`  ✗ ${f}: ${lines} lines (limit 375)`);
      fail++;
    } else {
      sections.push(`  ✓ ${f}: ${lines}/375`);
      pass++;
    }
  }

  // Gate 0: Syntax (bash -n — safe, only runs on .sh files we control)
  sections.push('');
  sections.push('--- Syntax ---');
  for (const f of modified) {
    if (!existsSync(f) || !f.endsWith('.sh')) continue;
    try {
      await exec(`bash -n ${shellEscape(f)}`, { timeout: 3000 });
      sections.push(`  ✓ bash: ${f}`);
      pass++;
    } catch (err) {
      const msg = err instanceof Error ? err.message.split('\n')[0] : 'syntax error';
      sections.push(`  ✗ bash: ${f} — ${msg}`);
      fail++;
    }
  }

  // Gate 1: Intent (S+)
  if (size !== 'XS') {
    sections.push('');
    sections.push('--- Intent (MANUAL) ---');
    sections.push('  → Does this match the user\'s actual request?');
    sections.push('  → Is tasks/current.md plan updated?');
    manual += 2;
  }

  // Gate 2: Quality (M+)
  if (['M', 'L', 'XL'].includes(size)) {
    sections.push('');
    sections.push('--- Quality (MANUAL) ---');
    sections.push('  → What is the WEAKEST part?');
    sections.push('  → What alternative did you consider?');
    sections.push('  → New shared utils registered in tool-registry?');
    manual += 3;
  }

  // Gate 3-4: Full (L/XL)
  if (['L', 'XL'].includes(size)) {
    sections.push('');
    sections.push('--- Full (MANUAL) ---');
    sections.push('  → Pre-mortem: if this fails, why?');
    sections.push('  → User checkpoint reached?');
    manual += 2;
  }

  sections.push('');
  sections.push(`RESULT: ${pass} auto ✓, ${fail} auto ✗, ${manual} manual remaining`);
  if (fail > 0) sections.push('STATUS: FIX auto failures before proceeding.');
  else sections.push(`STATUS: Auto PASS. Complete ${manual} manual check(s).`);

  return sections.join('\n');
}

export async function runPlanScaffold(task: string): Promise<string> {
  const sections: string[] = [];
  const words = task.split(/\s+/).filter(w => w.length > 2).slice(0, 5);
  let affectedFiles: string[] = [];

  for (const word of words) {
    const found = await nativeFindByKeyword(word);
    affectedFiles.push(...found);
  }
  affectedFiles = [...new Set(affectedFiles)].slice(0, 15);

  let size = 'S';
  if (affectedFiles.length <= 1) size = 'XS';
  else if (affectedFiles.length <= 2) size = 'S';
  else if (affectedFiles.length <= 7) size = 'M';
  else if (affectedFiles.length <= 15) size = 'L';
  else size = 'XL';

  sections.push(`## Plan — ${task}`);
  sections.push('');
  sections.push('### Goal');
  sections.push(task);
  sections.push('');
  sections.push('### Complexity Estimate');
  sections.push(`- Size: ${size}`);
  sections.push(`- Files to modify: ~${affectedFiles.length} (estimated)`);
  sections.push('- Files to create: [FILL IN]');
  sections.push('- Risk: [LOW/MEDIUM/HIGH — FILL IN]');
  sections.push('');
  sections.push('### File Architecture');
  if (affectedFiles.length > 0) {
    for (const f of affectedFiles) {
      const lines = existsSync(f) ? countLines(f) : 0;
      sections.push(`  ${f}  — [MODIFY] ${lines} lines`);
    }
  } else {
    sections.push('  [No matching files found. Fill in manually.]');
  }
  sections.push('');
  sections.push('### Implementation Order');
  sections.push('1. [FILL IN — types first]');
  sections.push('2. [FILL IN — core logic]');
  sections.push('3. [FILL IN — tests]');
  sections.push('');
  sections.push('### Plan B');
  sections.push('- Alternative: [FILL IN]');
  sections.push('- Trigger: [FILL IN]');

  return sections.join('\n');
}

// --- Helpers ---

function findSourceFiles(dir: string, max: number): string[] {
  const results: string[] = [];
  const exts = ['.ts', '.tsx', '.js', '.jsx', '.py', '.go', '.rs', '.vue', '.svelte'];
  try {
    const walk = (d: string) => {
      if (results.length >= max) return;
      for (const entry of readdirSync(d)) {
        if (entry === 'node_modules' || entry === '.git' || entry === 'dist') continue;
        const full = join(d, entry);
        try {
          const s = statSync(full);
          if (s.isDirectory()) walk(full);
          else if (exts.some(e => full.endsWith(e)) && !full.includes('.test.') && !full.includes('.spec.')) {
            results.push(full);
          }
        } catch { /* skip inaccessible */ }
      }
    };
    walk(dir);
  } catch { /* dir inaccessible */ }
  return results;
}

function countLines(file: string): number {
  try {
    return readFileSync(file, 'utf-8').split('\n').length;
  } catch {
    return 0;
  }
}
