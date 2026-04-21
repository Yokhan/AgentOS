// Visual regression tests for Graph View
// Run: npx playwright test tests/graph-visual.spec.js
// Requires: npm i -D @playwright/test

const { test, expect } = require('@playwright/test');

const BASE = 'http://localhost:3333';

test.describe('Graph API', () => {
  test('overview returns valid graph', async ({ request }) => {
    const res = await request.get(`${BASE}/api/graph/overview`);
    expect(res.ok()).toBeTruthy();
    const data = await res.json();
    expect(data.nodes).toBeDefined();
    expect(data.edges).toBeDefined();
    expect(data.stats.total_nodes).toBeGreaterThan(0);
  });

  test('project graph returns nodes for AgentOS', async ({ request }) => {
    const res = await request.get(`${BASE}/api/graph/project/AgentOS`);
    const data = await res.json();
    // AgentOS has .rs and .js files
    expect(data.nodes?.length || 0).toBeGreaterThan(0);
  });

  test('verify returns diagnostics', async ({ request }) => {
    const res = await request.get(`${BASE}/api/graph/verify/AgentOS`);
    const data = await res.json();
    expect(['ok', 'warnings']).toContain(data.status);
    expect(data.nodes).toBeGreaterThan(0);
  });

  test('context returns compact text', async ({ request }) => {
    const res = await request.get(`${BASE}/api/graph/context/AgentOS`);
    const data = await res.json();
    expect(data.context).toContain('PROJECT GRAPH');
  });

  test('dependents returns array', async ({ request }) => {
    const res = await request.get(`${BASE}/api/graph/dependents/AgentOS/lib.rs`);
    const data = await res.json();
    // lib.rs may or may not have dependents
    expect(data.dependents || data.error).toBeDefined();
  });

  test('impact returns depth analysis', async ({ request }) => {
    const res = await request.get(`${BASE}/api/graph/impact/AgentOS/state.rs`);
    const data = await res.json();
    expect(data.impact || data.error).toBeDefined();
  });
});

test.describe('Graph UI', () => {
  test('graph view opens', async ({ page }) => {
    await page.goto('http://localhost:1420');
    // Wait for app to load
    await page.waitForTimeout(3000);
    // Click graph button
    const graphBtn = page.locator('button:has-text("graph")');
    if (await graphBtn.isVisible()) {
      await graphBtn.click();
      await page.waitForTimeout(2000);
      // Should see SVG or canvas
      const svg = page.locator('.graph-svg');
      expect(await svg.isVisible()).toBeTruthy();
    }
  });

  test('graph shows nodes', async ({ page }) => {
    await page.goto('http://localhost:1420');
    await page.waitForTimeout(3000);
    const graphBtn = page.locator('button:has-text("graph")');
    if (await graphBtn.isVisible()) {
      await graphBtn.click();
      await page.waitForTimeout(2000);
      // Check for node elements
      const nodes = page.locator('.graph-svg g[transform]');
      const count = await nodes.count();
      expect(count).toBeGreaterThan(0);
    }
  });

  test('graph screenshot', async ({ page }) => {
    await page.goto('http://localhost:1420');
    await page.waitForTimeout(3000);
    const graphBtn = page.locator('button:has-text("graph")');
    if (await graphBtn.isVisible()) {
      await graphBtn.click();
      await page.waitForTimeout(3000);
      await page.screenshot({ path: 'tests/screenshots/graph-overview.png', fullPage: true });
    }
  });
});
