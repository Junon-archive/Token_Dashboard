import { renderUsageWidget } from './widget.js';

const fallbackSnapshot = {
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
};

async function loadSnapshot() {
  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    return fallbackSnapshot;
  }
  try {
    return await invoke('mock_claude_snapshot');
  } catch {
    return fallbackSnapshot;
  }
}

const root = document.querySelector('#app');
root.innerHTML = renderUsageWidget(await loadSnapshot());

const widget = root.querySelector('.widget');

function clearHover() {
  widget?.classList.remove('is-hovered');
}

function syncHover(event) {
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

if (widget) {
  widget.addEventListener('mouseenter', () => widget.classList.add('is-hovered'));
  widget.addEventListener('mouseleave', clearHover);
  widget.addEventListener('pointerenter', () => widget.classList.add('is-hovered'));
  widget.addEventListener('pointerleave', clearHover);
  window.addEventListener('mousemove', syncHover);
  window.addEventListener('pointermove', syncHover);
  window.addEventListener('mouseleave', clearHover);
  document.addEventListener('mouseleave', clearHover);
  window.addEventListener('blur', clearHover);
  document.addEventListener('mouseout', (event) => {
    if (!event.relatedTarget) {
      clearHover();
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
