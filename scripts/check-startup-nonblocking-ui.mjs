import fs from "node:fs";

const app = fs.readFileSync("src-ui/app.js", "utf8");
const renderIndex = app.indexOf("render(html`<${App} />`, document.body)");
const startupIndex = app.indexOf("runStartupLoad().catch");
const promiseAllIndex = app.indexOf("await Promise.all([");

if (renderIndex < 0) {
  throw new Error("main app render call not found");
}
if (startupIndex < 0) {
  throw new Error("startup loader must run after first render");
}
if (startupIndex < renderIndex) {
  throw new Error("startup loader must not block first render");
}
if (promiseAllIndex >= 0 && promiseAllIndex < renderIndex) {
  throw new Error("top-level Promise.all before first render can freeze startup UI");
}
if (!app.includes("startupTask(") || !app.includes("startup task timed out")) {
  throw new Error("startup tasks must have per-task timeouts");
}
if (!app.includes("startPolling();")) {
  throw new Error("polling must start only after startup loader settles");
}

console.log("startup nonblocking UI checks ok");
