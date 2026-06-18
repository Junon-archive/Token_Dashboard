export const DEFAULT_POMODORO = {
  focusMin: 20,
  breakMin: 5,
};

const ENDING_BLINK_MS = 30000;

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
    isEnding: false,
    pendingPhase: null,
    startedAt,
    durationMs,
    remainingMs: durationMs,
    endedAt: null,
  };
}

export function updatePomodoroSettings(state, options = {}, now = new Date()) {
  const focusMin = Number(options.focusMin ?? options.focus_min ?? state.settings.focusMin);
  const breakMin = Number(options.breakMin ?? options.break_min ?? state.settings.breakMin);
  const settings = {
    focusMin: Number.isFinite(focusMin) ? Math.max(1, Math.min(180, Math.round(focusMin))) : state.settings.focusMin,
    breakMin: Number.isFinite(breakMin) ? Math.max(1, Math.min(180, Math.round(breakMin))) : state.settings.breakMin,
  };
  const current = tickPomodoro(state, now);
  const durationMs = durationForPhase(settings, current.phase);
  const ratio = current.durationMs > 0 ? current.remainingMs / current.durationMs : 1;
  const remainingMs = current.isRunning
    ? Math.max(0, Math.min(durationMs, durationMs * ratio))
    : durationMs;

  return {
    ...current,
    settings,
    startedAt: current.isRunning ? nowMs(now) - (durationMs - remainingMs) : nowMs(now),
    durationMs,
    remainingMs,
  };
}

function setPhase(state, phase, now, running = true) {
  const durationMs = durationForPhase(state.settings, phase);
  return {
    ...state,
    phase,
    isRunning: running,
    isEnding: false,
    pendingPhase: null,
    startedAt: nowMs(now),
    durationMs,
    remainingMs: durationMs,
    endedAt: null,
  };
}

function pauseAtCurrentPhase(current, now) {
  return {
    ...current,
    isRunning: false,
    isEnding: false,
    pendingPhase: null,
    startedAt: nowMs(now),
    endedAt: null,
    remainingMs: current.durationMs,
  };
}

function enterEndingPhase(current, now) {
  const nextPhase = current.phase === 'FOCUS' ? 'BREAK' : 'FOCUS';
  return {
    ...current,
    isRunning: false,
    isEnding: true,
    pendingPhase: nextPhase,
    endedAt: nowMs(now),
    remainingMs: 0,
  };
}

function acknowledgeEndedPhase(current, now) {
  if (!current.isEnding) {
    return current;
  }
  const nextPhase = current.pendingPhase ?? (current.phase === 'FOCUS' ? 'BREAK' : 'FOCUS');
  return {
    ...current,
    phase: nextPhase,
    isRunning: false,
    isEnding: false,
    pendingPhase: null,
    startedAt: nowMs(now),
    durationMs: durationForPhase(current.settings, nextPhase),
    remainingMs: durationForPhase(current.settings, nextPhase),
    endedAt: null,
  };
}

export function tickPomodoro(state, now = new Date()) {
  if (state.isEnding) {
    const endedAt = Number.isFinite(state.endedAt) ? state.endedAt : nowMs(now);
    if (nowMs(now) - endedAt >= ENDING_BLINK_MS) {
      return acknowledgeEndedPhase(state, now);
    }
    return {
      ...state,
      remainingMs: 0,
      durationMs: durationForPhase(state.settings, state.phase),
      endedAt,
    };
  }
  if (!state.isRunning) {
    return state;
  }
  let phase = state.phase;
  let elapsedMs = Math.max(0, nowMs(now) - state.startedAt);
  let durationMs = durationForPhase(state.settings, phase);

  while (elapsedMs >= durationMs) {
    elapsedMs -= durationMs;
    return enterEndingPhase({
      ...state,
      phase,
      durationMs,
      startedAt: nowMs(now) - elapsedMs,
      remainingMs: 0,
    }, now);
  }

  return {
    ...state,
    phase,
    durationMs,
    startedAt: nowMs(now) - elapsedMs,
    remainingMs: durationMs - elapsedMs,
  };
}

export function applyPomodoroAction(state, action, now = new Date()) {
  const current = tickPomodoro(state, now);
  if (action === 'toggle') {
    if (current.isEnding) {
      return acknowledgeEndedPhase(current, now);
    }
    if (current.isRunning) {
      return {
        ...current,
        isRunning: false,
        remainingMs: Math.max(0, current.startedAt + current.durationMs - nowMs(now)),
      };
    }
    if (current.remainingMs >= current.durationMs) {
      return {
        ...current,
        isRunning: true,
        startedAt: nowMs(now),
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
  if (action === 'acknowledge') {
    return acknowledgeEndedPhase(current, now);
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
    isEnding: false,
    pendingPhase: null,
    startedAt: nowMs(now),
    durationMs,
    remainingMs: durationMs,
    endedAt: null,
  };
}

export function pomodoroSnapshot(state, now = new Date()) {
  const current = tickPomodoro(state, now);
  const elapsedMs = Math.max(0, current.durationMs - current.remainingMs);
  const usedPct = current.durationMs > 0
    ? Math.min(100, (elapsedMs / current.durationMs) * 100)
    : 100;
  const stateLabel = current.isEnding
    ? 'ENDING'
    : current.isRunning
      ? current.phase
      : 'PAUSED';
  const actionLabel = current.isEnding
    ? 'Start'
    : current.isRunning
      ? 'Pause'
      : current.remainingMs >= current.durationMs
        ? 'Start'
        : 'Resume';
  return {
    provider: 'pomodoro',
    state: stateLabel,
    phase: current.phase,
    action_label: actionLabel,
    primary: {
      used_pct: usedPct,
      resets_at: new Date(nowMs(now) + Math.max(0, current.remainingMs)).toISOString(),
    },
    secondary: null,
    fetched_at: new Date(now).toISOString(),
    is_stale: false,
  };
}
