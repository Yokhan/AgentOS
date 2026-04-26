import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const storePath = path.join(root, "src-ui", "store.js");
const chatPath = path.join(root, "src-ui", "chat.js");

const storeSource = fs.readFileSync(storePath, "utf8");
const chatSource = fs.readFileSync(chatPath, "utf8");

const storeImport = chatSource.match(
  /import\s*\{([^}]*)\}\s*from\s*["']\/store\.js["']/,
);

if (!storeImport) {
  console.error("Missing /store.js import block in src-ui/chat.js.");
  process.exit(1);
}

const imported = new Set(
  storeImport[1]
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean),
);

const storeSignals = new Set(
  [...storeSource.matchAll(/const\s+([A-Za-z_$][\w$]*)\s*=\s*signal\(/g)].map(
    (match) => match[1],
  ),
);

const usedStoreSignals = new Set(
  [...chatSource.matchAll(/\b([A-Za-z_$][\w$]*)\.value\b/g)]
    .map((match) => match[1])
    .filter((name) => storeSignals.has(name)),
);

const missing = [...usedStoreSignals].filter((name) => !imported.has(name));

if (missing.length > 0) {
  console.error(
    `Missing /store.js signal imports in src-ui/chat.js: ${missing.join(", ")}`,
  );
  process.exit(1);
}

console.log(`store signal imports ok: ${usedStoreSignals.size}`);
