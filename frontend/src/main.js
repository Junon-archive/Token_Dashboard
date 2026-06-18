import {
  formatRemainingMinutes,
  formatResetCountdown,
  renderPomodoroWidget,
  renderUsageDashboard,
  renderUsageWidget,
} from './widget.js';
import {
  applyPomodoroAction,
  createPomodoroState,
  pomodoroSnapshot,
  setPomodoroMinutes,
  tickPomodoro,
  updatePomodoroSettings,
} from './pomodoro.js';
import { dispatchUsageNotifications, usageThresholdEvents } from './notifications.js';

const runtimeWidgetProvider = String(window.__TOKEN_DASHBOARD_WIDGET__ ?? '').toLowerCase();
const singleWidgetProviders = new Set(['claude', 'codex', 'pomodoro']);
const isSingleWidgetRuntime = singleWidgetProviders.has(runtimeWidgetProvider);
const isPomodoroWindow = runtimeWidgetProvider === 'pomodoro';

if (isSingleWidgetRuntime) {
  document.body.classList.add('single-widget-window');
}

const fallbackSnapshots = [{
  provider: 'claude',
  state: 'NORMAL',
  primary: {
    used_pct: 34,
    resets_at: new Date(Date.now() + 197 * 60000).toISOString(),
  },
  secondary: {
    used_pct: 8,
    resets_at: new Date(Date.now() + 4 * 24 * 60 * 60000).toISOString(),
  },
  extra: null,
  fetched_at: new Date().toISOString(),
  is_stale: false,
  error: null,
}, {
  provider: 'codex',
  state: 'WARN',
  primary: {
    used_pct: 82,
    resets_at: new Date(Date.now() + 102 * 60000).toISOString(),
  },
  secondary: {
    used_pct: 47,
    resets_at: new Date(Date.now() + 3 * 24 * 60 * 60000).toISOString(),
  },
  extra: null,
  fetched_at: new Date().toISOString(),
  is_stale: false,
  error: null,
}];

const fallbackAppSettings = {
  polling: {
    interval_sec: 180,
    min_interval_sec: 120,
  },
  notifications: {
    enabled: true,
    thresholds: [80, 95],
  },
  widgets: {
    claude: { enabled: true },
    codex: { enabled: true },
    pomodoro: { enabled: true },
  },
  pomodoro: {
    focus_min: 20,
    break_min: 5,
  },
};

function degradedSnapshots() {
  const fetchedAt = new Date().toISOString();
  const providers = isSingleWidgetRuntime && !isPomodoroWindow
    ? [runtimeWidgetProvider]
    : ['claude', 'codex'];
  return providers.map((provider) => ({
    provider,
    state: 'STALE',
    primary: null,
    secondary: null,
    fetched_at: fetchedAt,
    is_stale: true,
  }));
}

async function loadSnapshots() {
  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    return fallbackSnapshots;
  }
  try {
    if (isPomodoroWindow) {
      return [];
    }
    if (isSingleWidgetRuntime) {
      return [await invoke('usage_snapshot', { provider: runtimeWidgetProvider })];
    }
    return await invoke('usage_snapshots');
  } catch {
    return degradedSnapshots();
  }
}

async function loadAppSettings() {
  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    return fallbackAppSettings;
  }
  try {
    return await invoke('get_app_settings');
  } catch {
    return fallbackAppSettings;
  }
}

function providerPollingMs(settings) {
  const polling = settings?.polling ?? fallbackAppSettings.polling;
  const minIntervalSec = Math.max(120, Number(polling.min_interval_sec) || 120);
  const intervalSec = Math.max(minIntervalSec, Number(polling.interval_sec) || minIntervalSec);
  return intervalSec * 1000;
}

function syncWidgetScale() {
  const scale = Number(appSettings.widget_scale) || 1;
  document.documentElement.style.setProperty('--scale', String(scale));
}

function windowEnabled(provider) {
  return appSettings.widgets?.[provider]?.enabled !== false;
}

function currentDashboardSnapshots() {
  if (isSingleWidgetRuntime) {
    if (isPomodoroWindow) {
      return windowEnabled('pomodoro') ? [pomodoroSnapshot(pomodoro)] : [];
    }
    if (!windowEnabled(runtimeWidgetProvider)) {
      return [];
    }
    return providerSnapshots.filter((snapshot) => (
      String(snapshot.provider).toLowerCase() === runtimeWidgetProvider
    ));
  }
  const snapshots = providerSnapshots.filter((snapshot) => (
    appSettings.widgets?.[String(snapshot.provider ?? '').toLowerCase()]?.enabled !== false
  ));
  if (appSettings.widgets?.pomodoro?.enabled !== false) {
    snapshots.push(pomodoroSnapshot(pomodoro));
  }
  return snapshots;
}

/* [REFACTOR] Keep one stable dashboard root and reconcile widget sections in place so settings changes do not repaint a full-window rectangle. */
function renderWidgetMarkup(snapshot) {
  return String(snapshot.provider).toLowerCase() === 'pomodoro'
    ? renderPomodoroWidget(snapshot)
    : renderUsageWidget(snapshot);
}

function createWidgetElement(snapshot) {
  const template = document.createElement('template');
  template.innerHTML = renderWidgetMarkup(snapshot).trim();
  return template.content.firstElementChild;
}

function ensureDashboardElement() {
  let dashboard = root.querySelector('.dashboard');
  if (dashboard) {
    return dashboard;
  }
  root.innerHTML = renderUsageDashboard(currentDashboardSnapshots());
  dashboard = root.querySelector('.dashboard');
  bindDashboardInteractions();
  return dashboard;
}

function reconcileDashboardStructure() {
  const dashboard = ensureDashboardElement();
  const snapshots = currentDashboardSnapshots();
  const desiredProviders = new Set(snapshots.map((snapshot) => String(snapshot.provider).toLowerCase()));

  for (const widget of dashboard.querySelectorAll('.widget')) {
    if (!desiredProviders.has(String(widget.dataset.provider).toLowerCase())) {
      widget.remove();
    }
  }

  snapshots.forEach((snapshot, index) => {
    const provider = String(snapshot.provider).toLowerCase();
    let widget = dashboard.querySelector(`.widget[data-provider="${provider}"]`);
    if (!widget) {
      widget = createWidgetElement(snapshot);
      bindWidgetInteractions(widget);
    }
    const currentAtIndex = dashboard.children[index] ?? null;
    if (currentAtIndex !== widget) {
      dashboard.insertBefore(widget, currentAtIndex);
    }
  });
}

function mountDashboard() {
  pomodoro = updatePomodoroSettings(pomodoro, {
    focusMin: appSettings.pomodoro?.focus_min,
    breakMin: appSettings.pomodoro?.break_min,
  });
  syncWidgetScale();
  reconcileDashboardStructure();
  collectDashboardRefs();
  updateDashboardTime();
  updatePomodoroWidget();
}

let root = document.querySelector('#app');
let appSettings = await loadAppSettings();
let providerSnapshots = await loadSnapshots();
let pomodoro = createPomodoroState({
  focusMin: appSettings.pomodoro?.focus_min,
  breakMin: appSettings.pomodoro?.break_min,
});
let editingPomodoroMinutes = false;
const sentNotificationKeys = new Set();
let dragState = null;
/* [REFACTOR] Keep live DOM references per widget so HTML span updates do not touch SVG glyph layers. */
const dashboardRefs = {
  widgets: new Map(),
};

function widgetStateClass(snapshot) {
  if (snapshot.provider === 'pomodoro') {
    if (snapshot.state === 'ENDING') {
      return 'ending';
    }
    if (snapshot.state === 'BREAK') {
      return 'break';
    }
    if (snapshot.state === 'PAUSED') {
      return 'paused';
    }
    return 'focus';
  }
  const usedPct = snapshot.primary?.used_pct;
  switch (snapshot.state) {
    case 'NORMAL':
      return '';
    case 'WARN':
      return 'low';
    case 'CRITICAL':
      return usedPct >= 100 ? 'depleted' : 'critical';
    case 'STALE':
    case 'RATE_LIMITED':
      return 'stale';
    case 'NOT_LOGGED_IN':
      return 'notin';
    case 'AUTH_ERROR':
      return 'autherr';
    default:
      return 'stale';
  }
}

function collectDashboardRefs() {
  dashboardRefs.widgets.clear();
  for (const widget of root.querySelectorAll('.widget')) {
    const provider = widget.dataset.provider;
    dashboardRefs.widgets.set(provider, {
      widget,
      arcMain: widget.querySelector('.arc-main'),
      arcSec: widget.querySelector('.arc-sec'),
      number: widget.querySelector('.gauge-label .num'),
      label: widget.querySelector('.gauge-label .name'),
      controls: widget.querySelector('.pomodoro-controls'),
      toggle: widget.querySelector('[data-pomodoro-action="toggle"]'),
      skip: widget.querySelector('[data-pomodoro-action="skip"]'),
    });
  }
}

function setWidgetClass(ref, snapshot) {
  const provider = String(snapshot.provider).toLowerCase();
  const stateClass = widgetStateClass({ ...snapshot, provider });
  const keepHover = ref.widget.classList.contains('is-hovered');
  ref.widget.className = ['widget', provider, stateClass].filter(Boolean).join(' ');
  if (keepHover) {
    ref.widget.classList.add('is-hovered');
  }
  ref.widget.dataset.state = snapshot.state;
}

function setArcProgress(arc, usedPct) {
  if (!arc) {
    return;
  }
  const dashArray = Number.parseFloat(arc.getAttribute('stroke-dasharray') ?? '');
  if (!Number.isFinite(dashArray)) {
    return;
  }
  const clamped = Math.max(0, Math.min(100, usedPct ?? 0));
  arc.setAttribute('stroke-dashoffset', (dashArray * (clamped / 100)).toFixed(2));
}

function updateProviderWidget(snapshot, now = new Date()) {
  const provider = String(snapshot.provider ?? '').toLowerCase();
  const ref = dashboardRefs.widgets.get(provider);
  if (!ref) {
    return;
  }
  setWidgetClass(ref, { ...snapshot, provider });
  setArcProgress(ref.arcMain, snapshot.primary?.used_pct);
  setArcProgress(ref.arcSec, snapshot.secondary?.used_pct);
  if (ref.number) {
    ref.number.textContent = formatResetCountdown(snapshot.primary?.resets_at, now);
  }
  /* [REFACTOR] Force a layout flush after text/arc mutation so stale transparent WebKit damage is cleared immediately. */
  void ref.widget.offsetHeight;
}

function updatePomodoroWidget(now = new Date()) {
  pomodoro = tickPomodoro(pomodoro, now);
  const snapshot = pomodoroSnapshot(pomodoro, now);
  const ref = dashboardRefs.widgets.get('pomodoro');
  if (!ref) {
    return;
  }
  setWidgetClass(ref, snapshot);
  setArcProgress(ref.arcMain, snapshot.primary?.used_pct);
  if (ref.number && !editingPomodoroMinutes) {
    ref.number.textContent = formatRemainingMinutes(snapshot.primary?.resets_at, now);
  }
  if (ref.label) {
    ref.label.textContent = snapshot.phase === 'BREAK' ? 'Break' : 'Focus';
  }
  if (ref.toggle) {
    ref.toggle.classList.toggle('is-running', snapshot.state === 'FOCUS' || snapshot.state === 'BREAK');
    if (ref.toggle.textContent !== snapshot.action_label) {
      ref.toggle.textContent = snapshot.action_label;
    }
    ref.toggle.setAttribute('aria-label', `${snapshot.action_label} timer`);
  }
  if (ref.controls) {
    void ref.controls.offsetHeight;
  }
  if (ref.skip) {
    ref.skip.setAttribute('aria-label', snapshot.phase === 'BREAK' ? 'Start focus' : 'Start break');
  }
  /* [REFACTOR] Force a layout flush after text/arc mutation so stale transparent WebKit damage is cleared immediately. */
  void ref.widget.offsetHeight;
}

function renderInitialDashboard() {
  pomodoro = tickPomodoro(pomodoro);
  mountDashboard();
}

function handlePomodoroAction(action) {
  pomodoro = applyPomodoroAction(pomodoro, action);
  updatePomodoroWidget();
}

function clearHover(widget, event) {
  if (!widget) {
    return;
  }
  const relatedTarget = event?.relatedTarget;
  if (relatedTarget && widget.contains(relatedTarget)) {
    return;
  }
  widget.classList.remove('is-hovered');
  if (widget?.classList.contains('pomodoro')) {
    const controls = widget.querySelector('.pomodoro-controls');
    if (controls) {
      void controls.offsetHeight;
    }
  }
}

function hoverBoundsForWidget(widget) {
  if (!widget?.classList.contains('pomodoro')) {
    return [widget?.getBoundingClientRect?.()].filter(Boolean);
  }
  const bounds = [widget.getBoundingClientRect()];
  const controls = widget.querySelector('.pomodoro-controls');
  if (controls) {
    const rect = controls.getBoundingClientRect();
    bounds.push(rect);
    bounds.push({
      left: Math.min(bounds[0].left, rect.left) - 4,
      right: Math.max(bounds[0].right, rect.right) + 4,
      top: Math.min(bounds[0].top, rect.top) - 4,
      bottom: Math.max(bounds[0].bottom, rect.bottom) + 4,
    });
  }
  return bounds;
}

function updateDashboardTime() {
  const now = new Date();
  for (const snapshot of providerSnapshots) {
    updateProviderWidget(snapshot, now);
  }
}

async function evaluateUsageNotifications(snapshots) {
  const events = usageThresholdEvents(snapshots, appSettings, sentNotificationKeys);
  await dispatchUsageNotifications(events, sentNotificationKeys);
}

async function reloadProviderSnapshots() {
  if (isPomodoroWindow) {
    return;
  }
  providerSnapshots = await loadSnapshots();
  updateDashboardTime();
  if (!isSingleWidgetRuntime) {
    await evaluateUsageNotifications(providerSnapshots);
  }
}

function settingsSignature(settings) {
  return JSON.stringify({
    widgets: settings.widgets,
    widget_scale: settings.widget_scale,
    pomodoro: settings.pomodoro,
  });
}

let renderedSettingsSignature = settingsSignature(appSettings);

async function reloadAppSettings() {
  const nextSettings = await loadAppSettings();
  const previousSignature = renderedSettingsSignature;
  appSettings = nextSettings;
  renderedSettingsSignature = settingsSignature(appSettings);
  if (renderedSettingsSignature !== previousSignature) {
    mountDashboard();
  }
}

function updatePomodoroTime() {
  updatePomodoroWidget(new Date());
}

function finishPomodoroMinuteEdit(input, commit) {
  if (!editingPomodoroMinutes) {
    return;
  }
  editingPomodoroMinutes = false;
  if (commit) {
    pomodoro = setPomodoroMinutes(pomodoro, input.value);
  }
  const snapshot = pomodoroSnapshot(pomodoro);
  const ref = dashboardRefs.widgets.get('pomodoro');
  const next = createPomodoroMinuteSpan(formatRemainingMinutes(snapshot.primary?.resets_at));
  input.replaceWith(next);
  if (ref) {
    ref.number = next;
  }
  updatePomodoroWidget();
}

/* [REFACTOR] Keep Pomodoro minute editing on HTML spans so the editable glyph is detached from SVG repaint timing. */
function createPomodoroMinuteSpan(text) {
  const span = document.createElement('span');
  span.className = 'num pomodoro-display';
  span.dataset.pomodoroEdit = 'minutes';
  span.dataset.noDrag = 'true';
  span.tabIndex = 0;
  span.setAttribute('role', 'button');
  span.setAttribute('aria-label', 'Set Pomodoro minutes');
  span.textContent = text;
  return span;
}

function beginPomodoroMinuteEdit(number) {
  if (editingPomodoroMinutes) {
    return;
  }
  editingPomodoroMinutes = true;
  const input = document.createElement('input');
  input.className = 'num pomodoro-minute-input';
  input.type = 'number';
  input.inputMode = 'numeric';
  input.min = '1';
  input.max = '180';
  input.step = '1';
  input.value = number.textContent.trim();
  input.setAttribute('aria-label', 'Pomodoro minutes');
  input.dataset.noDrag = 'true';

  const ref = dashboardRefs.widgets.get('pomodoro');
  number.replaceWith(input);
  if (ref) {
    ref.number = input;
  }
  input.focus();
  input.select();

  input.addEventListener('blur', () => finishPomodoroMinuteEdit(input, true), { once: true });
  input.addEventListener('keydown', (event) => {
    if (event.key === 'Enter') {
      event.preventDefault();
      finishPomodoroMinuteEdit(input, true);
    }
    if (event.key === 'Escape') {
      event.preventDefault();
      finishPomodoroMinuteEdit(input, false);
    }
  });
}

function removeDragListeners() {
  window.removeEventListener('pointermove', onWidgetDragMove, true);
  window.removeEventListener('pointerup', onWidgetDragEnd, true);
  window.removeEventListener('pointercancel', onWidgetDragCancel, true);
}

async function invokeWidgetMove(provider, x, y, persist) {
  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    return appSettings;
  }
  return invoke('move_widget_windows', { provider, x, y, persist });
}

function syncDraggedSettings(nextSettings) {
  if (!nextSettings) {
    return;
  }
  appSettings = nextSettings;
  renderedSettingsSignature = settingsSignature(appSettings);
}

function dragTargetPosition(state) {
  return {
    x: state.originX + Math.round(state.dx),
    y: state.originY + Math.round(state.dy),
  };
}

async function flushWidgetDrag(persist = false) {
  if (!dragState || dragState.inFlight) {
    return;
  }
  const state = dragState;
  const { x, y } = dragTargetPosition(state);
  state.inFlight = true;
  state.needsFlush = false;
  try {
    const nextSettings = await invokeWidgetMove(state.provider, x, y, persist);
    if (dragState === state) {
      syncDraggedSettings(nextSettings);
    }
  } finally {
    if (dragState !== state) {
      return;
    }
    state.inFlight = false;
    if (state.finalizing) {
      if (persist) {
        dragState = null;
        removeDragListeners();
        return;
      }
      void flushWidgetDrag(true);
      return;
    }
    if (state.needsFlush) {
      state.frame = requestAnimationFrame(() => {
        state.frame = 0;
        void flushWidgetDrag(false);
      });
    }
  }
}

function scheduleWidgetDragFlush() {
  if (!dragState) {
    return;
  }
  dragState.needsFlush = true;
  if (dragState.frame || dragState.inFlight) {
    return;
  }
  dragState.frame = requestAnimationFrame(() => {
    if (!dragState) {
      return;
    }
    dragState.frame = 0;
    void flushWidgetDrag(false);
  });
}

function beginWidgetDrag(widget, event) {
  const provider = String(widget?.dataset?.provider ?? '').toLowerCase();
  const origin = appSettings.widgets?.[provider]?.position;
  if (!provider || !origin) {
    return;
  }
  if (dragState?.frame) {
    cancelAnimationFrame(dragState.frame);
  }
  dragState = {
    provider,
    pointerId: event.pointerId,
    originScreenX: event.screenX,
    originScreenY: event.screenY,
    originX: Number(origin.x) || 0,
    originY: Number(origin.y) || 0,
    dx: 0,
    dy: 0,
    frame: 0,
    inFlight: false,
    needsFlush: false,
    finalizing: false,
  };
  widget.setPointerCapture?.(event.pointerId);
  window.addEventListener('pointermove', onWidgetDragMove, true);
  window.addEventListener('pointerup', onWidgetDragEnd, true);
  window.addEventListener('pointercancel', onWidgetDragCancel, true);
}

function onWidgetDragMove(event) {
  if (!dragState || event.pointerId !== dragState.pointerId) {
    return;
  }
  dragState.dx = event.screenX - dragState.originScreenX;
  dragState.dy = event.screenY - dragState.originScreenY;
  scheduleWidgetDragFlush();
}

function onWidgetDragEnd(event) {
  if (!dragState || event.pointerId !== dragState.pointerId) {
    return;
  }
  dragState.dx = event.screenX - dragState.originScreenX;
  dragState.dy = event.screenY - dragState.originScreenY;
  dragState.finalizing = true;
  if (dragState.frame) {
    cancelAnimationFrame(dragState.frame);
    dragState.frame = 0;
  }
  if (dragState.inFlight) {
    return;
  }
  void flushWidgetDrag(true);
}

function onWidgetDragCancel(event) {
  if (!dragState || event.pointerId !== dragState.pointerId) {
    return;
  }
  if (dragState.frame) {
    cancelAnimationFrame(dragState.frame);
  }
  dragState = null;
  removeDragListeners();
}

function syncHover(widget, event) {
  if (!widget) {
    return;
  }
  const isInside = hoverBoundsForWidget(widget).some((rect) => (
    event.clientX >= rect.left &&
    event.clientX <= rect.right &&
    event.clientY >= rect.top &&
    event.clientY <= rect.bottom
  ));
  widget.classList.toggle('is-hovered', isInside);
}

function bindWidgetInteractions(scope = root) {
  const widgets = scope.matches?.('.widget') ? [scope] : scope.querySelectorAll('.widget');
  for (const widget of widgets) {
    widget.addEventListener('mouseenter', () => widget.classList.add('is-hovered'));
    widget.addEventListener('mouseleave', (event) => clearHover(widget, event));
    widget.addEventListener('pointerenter', () => widget.classList.add('is-hovered'));
    widget.addEventListener('pointerleave', (event) => clearHover(widget, event));

    widget.addEventListener(
      'pointerdown',
      (event) => {
        if (event.target.closest('[data-pomodoro-edit="minutes"]')) {
          event.preventDefault();
          return;
        }
        if (event.target.closest('[data-no-drag="true"]')) {
          return;
        }
        if (event.button !== 0 || event.detail !== 1) {
          return;
        }
        event.preventDefault();
        beginWidgetDrag(widget, event);
      },
      { capture: true },
    );
  }
}

function bindDashboardInteractions(scope = root) {
  bindWidgetInteractions(scope);

  scope.addEventListener('click', (event) => {
    const actionButton = event.target.closest('[data-pomodoro-action]');
    if (actionButton) {
      event.preventDefault();
      event.stopPropagation();
      handlePomodoroAction(actionButton.dataset.pomodoroAction);
      return;
    }

    const pomodoroWidget = event.target.closest('.widget.pomodoro');
    if (pomodoroWidget && pomodoro.state === 'ENDING' && !event.target.closest('.pomodoro-controls')) {
      event.preventDefault();
      handlePomodoroAction('acknowledge');
      return;
    }

    const edit = event.target.closest('[data-pomodoro-edit="minutes"]');
    if (edit) {
      event.preventDefault();
      beginPomodoroMinuteEdit(edit);
      return;
    }
  });

  scope.addEventListener('keydown', (event) => {
    const edit = event.target.closest('[data-pomodoro-edit="minutes"]');
    if (!edit) {
      return;
    }
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      beginPomodoroMinuteEdit(edit);
    }
  });

  scope.addEventListener('contextmenu', (event) => {
    event.preventDefault();
    window.__TAURI__?.core?.invoke?.('open_settings_window');
  });
}

function listenForSettingsUpdates() {
  const listen = window.__TAURI__?.event?.listen;
  if (!listen) {
    return;
  }
  listen('app-settings-updated', (event) => {
    const nextSettings = event?.payload;
    if (!nextSettings) {
      return;
    }
    const previousSignature = renderedSettingsSignature;
    appSettings = nextSettings;
    renderedSettingsSignature = settingsSignature(appSettings);
    if (renderedSettingsSignature !== previousSignature) {
      mountDashboard();
    }
  });
}

window.addEventListener('pointermove', (event) => {
  for (const widget of root.querySelectorAll('.widget')) {
    syncHover(widget, event);
  }
});
window.addEventListener('mouseleave', () => root.querySelectorAll('.widget').forEach(clearHover));
document.addEventListener('mouseleave', () => root.querySelectorAll('.widget').forEach(clearHover));
window.addEventListener('blur', () => root.querySelectorAll('.widget').forEach(clearHover));
document.addEventListener('mouseout', (event) => {
  if (!event.relatedTarget) {
    root.querySelectorAll('.widget').forEach(clearHover);
  }
});

listenForSettingsUpdates();
renderInitialDashboard();
if (!isSingleWidgetRuntime) {
  evaluateUsageNotifications(providerSnapshots);
}
if (!isPomodoroWindow) {
  setInterval(reloadProviderSnapshots, providerPollingMs(appSettings));
  setInterval(updateDashboardTime, 60000);
}
if (!window.__TAURI__?.event?.listen) {
  setInterval(reloadAppSettings, 1000);
}
if (currentDashboardSnapshots().some((snapshot) => String(snapshot.provider).toLowerCase() === 'pomodoro')) {
  setInterval(updatePomodoroTime, 250);
}
