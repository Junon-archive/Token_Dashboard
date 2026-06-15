const CX = 70;
const CY = 70;
const MAIN_R = 55;
const SEC_R = 43;
const C_MAIN = 2 * Math.PI * MAIN_R;
const C_SEC = 2 * Math.PI * SEC_R;
const TICK_N = 48;
const TICK_Y = 7;
const TICK_MINOR_W = 1.4;
const TICK_MINOR_H = 4.5;
const TICK_MAJOR_W = 2;
const TICK_MAJOR_H = 7;

const LAMP_KEY =
  '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="8" cy="8" r="4.2"/><path d="M11 11l8 8"/><path d="M16 16l2.2-2.2"/><path d="M18.2 18.2l1.6-1.6"/></svg>';
const LAMP_BANG =
  '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 3.5L22 20H2L12 3.5z"/><path d="M12 9.5v4.4"/><path d="M12 17.4v.1"/></svg>';
const CLOCK =
  '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/></svg>';

const PROVIDERS = {
  claude: {
    className: 'claude',
    label: 'Claude',
    ariaLabel: 'Claude usage widget',
  },
  codex: {
    className: 'codex',
    label: 'Codex',
    ariaLabel: 'Codex usage widget',
  },
  pomodoro: {
    className: 'pomodoro',
    label: 'Focus',
    ariaLabel: 'Pomodoro timer widget',
  },
};

export function providerView(provider) {
  return PROVIDERS[String(provider ?? '').toLowerCase()] ?? PROVIDERS.claude;
}

export function visualClassForSnapshot(snapshot) {
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
    case 'FOCUS':
      return 'focus';
    case 'BREAK':
      return 'break';
    case 'PAUSED':
      return 'paused';
    default:
      return 'stale';
  }
}

export function ticksSvg(options = {}) {
  const majorEvery = options.majorEvery ?? 6;
  let output = '';
  for (let i = 0; i < TICK_N; i += 1) {
    const isMajor = i % majorEvery === 0;
    const width = isMajor ? TICK_MAJOR_W : TICK_MINOR_W;
    const height = isMajor ? TICK_MAJOR_H : TICK_MINOR_H;
    const x = CX - width / 2;
    const rotation = (i / TICK_N) * 360;
    const className = isMajor ? 'tick tick-major' : 'tick';
    output += `<rect class="${className}" x="${x.toFixed(2)}" y="${TICK_Y}" width="${width}" height="${height}" rx="${(width / 2).toFixed(2)}" transform="rotate(${rotation.toFixed(2)} ${CX} ${CY})"/>`;
  }
  return output;
}

export function remainingFraction(window) {
  if (!window || !Number.isFinite(window.used_pct)) {
    return 0.5;
  }
  return Math.max(0, Math.min(1, 1 - window.used_pct / 100));
}

function arcsSvg(primary, secondary) {
  const mainRemaining = remainingFraction(primary);
  const mainOffset = (C_MAIN * (1 - mainRemaining)).toFixed(2);
  let output = `<g transform="rotate(-90 ${CX} ${CY})">`;
  output += `<circle class="ring-track-main" cx="${CX}" cy="${CY}" r="${MAIN_R}"/>`;
  output += `<circle class="arc-main" cx="${CX}" cy="${CY}" r="${MAIN_R}" stroke-dasharray="${C_MAIN.toFixed(2)}" stroke-dashoffset="${mainOffset}"/>`;
  if (secondary) {
    const secRemaining = remainingFraction(secondary);
    const secOffset = (C_SEC * (1 - secRemaining)).toFixed(2);
    output += `<circle class="ring-track-sec" cx="${CX}" cy="${CY}" r="${SEC_R}"/>`;
    output += `<circle class="arc-sec" cx="${CX}" cy="${CY}" r="${SEC_R}" stroke-dasharray="${C_SEC.toFixed(2)}" stroke-dashoffset="${secOffset}"/>`;
  }
  output += '</g>';
  return output;
}

export function formatResetCountdown(resetsAt, now = new Date()) {
  const resetMs = new Date(resetsAt).getTime();
  if (!Number.isFinite(resetMs)) {
    return '--:--';
  }
  const totalMinutes = Math.max(0, Math.ceil((resetMs - now.getTime()) / 60000));
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  return `${hours}:${String(minutes).padStart(2, '0')}`;
}

export function staleAgeLabel(fetchedAt, now = new Date()) {
  const fetchedMs = new Date(fetchedAt).getTime();
  if (!Number.isFinite(fetchedMs)) {
    return 'stale';
  }
  const minutes = Math.max(0, Math.floor((now.getTime() - fetchedMs) / 60000));
  return `${minutes}m ago`;
}

export function formatRemainingMinutes(endsAt, now = new Date()) {
  const endMs = new Date(endsAt).getTime();
  if (!Number.isFinite(endMs)) {
    return '--';
  }
  return String(Math.max(0, Math.ceil((endMs - now.getTime()) / 60000)));
}

function lampForSnapshot(snapshot) {
  if (snapshot.state === 'NOT_LOGGED_IN') {
    return { icon: LAMP_KEY, text: 'Sign in' };
  }
  if (snapshot.state === 'AUTH_ERROR') {
    return { icon: LAMP_BANG, text: 'Auth' };
  }
  return { icon: '', text: '' };
}

export function renderUsageWidget(snapshot, options = {}) {
  const now = options.now ?? new Date();
  const provider = providerView(options.provider ?? snapshot.provider);
  const classes = ['widget', provider.className, visualClassForSnapshot(snapshot)].filter(Boolean).join(' ');
  const countdown = formatResetCountdown(snapshot.primary?.resets_at, now);
  const lamp = lampForSnapshot(snapshot);

  return `<section class="${classes}" data-provider="${provider.className}" data-state="${snapshot.state}" data-tauri-drag-region="deep" aria-label="${provider.ariaLabel}">
    <div class="disk"></div>
    <svg class="gauge" viewBox="0 0 140 140" aria-hidden="true">${arcsSvg(snapshot.primary, snapshot.secondary)}${ticksSvg()}</svg>
    <div class="center">
      <div class="num" data-countdown-provider="${provider.className}">${countdown}</div>
      <div class="lbl">${provider.label}</div>
      <div class="lamp">${lamp.icon}<span class="lt">${lamp.text}</span></div>
    </div>
    <div class="update-badge">${CLOCK}<span>${staleAgeLabel(snapshot.fetched_at, now)}</span></div>
  </section>`;
}

export function renderClaudeWidget(snapshot, options = {}) {
  return renderUsageWidget({ ...snapshot, provider: 'claude' }, options);
}

export function renderPomodoroWidget(timer, options = {}) {
  const now = options.now ?? new Date();
  const classes = ['widget', 'pomodoro', visualClassForSnapshot(timer)].filter(Boolean).join(' ');
  const label = timer.state === 'BREAK' ? 'Break' : timer.state === 'PAUSED' ? 'Paused' : 'Focus';
  const actionLabel = timer.action_label ?? (timer.state === 'PAUSED' ? 'Resume' : 'Pause');
  const minutes = formatRemainingMinutes(timer.primary?.resets_at, now);

  const skipLabel = timer.phase === 'BREAK' ? 'Start focus' : 'Start break';

  return `<section class="${classes}" data-provider="pomodoro" data-state="${timer.state}" data-tauri-drag-region="deep" aria-label="Pomodoro timer widget">
    <div class="disk"></div>
    <svg class="gauge" viewBox="0 0 140 140" aria-hidden="true">${arcsSvg(timer.primary, null)}${ticksSvg({ majorEvery: 4 })}</svg>
    <div class="center">
      <button class="num pomodoro-time" type="button" data-no-drag="true" data-pomodoro-edit="minutes" aria-label="Set Pomodoro minutes">${minutes}</button>
      <div class="lbl">${label}</div>
      <div class="lamp"><span class="lt"></span></div>
    </div>
    <div class="pomodoro-controls" role="toolbar" aria-label="Pomodoro controls" data-no-drag="true">
      <button class="pomodoro-btn reset" type="button" data-pomodoro-action="reset" aria-label="Reset timer">Reset</button>
      <button class="pomodoro-btn toggle" type="button" data-pomodoro-action="toggle" aria-label="${actionLabel} timer">${actionLabel}</button>
      <button class="pomodoro-btn skip" type="button" data-pomodoro-action="skip" aria-label="${skipLabel}">Skip</button>
    </div>
  </section>`;
}

export function renderUsageDashboard(snapshots, options = {}) {
  const items = Array.isArray(snapshots) ? snapshots : [snapshots].filter(Boolean);
  const widgets = items
    .map((snapshot) => (
      String(snapshot.provider).toLowerCase() === 'pomodoro'
        ? renderPomodoroWidget(snapshot, options)
        : renderUsageWidget(snapshot, options)
    ))
    .join('');
  return `<main class="dashboard" aria-label="Token usage dashboard">${widgets}</main>`;
}
