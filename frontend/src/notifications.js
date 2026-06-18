const DEFAULT_THRESHOLDS = [80, 95];

function providerLabel(provider) {
  if (provider === 'codex') {
    return 'Codex';
  }
  if (provider === 'claude') {
    return 'Claude';
  }
  return String(provider ?? 'Provider');
}

function isAlertableState(state) {
  return state === 'NORMAL' || state === 'WARN' || state === 'CRITICAL';
}

function usageWindows(snapshot) {
  return [
    ['primary', snapshot.primary],
    ['secondary', snapshot.secondary],
  ].filter(([, window]) => (
    window && Number.isFinite(Number(window.used_pct)) && Number(window.used_pct) >= 0 && window.resets_at
  ));
}

export function normalizeNotificationSettings(settings = {}) {
  const notifications = settings.notifications ?? {};
  const thresholds = Array.isArray(notifications.thresholds)
    ? notifications.thresholds.map(Number).filter((value) => value >= 1 && value <= 100)
    : DEFAULT_THRESHOLDS;
  const normalizedThresholds = thresholds.length > 0 ? [...new Set(thresholds)].sort((a, b) => a - b) : DEFAULT_THRESHOLDS;
  return {
    enabled: notifications.enabled !== false,
    thresholds: normalizedThresholds,
  };
}

export function usageThresholdEvents(snapshots, settings, sentKeys = new Set()) {
  const notificationSettings = normalizeNotificationSettings(settings);
  if (!notificationSettings.enabled) {
    return [];
  }

  const events = [];
  for (const snapshot of snapshots ?? []) {
    const provider = String(snapshot.provider ?? '').toLowerCase();
    if (provider !== 'claude' && provider !== 'codex') {
      continue;
    }
    if (!isAlertableState(snapshot.state)) {
      continue;
    }

    const windows = usageWindows(snapshot);
    if (windows.length === 0) {
      continue;
    }

    const [windowKind, highestWindow] = windows.reduce((highest, current) => (
      Number(current[1].used_pct) > Number(highest[1].used_pct) ? current : highest
    ));
    const usedPct = Number(highestWindow.used_pct);
    const threshold = notificationSettings.thresholds.filter((value) => usedPct >= value).at(-1);
    if (!threshold) {
      continue;
    }
    const key = `${provider}:${windowKind}:${threshold}:${highestWindow.resets_at}`;
    if (sentKeys.has(key)) {
      continue;
    }
    events.push({
      key,
      provider,
      threshold,
      window: windowKind,
      title: `${providerLabel(provider)} usage ${threshold}%`,
      body: `${Math.round(usedPct)}% used. Resets ${new Date(highestWindow.resets_at).toLocaleString()}.`,
    });
  }
  return events;
}

export async function dispatchUsageNotifications(events, sentKeys, targetWindow = window) {
  const NotificationApi = targetWindow.Notification;
  if (!NotificationApi) {
    return;
  }

  let permission = NotificationApi.permission;
  if (permission === 'default' && typeof NotificationApi.requestPermission === 'function') {
    permission = await NotificationApi.requestPermission();
  }
  if (permission !== 'granted') {
    return;
  }

  for (const event of events) {
    new NotificationApi(event.title, { body: event.body });
    sentKeys.add(event.key);
  }
}
