"use strict";

const fs = require("fs");
const path = require("path");

function resolveRepoFile(root, value, options = {}) {
  const rootPath = path.resolve(root);
  const candidate = path.resolve(rootPath, typeof value === "string" ? value : "");
  const relative = path.relative(rootPath, candidate);
  const normalized = relative.split(path.sep).join("/");
  if (!relative || relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative)) {
    return { error: "must resolve to a file inside the repository" };
  }
  if ((options.forbidPrefixes || []).some((prefix) => normalized.startsWith(prefix))) {
    return { error: `cannot use forbidden path: ${normalized}` };
  }
  try {
    if (!fs.statSync(candidate).isFile()) return { error: `is not a file: ${normalized}` };
  } catch {
    return { error: `does not exist: ${normalized}` };
  }
  return { file: candidate, relative: normalized };
}

module.exports = { resolveRepoFile };
