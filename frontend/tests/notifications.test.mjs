import assert from 'node:assert/strict';
import test from 'node:test';
import {
  dispatchUsageNotifications,
  normalizeNotificationSettings,
  usageThresholdEvents,
} from '../src/notifications.js';

function snapshot(provider, usedPct, overrides = {}) {
  return {
    provider,
    state: 'NORMAL',
    primary: {
      used_pct: usedPct,
      resets_at: '2026-06-16T04:00:00.000Z',
    },
    secondary: null,
    fetched_at: '2026-06-16T01:00:00.000Z',
    is_stale: false,
    ...overrides,
  };
}

test('normalizes notification settings with safe defaults', () => {
  assert.deepEqual(normalizeNotificationSettings({}), { enabled: true, thresholds: [80, 95] });
  assert.deepEqual(
    normalizeNotificationSettings({ notifications: { enabled: true, thresholds: [95, 0, 80, 95, 101] } }),
    { enabled: true, thresholds: [80, 95] },
  );
  assert.deepEqual(
    normalizeNotificationSettings({ notifications: { enabled: false, thresholds: [80] } }),
    { enabled: false, thresholds: [80] },
  );
});

test('emits threshold events once per provider window reset', () => {
  const sent = new Set();
  let events = usageThresholdEvents([snapshot('claude', 79.999)], {}, sent);
  assert.equal(events.length, 0);

  events = usageThresholdEvents([snapshot('claude', 80)], {}, sent);
  assert.equal(events.length, 1);
  assert.equal(events[0].threshold, 80);
  sent.add(events[0].key);

  assert.equal(usageThresholdEvents([snapshot('claude', 90)], {}, sent).length, 0);

  events = usageThresholdEvents([snapshot('claude', 95)], {}, sent);
  assert.equal(events.length, 1);
  assert.equal(events[0].threshold, 95);
  sent.add(events[0].key);

  assert.equal(usageThresholdEvents([snapshot('claude', 120)], {}, sent).length, 0);
});

test('first snapshot above multiple thresholds emits only the highest reached threshold', () => {
  const events = usageThresholdEvents([snapshot('codex', 96)], {}, new Set());

  assert.equal(events.length, 1);
  assert.equal(events[0].provider, 'codex');
  assert.equal(events[0].threshold, 95);
});

test('providers and reset windows rearm independently', () => {
  const sent = new Set();
  const first = usageThresholdEvents([
    snapshot('claude', 82),
    snapshot('codex', 82),
  ], {}, sent);
  assert.equal(first.length, 2);
  first.forEach((event) => sent.add(event.key));

  const second = usageThresholdEvents([
    snapshot('claude', 82),
    snapshot('codex', 82, { primary: { used_pct: 82, resets_at: '2026-06-16T09:00:00.000Z' } }),
  ], {}, sent);
  assert.equal(second.length, 1);
  assert.equal(second[0].provider, 'codex');
});

test('secondary usage can trigger notifications and degraded snapshots do not', () => {
  const secondary = usageThresholdEvents([
    snapshot('codex', 20, {
      secondary: {
        used_pct: 80,
        resets_at: '2026-06-20T00:00:00.000Z',
      },
    }),
  ], {}, new Set());
  assert.equal(secondary.length, 1);
  assert.equal(secondary[0].window, 'secondary');

  for (const state of ['STALE', 'RATE_LIMITED', 'NOT_LOGGED_IN', 'AUTH_ERROR']) {
    assert.equal(usageThresholdEvents([snapshot('claude', 95, { state })], {}, new Set()).length, 0);
  }
  assert.equal(usageThresholdEvents([snapshot('claude', Number.NaN)], {}, new Set()).length, 0);
  assert.equal(usageThresholdEvents([{ provider: 'claude', state: 'NORMAL', primary: null, secondary: null }], {}, new Set()).length, 0);
});

test('dispatch uses a fake Notification API without OS dependencies', async () => {
  const sentKeys = new Set();
  const calls = [];
  function FakeNotification(title, options) {
    calls.push([title, options]);
  }
  FakeNotification.permission = 'granted';

  await dispatchUsageNotifications([
    { key: 'claude:primary:80:reset', title: 'Claude usage 80%', body: '80% used' },
  ], sentKeys, { Notification: FakeNotification });

  assert.deepEqual(calls, [['Claude usage 80%', { body: '80% used' }]]);
  assert(sentKeys.has('claude:primary:80:reset'));
});
