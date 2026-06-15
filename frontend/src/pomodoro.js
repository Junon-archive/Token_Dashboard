export const DEFAULT_POMODORO = {
  focusMin: 20,
  breakMin: 5,
  dialFullMin: 60,
};

function durationForPhase(settings, phase) {
  return (phase === 'BREAK' ? settings.breakMin : settings.focusMin) * 60000;
}

function nowMs(now) {
  return new Date(now).getTime();
}

export function createPomodoroState(options = {}) {
  const settings = { ...DEFAULT_POMODORO, ...options };
  const startedAt = nowMs(options.now ?? new Date());
  const durationMs = durationForPhase(settings, 'FOCUS');
  return {
    settings,
    phase: 'FOCUS',
    isRunning: true,
    startedAt,
    durationMs,
    remainingMs: durationMs,
  };
}

function setPhase(state, phase, now, running = true) {
  const durationMs = durationForPhase(state.settings, phase);
  return {
    ...state,
    phase,
    isRunning: running,
    startedAt: nowMs(now),
    durationMs,
    remainingMs: durationMs,
  };
}

export function tickPomodoro(state, now = new Date()) {
  if (!state.isRunning) {
    return state;
  }
  const elapsedMs = Math.max(0, nowMs(now) - state.startedAt);
  if (elapsedMs < state.durationMs) {
    return {
      ...state,
      remainingMs: state.durationMs - elapsedMs,
    };
  }
  return setPhase(state, state.phase === 'FOCUS' ? 'BREAK' : 'FOCUS', now, true);
}

export function applyPomodoroAction(state, action, now = new Date()) {
  const current = tickPomodoro(state, now);
  if (action === 'toggle') {
    if (current.isRunning) {
      return {
        ...current,
        isRunning: false,
        remainingMs: Math.max(0, current.startedAt + current.durationMs - nowMs(now)),
      };
    }
    return {
      ...current,
      isRunning: true,
      startedAt: nowMs(now) - (current.durationMs - current.remainingMs),
    };
  }
  if (action === 'reset') {
    return setPhase(current, current.phase, now, false);
  }
  if (action === 'skip') {
    return setPhase(current, current.phase === 'FOCUS' ? 'BREAK' : 'FOCUS', now, true);
  }
  return current;
}

export function setPomodoroMinutes(state, minutes, now = new Date()) {
  const parsed = Number(minutes);
  if (!Number.isFinite(parsed)) {
    return tickPomodoro(state, now);
  }
  const clamped = Math.max(1, Math.min(180, Math.round(parsed)));
  const current = tickPomodoro(state, now);
  const settingsKey = current.phase === 'BREAK' ? 'breakMin' : 'focusMin';
  const settings = {
    ...current.settings,
    [settingsKey]: clamped,
  };
  const durationMs = clamped * 60000;

  return {
    ...current,
    settings,
    isRunning: false,
    startedAt: nowMs(now),
    durationMs,
    remainingMs: durationMs,
  };
}

export function pomodoroSnapshot(state, now = new Date()) {
  const current = tickPomodoro(state, now);
  const elapsedMs = current.isRunning
    ? Math.max(0, nowMs(now) - current.startedAt)
    : current.durationMs - current.remainingMs;
  const usedPct = Math.min(100, (elapsedMs / (current.settings.dialFullMin * 60000)) * 100);
  return {
    provider: 'pomodoro',
    state: current.isRunning ? current.phase : 'PAUSED',
    phase: current.phase,
    action_label: current.isRunning ? 'Pause' : 'Resume',
    primary: {
      used_pct: usedPct,
      resets_at: new Date(nowMs(now) + current.remainingMs).toISOString(),
    },
    secondary: null,
    fetched_at: new Date(now).toISOString(),
    is_stale: false,
  };
}
