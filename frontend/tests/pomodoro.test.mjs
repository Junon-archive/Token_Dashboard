import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import {
  applyPomodoroAction,
  createPomodoroState,
  pomodoroSnapshot,
  setPomodoroMinutes,
  tickPomodoro,
  updatePomodoroSettings,
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

test('resume keeps the gauge progress from the paused point', () => {
  let state = createPomodoroState({ now: START });
  state = applyPomodoroAction(state, 'toggle', new Date('2026-06-12T00:08:00.000Z'));
  const paused = pomodoroSnapshot(state, new Date('2026-06-12T00:08:00.000Z'));

  state = applyPomodoroAction(state, 'toggle', new Date('2026-06-12T00:10:00.000Z'));
  const resumed = pomodoroSnapshot(state, new Date('2026-06-12T00:10:00.000Z'));
  const later = pomodoroSnapshot(state, new Date('2026-06-12T00:12:00.000Z'));

  assert.equal(paused.primary.used_pct, 40);
  assert.equal(resumed.primary.used_pct, 40);
  assert.equal(formatRemainingMinutes(resumed.primary.resets_at, new Date('2026-06-12T00:10:00.000Z')), '12');
  assert.equal(later.primary.used_pct, 50);
  assert.equal(formatRemainingMinutes(later.primary.resets_at, new Date('2026-06-12T00:12:00.000Z')), '10');
});

test('gauge scales against the configured phase duration', () => {
  let state = createPomodoroState({ now: START, focusMin: 1 });
  state = tickPomodoro(state, new Date('2026-06-12T00:00:30.000Z'));
  const snapshot = pomodoroSnapshot(state, new Date('2026-06-12T00:00:30.000Z'));

  assert.equal(snapshot.primary.used_pct, 50);
  assert.equal(formatRemainingMinutes(snapshot.primary.resets_at, new Date('2026-06-12T00:00:30.000Z')), '1');
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
  assert.match(html, /<span class="name">Break<\/span>/);
});

test('skip advances break to focus', () => {
  let state = createPomodoroState({ now: START });
  state = applyPomodoroAction(state, 'skip', START);
  state = applyPomodoroAction(state, 'skip', START);
  const snapshot = pomodoroSnapshot(state, START);

  assert.equal(snapshot.state, 'FOCUS');
  assert.equal(formatRemainingMinutes(snapshot.primary.resets_at, START), '20');
});

test('timer enters a blinking ending state before advancing to the next paused phase', () => {
  let state = createPomodoroState({ now: START });
  state = tickPomodoro(state, new Date('2026-06-12T00:20:00.001Z'));
  assert.equal(state.phase, 'FOCUS');
  assert.equal(state.isEnding, true);
  assert.equal(state.isRunning, false);

  state = tickPomodoro(state, new Date('2026-06-12T00:20:30.002Z'));
  assert.equal(state.phase, 'BREAK');
  assert.equal(state.isEnding, false);
  assert.equal(state.isRunning, false);
  assert.equal(state.remainingMs, state.durationMs);
});

test('ending state can be acknowledged before the blink timeout finishes', () => {
  let state = createPomodoroState({ now: START });
  state = tickPomodoro(state, new Date('2026-06-12T00:20:00.001Z'));
  state = applyPomodoroAction(state, 'acknowledge', new Date('2026-06-12T00:20:10.000Z'));

  assert.equal(state.phase, 'BREAK');
  assert.equal(state.isEnding, false);
  assert.equal(state.isRunning, false);
  assert.equal(state.remainingMs, state.durationMs);
});

test('setting minutes updates the current phase duration and pauses at full time', () => {
  let state = createPomodoroState({ now: START });
  state = tickPomodoro(state, new Date('2026-06-12T00:07:00.000Z'));
  state = setPomodoroMinutes(state, 35, new Date('2026-06-12T00:07:00.000Z'));
  const snapshot = pomodoroSnapshot(state, new Date('2026-06-12T00:07:00.000Z'));

  assert.equal(state.settings.focusMin, 35);
  assert.equal(snapshot.state, 'PAUSED');
  assert.equal(snapshot.phase, 'FOCUS');
  assert.equal(snapshot.primary.used_pct, 0);
  assert.equal(formatRemainingMinutes(snapshot.primary.resets_at, new Date('2026-06-12T00:07:00.000Z')), '35');
});

test('setting minutes clamps to a practical local-only range', () => {
  let state = createPomodoroState({ now: START });
  state = setPomodoroMinutes(state, 0, START);
  assert.equal(state.settings.focusMin, 1);

  state = setPomodoroMinutes(state, 999, START);
  assert.equal(state.settings.focusMin, 180);
});

test('settings update applies configured phase durations to the active timer', () => {
  let state = createPomodoroState({ now: START, focusMin: 20, breakMin: 5 });
  state = updatePomodoroSettings(state, { focusMin: 10, breakMin: 3 }, START);
  let snapshot = pomodoroSnapshot(state, START);

  assert.equal(state.settings.focusMin, 10);
  assert.equal(state.settings.breakMin, 3);
  assert.equal(formatRemainingMinutes(snapshot.primary.resets_at, START), '10');

  state = applyPomodoroAction(state, 'skip', START);
  snapshot = pomodoroSnapshot(state, START);
  assert.equal(snapshot.phase, 'BREAK');
  assert.equal(formatRemainingMinutes(snapshot.primary.resets_at, START), '3');
});

test('pomodoro module has no provider or network dependencies', async () => {
  const source = await readFile(new URL('../src/pomodoro.js', import.meta.url), 'utf8');

  assert.doesNotMatch(source, /__TAURI__/);
  assert.doesNotMatch(source, /\binvoke\s*\(/);
  assert.doesNotMatch(source, /\bfetch\s*\(/);
  assert.doesNotMatch(source, /usage_snapshots/);
});
