import { escapeHtml } from '../components/layout.ts';
import type { AppSettings, DesktopDiagnostics } from '../types.ts';
import type { DesktopRuntimeState } from '../services/desktop-runtime.ts';
import type { ResearchStatus } from '../services/ai-research.ts';

function renderDiagnostics(runtime: DesktopRuntimeState): string {
  if (runtime.mode === 'browser') {
    return `<article class="panel diagnostics-panel"><div class="setting-copy"><p class="eyebrow">RUNTIME DIAGNOSTICS</p><h2>Browser mode</h2><p>The Web app runs directly in this browser. No desktop IPC service or localhost server is required.</p><span class="status-pill status-available">BROWSER · LOCAL WASM</span></div></article>`;
  }
  if (runtime.mode === 'starting') {
    return `<article class="panel diagnostics-panel"><div class="setting-copy"><p class="eyebrow">RUNTIME DIAGNOSTICS</p><h2>Desktop service starting</h2><p>Waiting for the embedded Rust host to answer its readiness command.</p></div></article>`;
  }
  if (runtime.mode === 'unavailable') {
    return `<article class="panel diagnostics-panel"><div class="setting-copy"><p class="eyebrow">RUNTIME DIAGNOSTICS</p><h2>Desktop service unavailable</h2><p>${escapeHtml(runtime.message)}</p></div><button class="button button-secondary" type="button" data-action="refresh-runtime">Retry health check</button></article>`;
  }
  const status: DesktopDiagnostics = runtime.diagnostics;
  return `<article class="panel diagnostics-panel"><div class="setting-copy"><p class="eyebrow">RUNTIME DIAGNOSTICS</p><h2>Desktop service ready</h2><p>Build ${escapeHtml(status.build_mode)} · UI version ${escapeHtml(status.app_version)} · API version ${status.api_version} · transport ${escapeHtml(status.transport)}. UI assets use Tauri's bundled application origin; no HTTP listener is opened.</p><p>Local capabilities: ${status.capabilities.map(escapeHtml).join(', ')}.</p><span class="status-pill status-available">READY · OFFLINE LOCAL</span></div><button class="button button-secondary" type="button" data-action="refresh-runtime">Refresh health check</button></article>`;
}

export function renderSettings(
  settings: AppSettings,
  runtime: DesktopRuntimeState,
  researchStatus: ResearchStatus | null = null,
): string {
  const searchTitle = runtime.mode === 'browser'
    ? 'Web Search Not Configured'
    : researchStatus?.configured
      ? 'Local provider settings detected · unverified'
      : researchStatus
        ? 'Web Search Not Configured'
        : runtime.mode === 'unavailable'
          ? 'Provider status unavailable'
          : 'Checking provider status';
  const searchNotice = runtime.mode === 'browser'
    ? 'Web search requires the desktop backend. No browser search request or credential access is performed.'
    : researchStatus?.notice
      ?? (runtime.mode === 'unavailable'
        ? 'The desktop backend could not report provider configuration. This does not confirm whether credentials exist.'
        : 'Waiting for the desktop backend to report provider configuration.');
  return `<section class="settings-grid">
    <article class="panel setting-card"><div class="setting-icon">UI</div><div class="setting-copy"><p class="eyebrow">APPEARANCE</p><h2>Color theme</h2><p>Your preference is stored locally in this browser or app profile.</p></div>
      <label class="select-setting"><span class="visually-hidden">Color theme</span><select name="theme" data-setting="theme"><option value="dark" ${settings.theme === 'dark' ? 'selected' : ''}>Dark</option><option value="light" ${settings.theme === 'light' ? 'selected' : ''}>Light</option></select></label>
    </article>
    <article class="panel setting-card"><div class="setting-icon">ST</div><div class="setting-copy"><p class="eyebrow">DEVICE PROFILE</p><h2>Local settings</h2><p>Schema version ${settings.schemaVersion}. No account or cloud sync is used. The same values are not shared across devices.</p></div><span class="status-pill status-available">LOCAL ONLY</span></article>
    <article class="panel setting-card"><div class="setting-icon">WS</div><div class="setting-copy"><p class="eyebrow">OPTIONAL SEARCH PROVIDER</p><h2>Web search</h2><p><strong>${escapeHtml(searchTitle)}</strong> · ${escapeHtml(searchNotice)}</p><p>No API keys or Search Engine IDs are stored in the frontend or shown here. The legacy Google adapter requires both values in the desktop app process environment; do not configure it until valid credentials and an Engine ID are confirmed.</p></div></article>
    <article class="panel setting-card danger-zone"><div class="setting-icon">↺</div><div class="setting-copy"><p class="eyebrow">RESET</p><h2>Restore defaults</h2><p>Resets theme and feature switches. Datasets and report files are not deleted.</p></div><button class="button button-danger" data-action="reset-settings">Reset settings</button></article>
    ${renderDiagnostics(runtime)}
  </section>
  <section class="panel limitations-panel"><p class="eyebrow">APP CAPABILITIES</p><h2>Explicitly out of scope</h2><div class="limitations-grid"><span>✕ Live exchange prices</span><span>✕ Broker orders</span><span>✕ Prediction accuracy claims</span><span>✕ Cloud accounts / telemetry</span></div><p class="table-caption">Synthetic input remains labeled synthetic. Backtests cannot guarantee future returns.</p></section>`;
}
