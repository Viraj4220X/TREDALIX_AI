import type { AppSettings } from '../types.ts';
import { featureBlockedReason } from '../settings/features.ts';
import type { DesktopRuntimeState } from '../services/desktop-runtime.ts';

export type PageId =
  | 'dashboard'
  | 'data'
  | 'backtest'
  | 'reports'
  | 'features'
  | 'settings'
  | 'ai';

const navigation: ReadonlyArray<{ id: PageId; label: string; marker: string }> = [
  { id: 'dashboard', label: 'Dashboard', marker: 'OV' },
  { id: 'data', label: 'Data manager', marker: 'DA' },
  { id: 'backtest', label: 'Backtesting', marker: 'BT' },
  { id: 'reports', label: 'Reports', marker: 'RE' },
  { id: 'features', label: 'Features', marker: 'FX' },
  { id: 'settings', label: 'Settings', marker: 'ST' },
];

export function escapeHtml(value: unknown): string {
  return String(value ?? '')
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');
}

export function pageHeading(page: PageId): { title: string; subtitle: string } {
  const headings: Record<PageId, { title: string; subtitle: string }> = {
    dashboard: {
      title: 'Research, without the noise.',
      subtitle: 'Local market data and transparent portfolio simulation.',
    },
    data: {
      title: 'Data manager',
      subtitle: 'Import a CSV and validate it with the Rust data pipeline.',
    },
    backtest: {
      title: 'Backtesting',
      subtitle: 'Run the existing strategy with explicit assumptions.',
    },
    reports: {
      title: 'Reports',
      subtitle: 'Review saved run summaries or export the current result.',
    },
    features: {
      title: 'Feature manager',
      subtitle: 'Control available operations and review what is not implemented.',
    },
    settings: {
      title: 'Settings',
      subtitle: 'Device-local display and feature preferences.',
    },
    ai: {
      title: 'AI Control Center',
      subtitle: 'Local evidence, model scenarios, capability status, and transparent limits.',
    },
  };
  return headings[page];
}

export function renderLayout(
  page: PageId,
  settings: AppSettings,
  content: string,
  online: boolean,
  runtime: DesktopRuntimeState,
): string {
  const { title, subtitle } = pageHeading(page);
  const nav = navigation
    .map(
      (item) => `<a class="nav-link${item.id === page ? ' active' : ''}" href="#${item.id}" data-page="${item.id}"${item.id === page ? ' aria-current="page"' : ''}>
        <span class="nav-marker" aria-hidden="true">${item.marker}</span><span>${item.label}</span>
      </a>`,
    )
    .join('');
  const backtestReason = featureBlockedReason('backtesting', settings.features);
  const runtimeLabel = runtime.mode === 'browser'
    ? 'Browser mode · local WASM'
    : runtime.mode === 'starting'
      ? 'Desktop service starting'
      : runtime.mode === 'ready'
        ? 'Desktop service ready'
        : 'Desktop service unavailable';
  const runtimeClass = runtime.mode === 'ready' || runtime.mode === 'browser'
    ? 'ready'
    : runtime.mode === 'unavailable'
      ? 'warning'
      : 'empty';
  return `<div class="app-shell">
    <aside class="sidebar" aria-label="Main navigation">
      <a class="brand" href="#dashboard" data-page="dashboard">
        <span class="brand-mark" aria-hidden="true">T</span>
        <span><strong>TREDALIX</strong><small>OFFLINE ANALYTICS</small></span>
      </a>
      <div class="nav-caption">WORKSPACE</div>
      <nav class="main-nav">${nav}</nav>
      <div class="sidebar-bottom">
        <div class="connection-state"><span class="status-dot ${runtimeClass}"></span>
          <span>${escapeHtml(runtimeLabel)}</span>
        </div>
        <div class="connection-state"><span class="status-dot ${online ? 'online' : 'offline'}"></span>
          <span>Network hint: ${online ? 'available' : 'unavailable'}</span>
        </div>
        <div class="local-note">Browser network status is not provider connectivity. Local analysis works independently.</div>
        <div class="sidebar-version">TREDALIX AI <span>v0.1.0</span></div>
      </div>
    </aside>
    <main class="main-area">
      <header class="topbar">
        <div class="mobile-brand"><span class="brand-mark">T</span><strong>TREDALIX</strong></div>
        <span class="topbar-label">OFFLINE-FIRST · NO LIVE ORDERS</span>
        <button class="ai-quick${page === 'ai' ? ' active' : ''}" type="button" data-page="ai" aria-label="Open AI Control Center" aria-current="${page === 'ai' ? 'page' : 'false'}"><span aria-hidden="true">✳</span> <span>AI</span></button>
        <button class="theme-quick" type="button" data-action="toggle-theme" aria-label="Switch color theme">${settings.theme === 'dark' ? '☼' : '◐'} <span>${settings.theme === 'dark' ? 'Light' : 'Dark'} theme</span></button>
      </header>
      <div class="page-wrap">
        <div class="page-heading">
          <div><p class="eyebrow">LOCAL WORKSPACE <span class="accent-dot"></span></p><h1>${escapeHtml(title)}</h1><p class="page-subtitle">${escapeHtml(subtitle)}</p></div>
          <div class="heading-status"><span class="status-dot ${backtestReason ? 'warning' : 'online'}"></span>${backtestReason ? 'Backtest feature gated' : 'Core features ready'}</div>
        </div>
        ${content}
        <footer class="page-footer">Synthetic samples are not market evidence. Backtests are simulations, not guarantees.</footer>
      </div>
    </main>
  </div>`;
}

export function renderNotice(
  notice: { kind: 'info' | 'success' | 'warning' | 'error'; message: string } | null,
): string {
  if (!notice) return '';
  return `<div class="notice notice-${notice.kind}" role="${notice.kind === 'error' ? 'alert' : 'status'}">${escapeHtml(notice.message)}</div>`;
}
