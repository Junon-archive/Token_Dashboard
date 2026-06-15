import { formatRemainingMinutes, formatResetCountdown, renderUsageDashboard } from './widget.js';
import {
  applyPomodoroAction,
  createPomodoroState,
  pomodoroSnapshot,
  setPomodoroMinutes,
  tickPomodoro,
} from './pomodoro.js';

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

function degradedSnapshots() {
  const fetchedAt = new Date().toISOString();
  return ['claude', 'codex'].map((provider) => ({
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
    return await invoke('usage_snapshots');
  } catch {
    return degradedSnapshots();
  }
}

const root = document.querySelector('#app');
let providerSnapshots = await loadSnapshots();
let pomodoro = createPomodoroState();
let editingPomodoroMinutes = false;
/* [REFACTOR] Keep live DOM references per widget so HTML span updates do not touch SVG glyph layers. */
const dashboardRefs = {
  widgets: new Map(),
};

function widgetStateClass(snapshot) {
  if (snapshot.provider === 'pomodoro') {
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
}

function updatePomodoroWidget(now = new Date()) {
  pomodoro = tickPomodoro(pomodoro);
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
    ref.label.textContent = snapshot.state === 'BREAK' ? 'Break' : snapshot.state === 'PAUSED' ? 'Paused' : 'Focus';
  }
  if (ref.toggle) {
    ref.toggle.classList.toggle('is-running', snapshot.state !== 'PAUSED');
    ref.toggle.setAttribute('aria-label', `${snapshot.action_label} timer`);
  }
  if (ref.skip) {
    ref.skip.setAttribute('aria-label', snapshot.phase === 'BREAK' ? 'Start focus' : 'Start break');
  }
}

function renderInitialDashboard() {
  pomodoro = tickPomodoro(pomodoro);
  root.innerHTML = renderUsageDashboard([...providerSnapshots, pomodoroSnapshot(pomodoro)]);
  collectDashboardRefs();
  bindWidgetInteractions();
}

function handlePomodoroAction(action) {
  pomodoro = applyPomodoroAction(pomodoro, action);
  updatePomodoroWidget();
}

function clearHover(widget) {
  widget?.classList.remove('is-hovered');
  if (widget?.classList.contains('pomodoro')) {
    const controls = widget.querySelector('.pomodoro-controls');
    if (controls) {
      void controls.offsetHeight;
    }
  }
}

function updateDashboardTime() {
  const now = new Date();
  for (const snapshot of providerSnapshots) {
    updateProviderWidget(snapshot, now);
  }
  if (!editingPomodoroMinutes) {
    updatePomodoroWidget(now);
  }
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

function syncHover(widget, event) {
  if (!widget) {
    return;
  }
  const rect = widget.getBoundingClientRect();
  const isInside =
    event.clientX >= rect.left &&
    event.clientX <= rect.right &&
    event.clientY >= rect.top &&
    event.clientY <= rect.bottom;
  widget.classList.toggle('is-hovered', isInside);
}

function bindWidgetInteractions(scope = root) {
  const widgets = scope.matches?.('.widget') ? [scope] : scope.querySelectorAll('.widget');
  for (const widget of widgets) {
    widget.addEventListener('mouseenter', () => widget.classList.add('is-hovered'));
    widget.addEventListener('mouseleave', () => clearHover(widget));
    widget.addEventListener('pointerenter', () => widget.classList.add('is-hovered'));
    widget.addEventListener('pointerleave', () => clearHover(widget));

    widget.addEventListener(
      'mousedown',
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
        window.__TAURI__?.window?.getCurrentWindow?.().startDragging?.();
      },
      { capture: true },
    );
  }
}

root.addEventListener('click', (event) => {
  const edit = event.target.closest('[data-pomodoro-edit="minutes"]');
  if (edit) {
    event.preventDefault();
    beginPomodoroMinuteEdit(edit);
    return;
  }

  const control = event.target.closest('[data-pomodoro-action]');
  if (!control) {
    return;
  }
  event.preventDefault();
  handlePomodoroAction(control.dataset.pomodoroAction);
});

root.addEventListener('keydown', (event) => {
  const edit = event.target.closest('[data-pomodoro-edit="minutes"]');
  if (!edit) {
    return;
  }
  if (event.key === 'Enter' || event.key === ' ') {
    event.preventDefault();
    beginPomodoroMinuteEdit(edit);
  }
});

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

renderInitialDashboard();
setInterval(updateDashboardTime, 60000);
