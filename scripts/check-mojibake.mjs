import fs from "node:fs";
import path from "node:path";

const root = process.cwd();

const includeEntries = [
  "src-ui",
  "src-tauri/src",
  "scripts",
  "docs",
  "tasks/plans",
  "tasks/RELEASE_NOTES.md",
];

const extensions = new Set([".js", ".css", ".rs", ".md", ".mjs", ".json", ".toml"]);
const excludedDirs = new Set([
  ".git",
  "node_modules",
  "target",
  "dist",
  "build",
  ".next",
  ".vite",
]);
const excludedFiles = new Set([
  path.normalize("scripts/check-mojibake.mjs"),
]);

const markerEscapes = [
  "\\u0412\\u00b7",
  "\\u0420\\u2019\\u0412\\u00b7",
  "\\u0432\\u0402",
  "\\u0432\\u2020",
  "\\u0432\\u045a",
  "\\u0432\\u2013",
  "\\u0432\\u2014",
  "\\u0432\\u20ac",
  "\\u0413\\u2014",
  "\\u0421\\u0402\\u0421\\u045f",
  "\\u0420\\u045f",
  "\\u0420\\u0457",
  "\\u0420\\u0456",
  "\\u0420\\u00b5",
  "\\u0420\\u00b0",
  "\\u0420\\u00b1",
  "\\u0420\\u0491",
  "\\u0420\\u00bb",
  "\\u0421\\u0403",
  "\\u0421\\u201a",
  "\\u0421\\u0402",
  "\\u0421\\u040f",
  "\\u0421\\u2020",
  "\\u0421\\u2039",
  "\\u0421\\u040a",
].map((value) => JSON.parse(`"${value}"`));

function walk(entry) {
  const fullPath = path.join(root, entry);
  if (!fs.existsSync(fullPath)) return [];
  const stat = fs.statSync(fullPath);
  if (stat.isFile()) return [entry];
  const files = [];
  for (const name of fs.readdirSync(fullPath)) {
    if (excludedDirs.has(name)) continue;
    const rel = path.join(entry, name);
    const child = path.join(root, rel);
    const childStat = fs.statSync(child);
    if (childStat.isDirectory()) {
      files.push(...walk(rel));
    } else if (extensions.has(path.extname(name))) {
      files.push(rel);
    }
  }
  return files;
}

const files = [...new Set(includeEntries.flatMap(walk))]
  .map((file) => path.normalize(file))
  .filter((file) => !excludedFiles.has(file))
  .sort();

const failures = [];
for (const file of files) {
  const fullPath = path.join(root, file);
  const text = fs.readFileSync(fullPath, "utf8");
  const lines = text.split(/\r?\n/);
  lines.forEach((line, index) => {
    for (const marker of markerEscapes) {
      if (line.includes(marker)) {
        failures.push(`${file}:${index + 1}: ${line.trim()}`);
        break;
      }
    }
  });
}

if (failures.length) {
  console.error("mojibake markers found:");
  failures.slice(0, 80).forEach((line) => console.error("  " + line));
  if (failures.length > 80) console.error(`  ...and ${failures.length - 80} more`);
  process.exit(1);
}

console.log(`mojibake check ok (${files.length} files)`);
