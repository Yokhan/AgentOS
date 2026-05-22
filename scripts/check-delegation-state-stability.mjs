import fs from "node:fs";

function assertContains(file, needle, message) {
  const text = fs.readFileSync(file, "utf8");
  if (!text.includes(needle)) {
    throw new Error(`${message}: missing ${needle} in ${file}`);
  }
}

const api = "src-ui/api.js";
const bridge = "src-ui/bridge.js";
const delegationRs = "src-tauri/src/commands/delegation.rs";

assertContains(api, "DELEGATION_LOCAL_HINT_TTL_MS", "local delegation hints need a bounded TTL");
assertContains(api, "shouldPreserveMissingDelegation", "snapshot replace must preserve fresh local hints");
assertContains(api, "markLocalDelegationHint", "chat/stream delegation tags must be marked as local hints");
assertContains(api, 'const r = await fetch("/api/delegations")', "delegation snapshot fetch must match GET backend route");
assertContains(bridge, 'method === "GET" || method === "POST"', "desktop bridge must support both old POST and current GET delegation fetches");
assertContains(delegationRs, ".delegation-archive.jsonl", "backend delegation snapshot must include recent archive rows");
assertContains(delegationRs, '"archived"', "archived snapshot rows need an explicit marker");

console.log("delegation state stability checks ok");
