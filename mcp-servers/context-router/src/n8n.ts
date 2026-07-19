/**
 * n8n Bridge — trigger and query n8n workflows via HTTP API
 * n8n must be running at N8N_URL (default: http://localhost:5678)
 * Each project configures its own webhook endpoints in n8n.
 */

const N8N_URL = process.env.N8N_URL || 'http://localhost:5678';
const N8N_API_KEY = process.env.N8N_API_KEY || '';
const TIMEOUT = 30_000; // 30s for pipeline execution

function headers(): Record<string, string> {
  const h: Record<string, string> = { 'Content-Type': 'application/json' };
  if (N8N_API_KEY) h['X-N8N-API-KEY'] = N8N_API_KEY;
  return h;
}

export async function runPipeline(
  name: string,
  params?: Record<string, string>
): Promise<string> {
  const url = `${N8N_URL}/webhook/${name}`;
  try {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), TIMEOUT);

    const response = await fetch(url, {
      method: 'POST',
      headers: headers(),
      body: JSON.stringify({
        pipeline: name,
        params: params || {},
        triggered_by: 'claude-mcp',
        timestamp: new Date().toISOString(),
      }),
      signal: controller.signal,
    });

    clearTimeout(timeout);

    if (!response.ok) {
      return `ERROR: n8n returned ${response.status} ${response.statusText} for webhook/${name}`;
    }

    const result = await response.json();
    return JSON.stringify(result, null, 2);
  } catch (err: unknown) {
    if (err instanceof Error && err.name === 'AbortError') {
      return `ERROR: Pipeline "${name}" timed out after ${TIMEOUT / 1000}s`;
    }
    return `ERROR: Cannot reach n8n at ${url}. Is n8n running? (${err instanceof Error ? err.message : 'unknown error'})`;
  }
}

export async function listPipelines(): Promise<string> {
  const url = `${N8N_URL}/api/v1/workflows?active=true`;
  try {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 5000);

    const response = await fetch(url, {
      headers: headers(),
      signal: controller.signal,
    });

    clearTimeout(timeout);

    if (!response.ok) {
      return `ERROR: n8n API returned ${response.status}. Check N8N_API_KEY in .env`;
    }

    const data = await response.json() as { data?: Array<{ id: string; name: string; active: boolean }> };
    if (!data.data || data.data.length === 0) {
      return 'No active workflows in n8n.';
    }

    const lines = ['ACTIVE PIPELINES:'];
    for (const wf of data.data) {
      lines.push(`  ${wf.name} (id: ${wf.id}, active: ${wf.active})`);
    }
    return lines.join('\n');
  } catch {
    return `ERROR: Cannot reach n8n at ${N8N_URL}. Is n8n running?`;
  }
}

export async function pipelineStatus(executionId: string): Promise<string> {
  const url = `${N8N_URL}/api/v1/executions/${executionId}`;
  try {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 5000);

    const response = await fetch(url, {
      headers: headers(),
      signal: controller.signal,
    });

    clearTimeout(timeout);

    if (!response.ok) {
      return `ERROR: n8n API returned ${response.status} for execution ${executionId}`;
    }

    const data = await response.json() as { finished: boolean; status: string; data?: unknown };
    return JSON.stringify({
      execution_id: executionId,
      finished: data.finished,
      status: data.status,
    }, null, 2);
  } catch {
    return `ERROR: Cannot reach n8n at ${N8N_URL}.`;
  }
}
