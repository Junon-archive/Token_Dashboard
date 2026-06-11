import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import {
  formatResetCountdown,
  providerView,
  remainingFraction,
  renderClaudeWidget,
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
  assert.equal((html.match(/class="tick"/g) ?? []).length, 48);
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

  assert.match(js, /import \{ renderUsageWidget \} from '\.\/widget\.js';/);
  assert.doesNotMatch(js, /renderClaudeWidget/);
  assert.doesNotMatch(css, /\.widget:hover \.disk/);
  assert.doesNotMatch(css, /\.widget:hover \.arc-main/);
  assert.match(js, /classList\.add\('is-hovered'\)/);
  assert.match(js, /classList\.remove\('is-hovered'\)/);
  assert.match(js, /addEventListener\('mouseleave', clearHover\)/);
  assert.match(js, /startDragging/);
  assert.doesNotMatch(css, /transition: background/);
  assert.doesNotMatch(css, /\.widget\.is-hovered \.disk/);
  assert.doesNotMatch(css, /\.widget\.is-hovered \.num/);
  assert.doesNotMatch(css, /\.widget\.is-hovered [^{]+\{[^}]*box-shadow:/s);
  assert.doesNotMatch(css, /\.widget\.is-hovered [^{]+\{[^}]*drop-shadow/s);
});

test('hover exposes last update age without changing transparent disk visuals', async () => {
  const css = await readFile(new URL('../src/styles.css', import.meta.url), 'utf8');

  assert.match(css, /\.widget\.is-hovered \.update-badge\s*\{\s*display: flex;/);
  assert.match(css, /\.widget\.stale \.update-badge\s*\{\s*display: flex;/);
});

test('generates exactly forty-eight gauge ticks', () => {
  assert.equal((ticksSvg().match(/<line class="tick"/g) ?? []).length, 48);
});

test('tick marks are visible on light backgrounds', async () => {
  const css = await readFile(new URL('../src/styles.css', import.meta.url), 'utf8');

  assert.match(css, /--tick: rgba\(255, 255, 255, \.46\);/);
  assert.match(css, /\.tick\s*\{\s*stroke: var\(--tick\);\s*stroke-width: 1\.5;/);
});

test('disk edge fades instead of ending with a hard border', async () => {
  const css = await readFile(new URL('../src/styles.css', import.meta.url), 'utf8');

  assert.match(css, /--disk-mask: radial-gradient\(circle,/);
  assert.match(css, /transparent 100%/);
  assert.match(css, /mask-image: var\(--disk-mask\);/);
  assert.match(css, /-webkit-mask-image: var\(--disk-mask\);/);
  assert.match(css, /\.disk\s*\{[^}]*border: 0;/s);
});
