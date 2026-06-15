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
  const snapshot = pomodoroSnapshot(pomodoro);
  const stateClass = snapshot.state === 'BREAK' ? 'break' : snapshot.state === 'PAUSED' ? 'paused' : 'focus';
  const isHovered = current.classList.contains('is-hovered');
  current.className = `widget pomodoro ${stateClass}`;
  if (isHovered) {
    current.classList.add('is-hovered');
  }
  current.dataset.state = snapshot.state;

  const arc = current.querySelector('.arc-main');
  const dashArray = Number.parseFloat(arc?.getAttribute('stroke-dasharray') ?? '');
  if (arc && Number.isFinite(dashArray)) {
    const usedPct = Math.max(0, Math.min(100, snapshot.primary?.used_pct ?? 0));
    arc.setAttribute('stroke-dashoffset', (dashArray * (usedPct / 100)).toFixed(2));
  }

  const minutes = current.querySelector('.pomodoro-time');
  if (minutes && !editingPomodoroMinutes) {
    minutes.textContent = formatRemainingMinutes(snapshot.primary?.resets_at);
  }

  const label = current.querySelector('.lbl');
  if (label) {
    label.textContent = snapshot.state === 'BREAK' ? 'Break' : snapshot.state === 'PAUSED' ? 'Paused' : 'Focus';
  }

  const toggle = current.querySelector('[data-pomodoro-action="toggle"]');
  if (toggle) {
    toggle.textContent = snapshot.action_label;
    toggle.setAttribute('aria-label', `${snapshot.action_label} timer`);
  }

  const skip = current.querySelector('[data-pomodoro-action="skip"]');
  if (skip) {
    skip.setAttribute('aria-label', snapshot.phase === 'BREAK' ? 'Start focus' : 'Start break');
  }
}

function handlePomodoroAction(action) {
  pomodoro = applyPomodoroAction(pomodoro, action);
  renderPomodoroOnly();
}

function clearHover(widget) {
  widget?.classList.remove('is-hovered');
}

function updateDashboardTime() {
  const now = new Date();
  for (const snapshot of providerSnapshots) {
    const provider = String(snapshot.provider ?? '').toLowerCase();
    const label = root.querySelector(`[data-countdown-provider="${provider}"]`);
    if (label) {
      label.textContent = formatResetCountdown(snapshot.primary?.resets_at, now);
    }
  }
  if (!editingPomodoroMinutes) {
    renderPomodoroOnly();
  }
}

function createPomodoroMinuteButton(text) {
  const button = document.createElement('button');
  button.className = 'num pomodoro-time';
  button.type = 'button';
  button.dataset.noDrag = 'true';
  button.dataset.pomodoroEdit = 'minutes';
  button.setAttribute('aria-label', 'Set Pomodoro minutes');
  button.textContent = text;
  return button;
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
  input.replaceWith(createPomodoroMinuteButton(formatRemainingMinutes(snapshot.primary?.resets_at)));
  renderPomodoroOnly();
}

function beginPomodoroMinuteEdit(button) {
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
  input.value = button.textContent.trim();
  input.setAttribute('aria-label', 'Pomodoro minutes');
  input.dataset.noDrag = 'true';

  button.replaceWith(input);
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

window.addEventListener('mouseleave', () => root.querySelectorAll('.widget').forEach(clearHover));
document.addEventListener('mouseleave', () => root.querySelectorAll('.widget').forEach(clearHover));
window.addEventListener('blur', () => root.querySelectorAll('.widget').forEach(clearHover));
document.addEventListener('mouseout', (event) => {
  if (!event.relatedTarget) {
    root.querySelectorAll('.widget').forEach(clearHover);
  }
});

renderDashboard();
setInterval(updateDashboardTime, 60000);
