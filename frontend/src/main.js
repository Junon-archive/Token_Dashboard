import { renderUsageDashboard } from './widget.js';
import { applyPomodoroAction, createPomodoroState, pomodoroSnapshot, tickPomodoro } from './pomodoro.js';

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

function renderDashboard() {
  pomodoro = tickPomodoro(pomodoro);
  root.innerHTML = renderUsageDashboard([...providerSnapshots, pomodoroSnapshot(pomodoro)]);
  bindWidgetInteractions();
}

function renderPomodoroOnly() {
  pomodoro = tickPomodoro(pomodoro);
  const current = root.querySelector('[data-provider="pomodoro"]');
  if (!current) {
    renderDashboard();
    return;
  }
  const template = document.createElement('template');
  template.innerHTML = renderUsageDashboard([pomodoroSnapshot(pomodoro)]);
  const next = template.content.querySelector('[data-provider="pomodoro"]');
  current.replaceWith(next);
  bindWidgetInteractions(next);
}

function handlePomodoroAction(action) {
  pomodoro = applyPomodoroAction(pomodoro, action);
  renderPomodoroOnly();
}

function clearHover(widget) {
  widget?.classList.remove('is-hovered');
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
  const control = event.target.closest('[data-pomodoro-action]');
  if (!control) {
    return;
  }
  event.preventDefault();
  handlePomodoroAction(control.dataset.pomodoroAction);
});

window.addEventListener('mousemove', (event) => {
  for (const widget of root.querySelectorAll('.widget')) {
    syncHover(widget, event);
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

renderDashboard();
setInterval(renderDashboard, 60000);
