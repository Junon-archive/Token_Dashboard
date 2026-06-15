import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import {
  applyPomodoroAction,
  createPomodoroState,
  pomodoroSnapshot,
  tickPomodoro,
} from '../src/pomodoro.js';
import { formatRemainingMinutes, renderPomodoroWidget } from '../src/widget.js';

const START = new Date('2026-06-12T00:00:00.000Z');

test('creates default focus snapshot', () => {
  const state = createPomodoroState({ now: START });
  const snapshot = pomodoroSnapshot(state, START);

  assert.equal(snapshot.provider, 'pomodoro');
  assert.equal(snapshot.state, 'FOCUS');
  assert.equal(snapshot.primary.used_pct, 0);
  assert.equal(snapshot.primary.resets_at, '2026-06-12T00:20:00.000Z');
  assert.equal(snapshot.secondary, null);
  assert.equal(snapshot.is_stale, false);
});

test('pause freezes remaining time', () => {
  let state = createPomodoroState({ now: START });
  state = applyPomodoroAction(state, 'toggle', new Date('2026-06-12T00:05:00.000Z'));
  const snapshot = pomodoroSnapshot(state, new Date('2026-06-12T00:10:00.000Z'));

  assert.equal(snapshot.state, 'PAUSED');
  assert.equal(formatRemainingMinutes(snapshot.primary.resets_at, new Date('2026-06-12T00:10:00.000Z')), '15');
});

test('resume preserves remaining duration', () => {
  let state = createPomodoroState({ now: START });
  state = applyPomodoroAction(state, 'toggle', new Date('2026-06-12T00:05:00.000Z'));
  state = applyPomodoroAction(state, 'toggle', new Date('2026-06-12T00:10:00.000Z'));
  const snapshot = pomodoroSnapshot(state, new Date('2026-06-12T00:12:00.000Z'));

  assert.equal(snapshot.state, 'FOCUS');
  assert.equal(formatRemainingMinutes(snapshot.primary.resets_at, new Date('2026-06-12T00:12:00.000Z')), '13');
});

test('reset returns current phase to full paused duration', () => {
  let state = createPomodoroState({ now: START });
  state = tickPomodoro(state, new Date('2026-06-12T00:07:00.000Z'));
  state = applyPomodoroAction(state, 'reset', new Date('2026-06-12T00:07:00.000Z'));
  const snapshot = pomodoroSnapshot(state, new Date('2026-06-12T00:07:00.000Z'));

  assert.equal(snapshot.state, 'PAUSED');
  assert.equal(snapshot.phase, 'FOCUS');
  assert.equal(snapshot.primary.used_pct, 0);
  assert.equal(formatRemainingMinutes(snapshot.primary.resets_at, new Date('2026-06-12T00:07:00.000Z')), '20');
});

test('skip advances focus to break and renders break widget', () => {
  let state = createPomodoroState({ now: START });
  state = applyPomodoroAction(state, 'skip', START);
  const snapshot = pomodoroSnapshot(state, START);
  const html = renderPomodoroWidget(snapshot, { now: START });

  assert.equal(snapshot.state, 'BREAK');
  assert.equal(formatRemainingMinutes(snapshot.primary.resets_at, START), '5');
  assert.match(html, /class="widget pomodoro break"/);
  assert.match(html, /<div class="lbl">Break<\/div>/);
});

test('skip advances break to focus', () => {
  let state = createPomodoroState({ now: START });
  state = applyPomodoroAction(state, 'skip', START);
  state = applyPomodoroAction(state, 'skip', START);
  const snapshot = pomodoroSnapshot(state, START);

  assert.equal(snapshot.state, 'FOCUS');
  assert.equal(formatRemainingMinutes(snapshot.primary.resets_at, START), '20');
});

test('timer auto-advances phases on tick', () => {
  let state = createPomodoroState({ now: START });
  state = tickPomodoro(state, new Date('2026-06-12T00:20:00.001Z'));
  assert.equal(state.phase, 'BREAK');
  state = tickPomodoro(state, new Date('2026-06-12T00:25:00.002Z'));
  assert.equal(state.phase, 'FOCUS');
});

test('pomodoro module has no provider or network dependencies', async () => {
  const source = await readFile(new URL('../src/pomodoro.js', import.meta.url), 'utf8');

  assert.doesNotMatch(source, /__TAURI__/);
  assert.doesNotMatch(source, /\binvoke\s*\(/);
  assert.doesNotMatch(source, /\bfetch\s*\(/);
  assert.doesNotMatch(source, /usage_snapshots/);
});
