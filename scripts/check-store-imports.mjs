import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const storePath = path.join(root, "src-ui", "store.js");
const srcUiPath = path.join(root, "src-ui");

const storeSource = fs.readFileSync(storePath, "utf8");
const storeSignals = new Set(
  [...storeSource.matchAll(/const\s+([A-Za-z_$][\w$]*)\s*=\s*signal\(/g)].map(
    (match) => match[1],
  ),
);

let checked = 0;
const failures = [];

for (const entry of fs.readdirSync(srcUiPath)) {
  if (!entry.endsWith(".js") || entry === "store.js") continue;
  const filePath = path.join(srcUiPath, entry);
  const source = fs.readFileSync(filePath, "utf8");
  const usedStoreSignals = new Set(
    [...source.matchAll(/\b([A-Za-z_$][\w$]*)\.value\b/g)]
      .map((match) => match[1])
      .filter((name) => storeSignals.has(name)),
  );
  if (!usedStoreSignals.size) continue;
  checked += 1;

  const imports = [
    ...source.matchAll(/import\s*\{([\s\S]*?)\}\s*from\s*["']([^"']+)["']/g),
  ].filter((match) => match[2] === "/store.js");
  const imported = new Set(
    imports
      .flatMap((match) => match[1].split(","))
      .map((item) =>
        item
          .trim()
          .split(/\s+as\s+/)
          .pop(),
      )
      .filter(Boolean),
  );
  const missing = [...usedStoreSignals].filter((name) => !imported.has(name));
  if (missing.length > 0) {
    failures.push(`${entry}: ${missing.join(", ")}`);
  }
}

if (failures.length > 0) {
  console.error(`Missing /store.js signal imports:\n${failures.join("\n")}`);
  process.exit(1);
}

console.log(`store signal imports ok: ${checked} files`);
