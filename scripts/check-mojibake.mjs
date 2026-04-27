import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const files = [
  "src-ui/app.js",
  "src-ui/api.js",
  "src-ui/chat.js",
  "src-ui/pages.js",
  "src-ui/views.js",
  "src-ui/route-state.js",
  "src-ui/styles/chat.css",
  "src-ui/styles/main.css",
  "src-ui/styles/toolcards.css",
];

const markers = [
  "\\u0412\\u00b7", // Cyrillic Ve + middle dot, common mojibake for bullet separator
  "\\u0420\\u2019\\u0412\\u00b7",
  "\\u0432\\u0402",
  "\\u0432\\u2020",
  "\\u0432\\u045a",
  "\\u0432\\u2013",
  "\\u0432\\u2014",
  "\\u0432\\u20ac",
  "\\u0413\\u2014",
  "\\u0421\\u0402\\u0421\\u045f",
].map((value) => JSON.parse(`"${value}"`));

const failures = [];
for (const file of files) {
  const fullPath = path.join(root, file);
  if (!fs.existsSync(fullPath)) continue;
  const text = fs.readFileSync(fullPath, "utf8");
  const lines = text.split(/\r?\n/);
  lines.forEach((line, index) => {
    for (const marker of markers) {
      if (line.includes(marker)) {
        failures.push(`${file}:${index + 1}: ${line.trim()}`);
        break;
      }
    }
  });
}

if (failures.length) {
  console.error("mojibake markers found:");
  failures.slice(0, 40).forEach((line) => console.error("  " + line));
  if (failures.length > 40) console.error(`  ...and ${failures.length - 40} more`);
  process.exit(1);
}

console.log("mojibake check ok");
