// AgentOS utility functions — shared across components
// SAFETY: esc() escapes &<> BEFORE regex — all injected HTML tags are static.
export const esc = s => (s || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

export function md(s) {
  if (!s) return '';
  return esc(s)
    .replace(/```([\s\S]*?)```/g, '<pre><code>$1</code></pre>')
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
    .replace(/\*([^*]+)\*/g, '<em>$1</em>')
    .replace(/^- (.+)$/gm, '• $1')
    .replace(/\n/g, '<br>');
}

export const ft = t => {
  try {
    const d = new Date(t || Date.now());
    const s = Math.floor((Date.now() - d.getTime()) / 1000);
    if (s < 60) return s + 's ago';
    if (s < 3600) return Math.floor(s / 60) + 'm ago';
    if (s < 86400) return Math.floor(s / 3600) + 'h ago';
    return Math.floor(s / 86400) + 'd ago';
  } catch { return ''; }
};

export const SC = { working: 'var(--green)', idle: 'var(--yellow)', sleeping: 'var(--mute)', blocked: 'var(--accent)' };
export const SL = { working: 'active', idle: 'idle', sleeping: 'sleeping', blocked: 'blocked' };

export function beep() {
  try {
    const a = new AudioContext(), o = a.createOscillator(), g = a.createGain();
    o.connect(g); g.connect(a.destination);
    o.frequency.value = 800; g.gain.value = 0.08;
    o.start(); g.gain.exponentialRampToValueAtTime(0.001, a.currentTime + 0.12);
    o.stop(a.currentTime + 0.12);
  } catch {}
}

export const SLASH_CMDS = [
  { cmd: '/clear', desc: 'Clear chat' },
  { cmd: '/mode-code', desc: 'Code mode' },
  { cmd: '/mode-design', desc: 'Design mode' },
  { cmd: '/mode-review', desc: 'Review mode' },
  { cmd: '/mode-fix', desc: 'Fix mode' },
  { cmd: '/status', desc: 'Status' },
  { cmd: '/health', desc: 'Health check' },
  { cmd: '/briefing', desc: 'Briefing' },
  { cmd: '/help', desc: 'Commands list' },
];
