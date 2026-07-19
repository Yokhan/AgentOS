#!/usr/bin/env node
const assert = require("assert");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");

const ROOT = path.resolve(__dirname, "..");
const VALIDATOR = path.join(ROOT, "scripts", "validate-design-policy.js");

function run(args, options = {}) {
  return spawnSync(process.execPath, [VALIDATOR, ...args], {
    cwd: options.cwd || ROOT,
    encoding: "utf8",
    env: { ...process.env, ...(options.env || {}) },
  });
}

function main() {
  const pass = run(["--path", "tests/fixtures/design-policy/pass/basic.css"]);
  assert.strictEqual(pass.status, 0, pass.stderr || pass.stdout);

  const fail = run(["--path", "tests/fixtures/design-policy/fail/gradient-text.css"]);
  assert.notStrictEqual(fail.status, 0, "gradient text fixture must fail");
  assert.match(fail.stderr, /design\/no-gradient-text/);
  assert.match(fail.stderr, /Impact:/);
  assert.match(fail.stderr, /Tune:/);

  const hook = run(["--hook"], {
    env: {
      FILE_PATH: "tests/fixtures/design-policy/fail/gradient-text.css",
      TOOL_NAME: "apply_patch",
    },
  });
  assert.strictEqual(hook.status, 0, hook.stderr || hook.stdout);
  assert.match(hook.stderr, /Design policy notification/);

  const temp = fs.mkdtempSync(path.join(os.tmpdir(), "design-policy-"));
  try {
    const target = path.join(temp, "gradient-text.css");
    fs.copyFileSync(
      path.join(ROOT, "tests/fixtures/design-policy/fail/gradient-text.css"),
      target,
    );
    fs.writeFileSync(
      path.join(temp, "design-policy.ignore"),
      "design/no-gradient-text gradient-text.css legacy-approved-exception\n",
      "utf8",
    );
    const ignored = run(["--path", "gradient-text.css"], { cwd: temp });
    assert.strictEqual(ignored.status, 0, ignored.stderr || ignored.stdout);
  } finally {
    fs.rmSync(temp, { force: true, recursive: true });
  }

  console.log("Design policy tests passed");
}

main();
