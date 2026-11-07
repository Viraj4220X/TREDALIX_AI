import { escapeHtml } from '../components/layout.ts';
import type { DatasetInspection, RunHistoryItem } from '../types.ts';

interface DashboardData {
  inspection: DatasetInspection | null;
  history: RunHistoryItem[];
  wasmReady: boolean;
}

export function renderDashboard(data: DashboardData): string {
  const dataset = data.inspection;
  const latestRun = data.history[0];
  const cards = [
    {
      label: 'Local dataset',
      value: dataset ? escapeHtml(dataset.instrument) : 'Not loaded',
      detail: dataset
        ? `${dataset.row_count.toLocaleString()} rows · ${dataset.interval_seconds}s candles`
        : 'Import a CSV to inspect its structure',
      state: dataset ? 'ready' : 'empty',
    },
    {
      label: 'Data quality',
      value: dataset ? 'Validated' : 'Awaiting data',
      detail: dataset
        ? `${dataset.gaps.observed} timestamp gap(s) observed`
        : 'Validation runs in the shared Rust engine',
      state: dataset ? 'ready' : 'empty',
    },
    {
      label: 'Latest backtest',
      value: latestRun ? escapeHtml(latestRun.instrument) : 'No runs yet',
      detail: latestRun
        ? `${latestRun.closed_trades} closed trades · ${escapeHtml(latestRun.data_class)}`
        : 'No performance figures are invented',
      state: latestRun ? 'ready' : 'empty',
    },
    {
      label: 'Engine',
      value: data.wasmReady ? 'Rust / WASM ready' : 'Loading local engine',
      detail: data.wasmReady
        ? 'Calculations run locally in the browser'
        : 'Build the WASM adapter before analysis',
      state: data.wasmReady ? 'ready' : 'warning',
    },
  ];
  return `<section class="hero-panel">
    <div class="hero-copy">
      <div class="hero-kicker"><span class="status-dot online"></span> PRIVATE BY DESIGN</div>
      <h2>Make the data<br><span>earn your trust.</span></h2>
      <p>Inspect local candles, configure a transparent simulation, and keep every assumption visible. No exchange connection. No live orders.</p>
      <div class="hero-actions">
        <button class="button button-primary" data-page="data">Import market data <span aria-hidden="true">→</span></button>
        <button class="button button-quiet" data-page="backtest">Configure backtest</button>
      </div>
    </div>
    <div class="hero-art" aria-hidden="true"><div class="orbit orbit-one"></div><div class="orbit orbit-two"></div><div class="orbit-core"><span>TA</span></div><div class="orbit-tag tag-a">LOCAL</div><div class="orbit-tag tag-b">RUST</div><div class="orbit-tag tag-c">OFFLINE</div></div>
    <div class="hero-foot"><span>01 / INPUT</span><span>02 / VALIDATE</span><span>03 / SIMULATE</span><span>NO LIVE EXECUTION</span></div>
  </section>
  <section class="section-block">
    <div class="section-title"><div><p class="eyebrow">AT A GLANCE</p><h2>Workspace status</h2></div><span class="subtle-label">REAL VALUES ONLY</span></div>
    <div class="stats-grid">${cards
      .map(
        (card, index) => `<article class="stat-card">
          <div class="stat-top"><span class="stat-index">0${index + 1}</span><span class="status-dot ${card.state}"></span></div>
          <p class="stat-label">${card.label}</p><strong class="stat-value">${card.value}</strong><p class="stat-detail">${escapeHtml(card.detail)}</p>
        </article>`,
      )
      .join('')}</div>
  </section>
  <section class="section-block dashboard-bottom">
    <article class="panel panel-wide"><div class="section-title compact"><div><p class="eyebrow">LATEST ACTIVITY</p><h2>Recent simulations</h2></div><button class="text-button" data-page="reports">View reports →</button></div>
      ${latestRun ? `<div class="activity-row"><span class="activity-icon">BT</span><div><strong>${escapeHtml(latestRun.instrument)} · ${escapeHtml(latestRun.data_class)}</strong><small>${new Date(latestRun.created_at).toLocaleString()} · ${escapeHtml(latestRun.dataset_fingerprint)}</small></div><span class="activity-number">${latestRun.closed_trades} trades</span></div>` : '<div class="empty-state compact-empty"><span class="empty-symbol">—</span><div><strong>No backtest history</strong><p>Run an analysis to record a real local summary.</p></div></div>'}
    </article>
    <article class="panel safety-panel"><p class="eyebrow">SAFETY CHECK</p><h2>Simulation only.</h2><p>Generic fee and slippage estimates do not represent complete Indian statutory charges. Synthetic data cannot establish market performance.</p><button class="text-button" data-page="features">Review feature availability →</button></article>
  </section>`;
}
