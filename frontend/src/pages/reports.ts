import { escapeHtml } from '../components/layout.ts';
import type { BacktestReport, RunHistoryItem } from '../types.ts';

export function renderReports(
  history: readonly RunHistoryItem[],
  currentReport: BacktestReport | null,
  exportEnabled: boolean,
): string {
  const entries = history.length
    ? history
        .map(
          (item) => `<article class="report-row">
            <span class="report-icon">BT</span>
            <div class="report-main"><strong>${escapeHtml(item.instrument)} · ${escapeHtml(item.data_class)}</strong><small>${new Date(item.created_at).toLocaleString()} · ${escapeHtml(item.dataset_fingerprint)}</small></div>
            <div class="report-metric"><small>NET PnL</small><strong>${item.net_pnl.toFixed(2)}</strong></div>
            <div class="report-metric"><small>RETURN</small><strong>${(item.return_fraction * 100).toFixed(2)}%</strong></div>
            <div class="report-metric"><small>TRADES</small><strong>${item.closed_trades}</strong></div>
          </article>`,
        )
        .join('')
    : '<div class="empty-state"><span class="empty-symbol">—</span><strong>No saved simulation summaries</strong><p>Only runs generated in this app appear here. No sample results are prefilled.</p></div>';
  const exportControls = currentReport
    ? `<div class="export-panel panel"><div><p class="eyebrow">CURRENT SESSION RESULT</p><h2>${escapeHtml(currentReport.instrument)} · ${escapeHtml(currentReport.dataset_fingerprint)}</h2><p>Download the actual Rust engine output in JSON and CSV formats.</p></div>
        ${exportEnabled ? `<div class="export-actions"><button class="button button-secondary" data-download="json">JSON</button><button class="button button-secondary" data-download="metrics">Metrics CSV</button><button class="button button-secondary" data-download="orders">Orders CSV</button><button class="button button-secondary" data-download="trades">Trades CSV</button><button class="button button-secondary" data-download="equity">Equity CSV</button></div>` : '<span class="disabled-help">Enable report export in Feature Manager.</span>'}
      </div>`
    : '<div class="export-panel panel export-empty"><div><p class="eyebrow">EXPORTS</p><h2>No current result to export</h2><p>Run a backtest to create exportable JSON and CSV files.</p></div><button class="button button-secondary" data-page="backtest">Go to backtesting →</button></div>';
  return `<section class="section-block">${exportControls}</section>
    <section class="panel report-list"><div class="section-title compact"><div><p class="eyebrow">DEVICE-LOCAL HISTORY</p><h2>Recent run summaries</h2></div><span class="subtle-label">${history.length} / 8</span></div>${entries}<p class="table-caption">History stores summary metrics only in this browser profile. The source CSV is not saved.</p></section>`;
}
