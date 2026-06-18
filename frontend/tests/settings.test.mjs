import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import {
  collectSettingsFromForm,
  DEFAULT_APP_SETTINGS,
  loadAppSettings,
  normalizeSettings,
  renderSettingsForm,
  saveAppSettings,
} from '../src/settings.js';

test('renders SPEC UI-9 settings controls with collapsed advanced endpoints', () => {
  const html = renderSettingsForm();

  assert.match(html, /<form class="settings-form" data-settings-form>/);
  assert.match(html, /<button class="settings-quit" type="button" data-quit-app>Quit<\/button>/);
  assert.match(html, /<button class="settings-save" type="submit">Save<\/button>/);
  assert.match(html, /id="widget-claude" name="widget-claude" type="checkbox" checked/);
  assert.match(html, /id="widget-codex" name="widget-codex" type="checkbox" checked/);
  assert.match(html, /id="widget-pomodoro" name="widget-pomodoro" type="checkbox" checked/);
  assert.match(html, /id="polling-interval-sec" name="polling-interval-sec" type="number" value="180" min="120" step="30"/);
  assert.match(html, /Usage polling interval seconds/);
  assert.match(html, /id="widget-scale" name="widget-scale" type="number" value="1" min="0.75" max="1.5" step="0.05"/);
  assert.doesNotMatch(html, /notifications-enabled|Notifications/);
  assert.match(html, /id="autostart" name="autostart" type="checkbox"/);
  assert.match(html, /id="pomodoro-focus-min" name="pomodoro-focus-min" type="number" value="20" min="1" max="180" step="1"/);
  assert.match(html, /id="pomodoro-break-min" name="pomodoro-break-min" type="number" value="5" min="1" max="60" step="1"/);
  assert.match(html, /<details class="settings-section settings-advanced">/);
  assert.match(html, /<summary>Advanced endpoints<\/summary>/);
  assert.match(html, /name="endpoints\.claude_usage"/);
  assert.match(html, /name="endpoints\.claude_beta_header"/);
  assert.match(html, /name="endpoints\.codex_base"/);
  assert.match(html, /name="endpoints\.codex_usage_path"/);
  assert.doesNotMatch(html, /Experimental features|click-through|Click through/);
});

test('settings page auto-initializes the settings app', async () => {
  const source = await readFile(new URL('../src/settings.js', import.meta.url), 'utf8');
  const html = await readFile(new URL('../../settings.html', import.meta.url), 'utf8');
  const buildScript = await readFile(new URL('../../scripts/build-frontend.mjs', import.meta.url), 'utf8');

  assert.match(html, /<script type="module" src="\.\/frontend\/src\/settings\.js"><\/script>/);
  assert.match(source, /if \(typeof document !== 'undefined'\) \{\s*initSettingsApp\(\);/s);
  assert.match(buildScript, /cp\('settings\.html', 'dist\/settings\.html'\)/);
});

test('settings quit button invokes desktop quit command', async () => {
  const calls = [];
  const listeners = {};
  const root = {
    innerHTML: '',
    querySelector: (selector) => {
      if (selector === '[data-settings-form]') {
        return {
          addEventListener: (event, handler) => {
            listeners[event] = handler;
          },
        };
      }
      if (selector === '[data-settings-status]') {
        return { textContent: '' };
      }
      if (selector === '.settings-save') {
        return { disabled: false, classList: { add() {}, remove() {} }, textContent: 'Save' };
      }
      if (selector === '[data-quit-app]') {
        return {
          addEventListener: (event, handler) => {
            listeners[`quit:${event}`] = handler;
          },
        };
      }
      return null;
    },
  };
  const targetWindow = {
    __TAURI__: {
      core: {
        invoke: async (command) => {
          calls.push(command);
          return command === 'get_app_settings' ? DEFAULT_APP_SETTINGS : null;
        },
      },
    },
  };

  await import('../src/settings.js').then(({ initSettingsApp }) => initSettingsApp(root, targetWindow));
  await listeners['quit:click']();

  assert(calls.includes('quit_app'));
});

test('settings surface exposes no credential fields', async () => {
  const html = renderSettingsForm();
  const source = await readFile(new URL('../src/settings.js', import.meta.url), 'utf8');

  assert.doesNotMatch(html, /access[_-]?token|refresh[_-]?token|id[_-]?token|api[_-]?key|authorization|secret/i);
  assert.doesNotMatch(source, /access[_-]?token|refresh[_-]?token|id[_-]?token|api[_-]?key|authorization|secret/i);
});

test('settings surface does not expose notifications', async () => {
  const html = renderSettingsForm();
  const settingsSource = await readFile(new URL('../src/settings.js', import.meta.url), 'utf8');
  const dashboardSource = await readFile(new URL('../src/main.js', import.meta.url), 'utf8');

  assert.doesNotMatch(html, /Notification|notifications-enabled/i);
  assert.doesNotMatch(settingsSource, /notifications-enabled/);
  assert.doesNotMatch(dashboardSource, /Notification|dispatchUsageNotifications|usageThresholdEvents/);
});

test('loads fallback defaults when Tauri invoke is absent', async () => {
  const settings = await loadAppSettings({});

  assert.deepEqual(settings, normalizeSettings(DEFAULT_APP_SETTINGS));
});

test('uses expected Tauri command names when invoke is present', async () => {
  const calls = [];
  const targetWindow = {
    __TAURI__: {
      core: {
        invoke: async (command, payload) => {
          calls.push([command, payload]);
          if (command === 'get_app_settings') {
            return { polling: { interval_sec: 240 }, widgets: { codex: { enabled: false } } };
          }
          return null;
        },
      },
    },
  };

  const loaded = await loadAppSettings(targetWindow);
  assert.equal(loaded.polling.interval_sec, 240);
  assert.equal(loaded.widgets.codex.enabled, false);

  await saveAppSettings(loaded, targetWindow);
  assert.deepEqual(calls.map(([command]) => command), ['get_app_settings', 'save_app_settings']);
  assert.equal(calls[1][1].settings.polling.interval_sec, 240);
});

test('save button has visible saving and saved states', async () => {
  const source = await readFile(new URL('../src/settings.js', import.meta.url), 'utf8');
  const css = await readFile(new URL('../src/settings.css', import.meta.url), 'utf8');

  assert.match(source, /saveButton\.disabled = true;/);
  assert.match(source, /saveButton\.textContent = 'Saving\.\.\.'/);
  assert.match(source, /saveButton\.classList\.add\('is-saved'\)/);
  assert.match(source, /status\.textContent = 'Saved';/);
  assert.doesNotMatch(source, /use this settings window to turn it off/);
  assert.match(css, /\.settings-save:active,/);
  assert.match(css, /\.settings-save\.is-saved/);
});

test('hidden click-through setting is preserved on save', () => {
  const html = renderSettingsForm({ click_through: true });
  assert.doesNotMatch(html, /click-through|Click through/);

  const template = {
    innerHTML: html,
    querySelector: (selector) => {
      if (selector === '[data-settings-form]') {
        return template.form;
      }
      return null;
    },
  };
  template.form = {
    elements: {
      'widget-claude': { checked: true },
      'widget-codex': { checked: true },
      'widget-pomodoro': { checked: true },
      'widget-scale': { value: '1' },
      'polling-interval-sec': { value: '180' },
      autostart: { checked: false },
      'pomodoro-focus-min': { value: '20' },
      'pomodoro-break-min': { value: '5' },
    },
  };

  const saved = normalizeSettings({
    ...DEFAULT_APP_SETTINGS,
    ...collectSettingsFromForm(template.form, { ...DEFAULT_APP_SETTINGS, click_through: true }),
  });
  assert.equal(saved.click_through, true);
});

test('normalization strips settings outside the frontend form contract', () => {
  const normalized = normalizeSettings({
    advanced: {
      access_token: 'synthetic',
    },
    endpoints: {
      claude_usage: 'https://api.anthropic.com/api/oauth/usage',
    },
  });

  assert.equal(Object.hasOwn(normalized, 'advanced'), false);
});
