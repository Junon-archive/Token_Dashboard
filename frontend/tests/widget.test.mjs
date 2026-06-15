import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import {
  formatResetCountdown,
  formatRemainingMinutes,
  providerView,
  remainingFraction,
  renderClaudeWidget,
  renderPomodoroWidget,
  renderUsageDashboard,
  renderUsageWidget,
  ticksSvg,
  visualClassForSnapshot,
} from '../src/widget.js';

const NOW = new Date('2026-06-12T00:00:00.000Z');

function snapshot(state, usedPct = 34, provider = 'claude') {
  return {
    provider,
    state,
    primary: {
      used_pct: usedPct,
      resets_at: '2026-06-12T03:17:00.000Z',
    },
    secondary: {
      used_pct: 8,
      resets_at: '2026-06-16T00:00:00.000Z',
    },
    extra: null,
    fetched_at: '2026-06-11T23:48:00.000Z',
    is_stale: state === 'STALE',
    error: null,
  };
}

test('maps seven logical states to design-reference visual classes', () => {
  assert.equal(visualClassForSnapshot(snapshot('NORMAL')), '');
  assert.equal(visualClassForSnapshot(snapshot('WARN', 80)), 'low');
  assert.equal(visualClassForSnapshot(snapshot('CRITICAL', 95)), 'critical');
  assert.equal(visualClassForSnapshot(snapshot('CRITICAL', 100)), 'depleted');
  assert.equal(visualClassForSnapshot(snapshot('STALE')), 'stale');
  assert.equal(visualClassForSnapshot(snapshot('RATE_LIMITED')), 'stale');
  assert.equal(visualClassForSnapshot(snapshot('NOT_LOGGED_IN')), 'notin');
  assert.equal(visualClassForSnapshot(snapshot('AUTH_ERROR')), 'autherr');
});

test('renders Claude widget DOM with expected gauge structure', () => {
  const html = renderUsageWidget(snapshot('NORMAL', 34), { now: NOW });

  assert.match(html, /class="widget claude"/);
  assert.match(html, /data-provider="claude"/);
  assert.match(html, /data-state="NORMAL"/);
  assert.match(html, /data-tauri-drag-region="deep"/);
  assert.match(html, /<div class="num">3:17<\/div>/);
  assert.match(html, /<div class="lbl">Claude<\/div>/);
  assert.match(html, /<div class="update-badge">/);
  assert.match(html, /12m ago/);
  assert.equal((html.match(/class="tick/g) ?? []).length, 48);
  assert.equal((html.match(/class="arc-main"/g) ?? []).length, 1);
  assert.equal((html.match(/class="arc-sec"/g) ?? []).length, 1);
});

test('keeps Claude wrapper compatible with the generic usage renderer', () => {
  assert.equal(
    renderClaudeWidget(snapshot('NORMAL', 34, 'codex'), { now: NOW }),
    renderUsageWidget(snapshot('NORMAL', 34, 'claude'), { now: NOW }),
  );
});

test('renders Codex usage widget through the generic renderer', async () => {
  const html = renderUsageWidget(snapshot('WARN', 80, 'codex'), { now: NOW });
  const css = await readFile(new URL('../src/styles.css', import.meta.url), 'utf8');

  assert.match(html, /class="widget codex low"/);
  assert.match(html, /data-provider="codex"/);
  assert.match(html, /aria-label="Codex usage widget"/);
  assert.match(html, /<div class="lbl">Codex<\/div>/);
  assert.doesNotMatch(html, /Claude/);
  assert.match(css, /--codex: #46c2d4;/);
  assert.match(css, /--codex-bright: #62d4e3;/);
  assert.match(css, /\.widget\.codex\s*\{\s*--brand: var\(--codex\);/);
});

test('renders Claude and Codex widgets in a horizontal dashboard', async () => {
  const html = renderUsageDashboard([
    snapshot('NORMAL', 34, 'claude'),
    snapshot('WARN', 82, 'codex'),
  ], { now: NOW });
  const css = await readFile(new URL('../src/styles.css', import.meta.url), 'utf8');

  assert.match(html, /<main class="dashboard" aria-label="Token usage dashboard">/);
  assert.match(html, /class="widget claude"/);
  assert.match(html, /class="widget codex low"/);
  assert.equal((html.match(/class="widget /g) ?? []).length, 2);
  assert.equal((html.match(/class="arc-main"/g) ?? []).length, 2);
  assert.equal((html.match(/class="arc-sec"/g) ?? []).length, 2);
  assert.match(css, /\.dashboard\s*\{[^}]*display: flex;/s);
  assert.match(css, /\.dashboard\s*\{[^}]*gap: 20px;/s);
});

test('renders Pomodoro as a provider-isolated third widget', async () => {
  const timer = {
    provider: 'pomodoro',
    state: 'FOCUS',
    phase: 'FOCUS',
    action_label: 'Pause',
    primary: {
      used_pct: 10,
      resets_at: '2026-06-12T00:20:00.000Z',
    },
    secondary: null,
    fetched_at: '2026-06-12T00:00:00.000Z',
    is_stale: false,
  };
  const html = renderUsageDashboard([
    { ...snapshot('STALE', 34, 'claude'), primary: null, secondary: null },
    { ...snapshot('AUTH_ERROR', 34, 'codex'), primary: null, secondary: null },
    timer,
  ], { now: NOW });
  const css = await readFile(new URL('../src/styles.css', import.meta.url), 'utf8');

  assert.match(html, /class="widget claude stale"/);
  assert.match(html, /class="widget codex autherr"/);
  assert.match(html, /class="widget pomodoro focus"/);
  assert.match(html, /data-provider="pomodoro"/);
  assert.match(html, /data-state="FOCUS"/);
  assert.match(html, /aria-label="Pomodoro timer widget"/);
  assert.match(html, /<div class="num">20<\/div>/);
  assert.match(html, /<div class="lbl">Focus<\/div>/);
  assert.match(html, /role="toolbar" aria-label="Pomodoro controls"/);
  assert.match(html, /data-pomodoro-action="toggle"/);
  assert.match(html, /data-pomodoro-action="reset"/);
  assert.match(html, /data-pomodoro-action="skip"/);
  assert.equal((html.match(/class="widget /g) ?? []).length, 3);
  assert.equal((html.match(/class="arc-sec"/g) ?? []).length, 0);
  assert.match(css, /--pomodoro-focus: #f0563d;/);
  assert.match(css, /--pomodoro-break: #5fb98c;/);
});

test('renders Pomodoro break and paused states without critical pulse', async () => {
  const base = {
    provider: 'pomodoro',
    primary: {
      used_pct: 25,
      resets_at: '2026-06-12T00:05:00.000Z',
    },
    secondary: null,
    fetched_at: '2026-06-12T00:00:00.000Z',
    is_stale: false,
  };
  const css = await readFile(new URL('../src/styles.css', import.meta.url), 'utf8');

  assert.match(renderPomodoroWidget({ ...base, state: 'BREAK', phase: 'BREAK' }, { now: NOW }), /aria-label="Start focus"/);
  assert.match(renderPomodoroWidget({ ...base, state: 'PAUSED', phase: 'FOCUS', action_label: 'Resume' }, { now: NOW }), /class="widget pomodoro paused"/);
  assert.match(renderPomodoroWidget({ ...base, state: 'PAUSED', phase: 'FOCUS', action_label: 'Resume' }, { now: NOW }), /aria-label="Resume timer"/);
  assert.doesNotMatch(css, /\.widget\.pomodoro[^}]*animation:/s);
});

test('renders provider-neutral state classes for Claude and Codex', () => {
  for (const provider of ['claude', 'codex']) {
    assert.match(renderUsageWidget(snapshot('CRITICAL', 100, provider), { now: NOW }), new RegExp(`widget ${provider} depleted`));
    assert.match(renderUsageWidget(snapshot('RATE_LIMITED', 34, provider), { now: NOW }), new RegExp(`widget ${provider} stale`));
    assert.match(renderUsageWidget(snapshot('AUTH_ERROR', 34, provider), { now: NOW }), new RegExp(`widget ${provider} autherr`));
  }
});

test('handles incomplete provider snapshots without crashing', () => {
  const noSecondary = renderUsageWidget({ ...snapshot('NORMAL', 34, 'codex'), secondary: null }, { now: NOW });
  assert.match(noSecondary, /class="widget codex"/);
  assert.equal((noSecondary.match(/class="arc-sec"/g) ?? []).length, 0);

  const missingPrimary = renderUsageWidget({ ...snapshot('NORMAL', 34, 'codex'), primary: null }, { now: NOW });
  assert.match(missingPrimary, /<div class="num">--:--<\/div>/);

  const badFetchedAt = renderUsageWidget({ ...snapshot('STALE', 34, 'codex'), fetched_at: 'not-a-date' }, { now: NOW });
  assert.match(badFetchedAt, />stale<\/span>/);
});

test('falls back to Claude view for unknown providers', () => {
  assert.deepEqual(providerView('unknown'), providerView('claude'));
});

test('renders stale and auth states without dropping last values', () => {
  const staleHtml = renderClaudeWidget(snapshot('STALE', 42), { now: NOW });
  assert.match(staleHtml, /class="widget claude stale"/);
  assert.match(staleHtml, /12m ago/);
  assert.match(staleHtml, /<div class="num">3:17<\/div>/);

  const signInHtml = renderClaudeWidget(snapshot('NOT_LOGGED_IN'), { now: NOW });
  assert.match(signInHtml, /class="widget claude notin"/);
  assert.match(signInHtml, /Sign in/);

  const authHtml = renderClaudeWidget(snapshot('AUTH_ERROR'), { now: NOW });
  assert.match(authHtml, /class="widget claude autherr"/);
  assert.match(authHtml, /Auth/);
});

test('uses remaining quota for ring fractions', () => {
  assert.equal(remainingFraction({ used_pct: 0 }), 1);
  assert.equal(remainingFraction({ used_pct: 34 }), 0.6599999999999999);
  assert.equal(remainingFraction({ used_pct: 100 }), 0);
  assert.equal(remainingFraction({ used_pct: 125 }), 0);
});

test('formats reset countdown as H:MM', () => {
  assert.equal(formatResetCountdown('2026-06-12T03:17:00.000Z', NOW), '3:17');
  assert.equal(formatResetCountdown('2026-06-12T00:00:00.000Z', NOW), '0:00');
  assert.equal(formatResetCountdown('not-a-date', NOW), '--:--');
});

test('formats Pomodoro remaining time as a single minute number', () => {
  assert.equal(formatRemainingMinutes('2026-06-12T00:20:00.000Z', NOW), '20');
  assert.equal(formatRemainingMinutes('2026-06-12T00:00:00.000Z', NOW), '0');
  assert.equal(formatRemainingMinutes('not-a-date', NOW), '--');
});

test('critical pulse exists only on critical arc and reduced motion disables it', async () => {
  const css = await readFile(new URL('../src/styles.css', import.meta.url), 'utf8');

  assert.match(css, /\.widget\.critical \.arc-main\s*\{\s*animation: criticalPulse 1800ms cubic-bezier\(\.45, 0, \.2, 1\) infinite;/);
  assert.match(css, /@media \(prefers-reduced-motion: reduce\)/);
  assert.match(css, /\.widget\.critical \.arc-main\s*\{\s*animation: none;/);
  assert.doesNotMatch(css, /\.widget\.low \.arc-main\s*\{[^}]*animation:/s);
  assert.doesNotMatch(css, /\.widget\.stale \.arc-main\s*\{[^}]*animation:/s);
});

test('runtime widget avoids hover visuals that persist in transparent windows', async () => {
  const css = await readFile(new URL('../src/styles.css', import.meta.url), 'utf8');
  const js = await readFile(new URL('../src/main.js', import.meta.url), 'utf8');

  assert.match(js, /import \{ renderUsageDashboard \} from '\.\/widget\.js';/);
  assert.match(js, /from '\.\/pomodoro\.js';/);
  assert.match(js, /tickPomodoro/);
  assert.match(js, /pomodoroSnapshot\(pomodoro\)/);
  assert.doesNotMatch(js, /pomodoroSnapshot\(\)/);
  assert.match(js, /usage_snapshots/);
  assert.doesNotMatch(js, /mock_usage_snapshots/);
  assert.match(js, /return fallbackSnapshots;/);
  assert.match(js, /return degradedSnapshots\(\);/);
  assert.match(js, /querySelectorAll\('\.widget'\)/);
  assert.doesNotMatch(js, /renderClaudeWidget/);
  assert.doesNotMatch(css, /\.widget:hover \.disk/);
  assert.doesNotMatch(css, /\.widget:hover \.arc-main/);
  assert.match(js, /classList\.add\('is-hovered'\)/);
  assert.match(js, /classList\.remove\('is-hovered'\)/);
  assert.match(js, /setInterval\(renderDashboard, 60000\)/);
  assert.doesNotMatch(js, /setInterval\(renderDashboard, 1000\)/);
  assert.match(js, /closest\('\[data-no-drag="true"\]'\)/);
  assert.match(js, /addEventListener\('mouseleave', \(\) => clearHover\(widget\)\)/);
  assert.match(js, /startDragging/);
  assert.doesNotMatch(css, /transition: background/);
  assert.doesNotMatch(css, /\.widget\.is-hovered \.disk/);
  assert.doesNotMatch(css, /\.widget\.is-hovered \.num/);
  assert.doesNotMatch(css, /\.widget\.is-hovered [^{]+\{[^}]*box-shadow:/s);
  assert.doesNotMatch(css, /\.widget\.is-hovered [^{]+\{[^}]*drop-shadow/s);
});

test('Pomodoro controls are hover/focus only and excluded from window drag', async () => {
  const css = await readFile(new URL('../src/styles.css', import.meta.url), 'utf8');
  const js = await readFile(new URL('../src/main.js', import.meta.url), 'utf8');

  assert.match(css, /\.pomodoro-controls\s*\{[^}]*display: none;/s);
  assert.match(css, /\.widget\.pomodoro\.is-hovered \.pomodoro-controls,/);
  assert.match(css, /\.widget\.pomodoro:focus-within \.pomodoro-controls\s*\{\s*display: flex;/);
  assert.match(css, /\.pomodoro-controls\s*\{[^}]*-webkit-app-region: no-drag;/s);
  assert.match(css, /\.pomodoro-btn\s*\{[^}]*-webkit-app-region: no-drag;/s);
  assert.match(js, /data-pomodoro-action/);
});

test('Tauri widget window is wide enough for Claude and Codex gauges', async () => {
  const config = JSON.parse(await readFile(new URL('../../src-tauri/tauri.conf.json', import.meta.url), 'utf8'));
  const window = config.app.windows.find((item) => item.label === 'claude-widget');

  assert.equal(config.app.withGlobalTauri, true);
  assert.equal(window.width, 500);
  assert.equal(window.minWidth, 500);
  assert.equal(window.height, 180);
});

test('hover exposes last update age without changing transparent disk visuals', async () => {
  const css = await readFile(new URL('../src/styles.css', import.meta.url), 'utf8');

  assert.match(css, /\.widget\.is-hovered \.update-badge\s*\{\s*display: flex;/);
  assert.match(css, /\.widget\.stale \.update-badge\s*\{\s*display: flex;/);
});

test('generates forty-eight block-style gauge ticks', () => {
  const ticks = ticksSvg();

  assert.equal((ticks.match(/<rect class="tick/g) ?? []).length, 48);
  assert.equal((ticks.match(/class="tick tick-major"/g) ?? []).length, 8);
  assert.match(ticks, /rx="0\.70"/);
  assert.match(ticks, /transform="rotate\(45\.00 70 70\)"/);
});

test('tick marks use subtle block fills instead of thin strokes', async () => {
  const css = await readFile(new URL('../src/styles.css', import.meta.url), 'utf8');

  assert.match(css, /--tick: rgba\(255, 255, 255, \.24\);/);
  assert.match(css, /\.tick\s*\{\s*fill: var\(--tick\);/);
  assert.match(css, /\.tick-major\s*\{\s*fill: var\(--arc\);\s*opacity: \.82;/);
  assert.doesNotMatch(css, /\.widget\.claude \.tick-major/);
  assert.doesNotMatch(css, /\.widget\.codex \.tick-major/);
  assert.doesNotMatch(css, /\.tick\s*\{[^}]*stroke-width:/s);
});

test('disk edge fades instead of ending with a hard border', async () => {
  const css = await readFile(new URL('../src/styles.css', import.meta.url), 'utf8');

  assert.match(css, /--disk-mask: radial-gradient\(circle,/);
  assert.match(css, /transparent 100%/);
  assert.match(css, /mask-image: var\(--disk-mask\);/);
  assert.match(css, /-webkit-mask-image: var\(--disk-mask\);/);
  assert.match(css, /\.disk\s*\{[^}]*border: 0;/s);
});
