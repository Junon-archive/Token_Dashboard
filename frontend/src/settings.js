export const DEFAULT_APP_SETTINGS = Object.freeze({
  version: 1,
  widgets: {
    claude: { enabled: true, position: { x: 120, y: 80 } },
    codex: { enabled: true, position: { x: 280, y: 80 } },
    pomodoro: { enabled: true, position: { x: 440, y: 80 } },
  },
  widget_scale: 1.0,
  polling: {
    interval_sec: 180,
    min_interval_sec: 120,
  },
  click_through: false,
  autostart: false,
  pomodoro: { focus_min: 20, break_min: 5 },
  endpoints: {
    claude_usage: 'https://api.anthropic.com/api/oauth/usage',
    claude_beta_header: 'oauth-2025-04-20',
    codex_base: 'https://chatgpt.com/backend-api/',
    codex_usage_path: 'wham/usage',
  },
});

const WIDGETS = [
  ['claude', 'Claude Code'],
  ['codex', 'Codex CLI'],
  ['pomodoro', 'Pomodoro'],
];

const ENDPOINT_FIELDS = [
  ['claude_usage', 'Claude usage URL', 'url'],
  ['claude_beta_header', 'Claude beta header', 'text'],
  ['codex_base', 'Codex base URL', 'url'],
  ['codex_usage_path', 'Codex usage path', 'text'],
];

function cloneDefaultSettings() {
  return JSON.parse(JSON.stringify(DEFAULT_APP_SETTINGS));
}

function isPlainObject(value) {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value);
}

function mergeSettings(base, override) {
  if (!isPlainObject(override)) {
    return base;
  }
  const merged = { ...base };
  for (const key of Object.keys(base)) {
    const value = override[key];
    if (isPlainObject(value) && isPlainObject(base[key])) {
      merged[key] = mergeSettings(base[key], value);
    } else if (value !== undefined) {
      merged[key] = value;
    }
  }
  return merged;
}

export function normalizeSettings(settings) {
  const normalized = mergeSettings(cloneDefaultSettings(), settings);
  normalized.widgets = mergeSettings(cloneDefaultSettings().widgets, normalized.widgets);
  normalized.polling.interval_sec = Number(normalized.polling.interval_sec) || 180;
  normalized.polling.min_interval_sec = Number(normalized.polling.min_interval_sec) || 120;
  normalized.widget_scale = Number(normalized.widget_scale) || 1;
  normalized.pomodoro.focus_min = Number(normalized.pomodoro.focus_min) || 20;
  normalized.pomodoro.break_min = Number(normalized.pomodoro.break_min) || 5;
  return normalized;
}

function invokeFor(targetWindow = window) {
  return targetWindow.__TAURI__?.core?.invoke ?? null;
}

export async function loadAppSettings(targetWindow = window) {
  const invoke = invokeFor(targetWindow);
  if (!invoke) {
    return normalizeSettings();
  }
  try {
    return normalizeSettings(await invoke('get_app_settings'));
  } catch {
    return normalizeSettings();
  }
}

export async function saveAppSettings(settings, targetWindow = window) {
  const normalized = normalizeSettings(settings);
  const invoke = invokeFor(targetWindow);
  if (!invoke) {
    return normalized;
  }
  return normalizeSettings(await invoke('save_app_settings', { settings: normalized }));
}

function escapeAttr(value) {
  return String(value ?? '')
    .replaceAll('&', '&amp;')
    .replaceAll('"', '&quot;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;');
}

function checked(value) {
  return value ? ' checked' : '';
}

function toggleRow(id, label, checkedValue) {
  return `<label class="settings-toggle" for="${id}">
    <span>${label}</span>
    <input id="${id}" name="${id}" type="checkbox"${checked(checkedValue)} />
  </label>`;
}

function numberField(id, label, value, attrs = '') {
  return `<label class="settings-field" for="${id}">
    <span>${label}</span>
    <input id="${id}" name="${id}" type="number" value="${escapeAttr(value)}" ${attrs} />
  </label>`;
}

function endpointField([key, label, type], settings) {
  const id = `endpoint-${key}`;
  return `<label class="settings-field" for="${id}">
    <span>${label}</span>
    <input id="${id}" name="endpoints.${key}" type="${type}" value="${escapeAttr(settings.endpoints[key] ?? '')}" autocomplete="off" />
  </label>`;
}

export function renderSettingsForm(settings = DEFAULT_APP_SETTINGS) {
  const normalized = normalizeSettings(settings);
  const widgetControls = WIDGETS.map(([key, label]) => (
    toggleRow(`widget-${key}`, label, normalized.widgets[key]?.enabled)
  )).join('');
  const endpointControls = ENDPOINT_FIELDS.map((field) => endpointField(field, normalized)).join('');

  return `<main class="settings-shell" aria-label="Settings">
    <form class="settings-form" data-settings-form>
      <header class="settings-header">
        <h1>Settings</h1>
        <button class="settings-save" type="submit">Save</button>
      </header>

      <section class="settings-section" aria-labelledby="settings-widgets">
        <h2 id="settings-widgets">Widgets</h2>
        <div class="settings-grid">${widgetControls}</div>
      </section>

      <section class="settings-section" aria-labelledby="settings-behavior">
        <h2 id="settings-behavior">Behavior</h2>
        <div class="settings-grid">
          ${numberField('polling-interval-sec', 'Usage polling interval seconds', normalized.polling.interval_sec, 'min="120" step="30"')}
          ${numberField('widget-scale', 'Widget scale', normalized.widget_scale, 'min="0.75" max="1.5" step="0.05"')}
          ${toggleRow('autostart', 'Autostart', normalized.autostart)}
        </div>
      </section>

      <section class="settings-section" aria-labelledby="settings-pomodoro">
        <h2 id="settings-pomodoro">Pomodoro</h2>
        <div class="settings-grid">
          ${numberField('pomodoro-focus-min', 'Focus minutes', normalized.pomodoro.focus_min, 'min="1" max="180" step="1"')}
          ${numberField('pomodoro-break-min', 'Break minutes', normalized.pomodoro.break_min, 'min="1" max="60" step="1"')}
        </div>
      </section>

      <details class="settings-section settings-advanced">
        <summary>Advanced endpoints</summary>
        <div class="settings-grid">${endpointControls}</div>
      </details>
      <details class="settings-section settings-advanced settings-experimental">
        <summary>Experimental features</summary>
        <div class="settings-grid">
          ${toggleRow('click-through', 'Click through', normalized.click_through)}
        </div>
        <p class="settings-hint">Experimental: click-through makes the widget ignore mouse input. Keep this settings window open to turn it off.</p>
      </details>
      <p class="settings-status" role="status" data-settings-status></p>
    </form>
  </main>`;
}

function numberValue(form, name, fallback) {
  const value = Number(form.elements[name]?.value);
  return Number.isFinite(value) ? value : fallback;
}

function checkboxValue(form, name) {
  return Boolean(form.elements[name]?.checked);
}

export function collectSettingsFromForm(form, previousSettings = DEFAULT_APP_SETTINGS) {
  const previous = normalizeSettings(previousSettings);
  return normalizeSettings({
    ...previous,
    widgets: {
      claude: { ...previous.widgets.claude, enabled: checkboxValue(form, 'widget-claude') },
      codex: { ...previous.widgets.codex, enabled: checkboxValue(form, 'widget-codex') },
      pomodoro: { ...previous.widgets.pomodoro, enabled: checkboxValue(form, 'widget-pomodoro') },
    },
    widget_scale: numberValue(form, 'widget-scale', previous.widget_scale),
    polling: {
      ...previous.polling,
      interval_sec: numberValue(form, 'polling-interval-sec', previous.polling.interval_sec),
    },
    click_through: checkboxValue(form, 'click-through'),
    autostart: checkboxValue(form, 'autostart'),
    pomodoro: {
      ...previous.pomodoro,
      focus_min: numberValue(form, 'pomodoro-focus-min', previous.pomodoro.focus_min),
      break_min: numberValue(form, 'pomodoro-break-min', previous.pomodoro.break_min),
    },
    endpoints: {
      claude_usage: form.elements['endpoints.claude_usage']?.value ?? previous.endpoints.claude_usage,
      claude_beta_header: form.elements['endpoints.claude_beta_header']?.value ?? previous.endpoints.claude_beta_header,
      codex_base: form.elements['endpoints.codex_base']?.value ?? previous.endpoints.codex_base,
      codex_usage_path: form.elements['endpoints.codex_usage_path']?.value ?? previous.endpoints.codex_usage_path,
    },
  });
}

export async function initSettingsApp(root = document.querySelector('#app'), targetWindow = window) {
  if (!root) {
    return null;
  }
  let currentSettings = await loadAppSettings(targetWindow);
  root.innerHTML = renderSettingsForm(currentSettings);

  const form = root.querySelector('[data-settings-form]');
  const status = root.querySelector('[data-settings-status]');
  const saveButton = root.querySelector('.settings-save');
  form.addEventListener('submit', async (event) => {
    event.preventDefault();
    currentSettings = collectSettingsFromForm(form, currentSettings);
    saveButton.disabled = true;
    saveButton.classList.add('is-saving');
    saveButton.textContent = 'Saving...';
    status.textContent = '';
    try {
      currentSettings = await saveAppSettings(currentSettings, targetWindow);
      saveButton.classList.remove('is-saving');
      saveButton.classList.add('is-saved');
      saveButton.textContent = 'Saved';
      status.textContent = currentSettings.click_through
        ? 'Saved. Click-through is on; use this settings window to turn it off.'
        : 'Saved';
      setTimeout(() => {
        saveButton.classList.remove('is-saved');
        saveButton.textContent = 'Save';
      }, 1200);
    } catch {
      saveButton.classList.remove('is-saving');
      saveButton.textContent = 'Save';
      status.textContent = 'Unable to save settings';
    } finally {
      saveButton.disabled = false;
    }
  });
  return { form, getSettings: () => currentSettings };
}

if (typeof document !== 'undefined') {
  initSettingsApp();
}
