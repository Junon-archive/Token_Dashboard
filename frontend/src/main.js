import { renderUsageDashboard } from './widget.js';

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

async function loadSnapshots() {
  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    return fallbackSnapshots;
  }
  try {
    return await invoke('mock_usage_snapshots');
  } catch {
    return fallbackSnapshots;
  }
}

const root = document.querySelector('#app');
root.innerHTML = renderUsageDashboard(await loadSnapshots());

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

for (const widget of root.querySelectorAll('.widget')) {
  widget.addEventListener('mouseenter', () => widget.classList.add('is-hovered'));
  widget.addEventListener('mouseleave', () => clearHover(widget));
  widget.addEventListener('pointerenter', () => widget.classList.add('is-hovered'));
  widget.addEventListener('pointerleave', () => clearHover(widget));
  window.addEventListener('mousemove', (event) => syncHover(widget, event));
  window.addEventListener('pointermove', (event) => syncHover(widget, event));
  window.addEventListener('mouseleave', () => clearHover(widget));
  document.addEventListener('mouseleave', () => clearHover(widget));
  window.addEventListener('blur', () => clearHover(widget));
  document.addEventListener('mouseout', (event) => {
    if (!event.relatedTarget) {
      clearHover(widget);
    }
  });

  widget.addEventListener(
    'mousedown',
    (event) => {
      if (event.button !== 0 || event.detail !== 1) {
        return;
      }
      event.preventDefault();
      window.__TAURI__?.window?.getCurrentWindow?.().startDragging?.();
    },
    { capture: true },
  );
}
