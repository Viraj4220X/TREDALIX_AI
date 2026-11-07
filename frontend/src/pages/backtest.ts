import { escapeHtml } from '../components/layout.ts';
import type { BacktestOptions, BacktestReport, DatasetInspection } from '../types.ts';

function metric(label: string, value: string, detail: string): string {
  return `<div class="result-metric"><span>${escapeHtml(label)}</span><strong>${escapeHtml(value)}</strong><small>${escapeHtml(detail)}</small></div>`;
}

export function renderBacktestPage(
  inspection: DatasetInspection | null,
  report: BacktestReport | null,
  busy: boolean,
  canRun: boolean,
  options: BacktestOptions,
): string {
  const results = report
    ? `<section class="panel result-panel">
        <div class="section-title compact"><div><p class="eyebrow">ACTUAL RUST ENGINE RESULT</p><h2>${escapeHtml(report.instrument)} · ${escapeHtml(report.data_class)}</h2></div><span class="class-badge">${escapeHtml(report.dataset_fingerprint)}</span></div>
        <div class="result-metrics">
          ${metric('Net PnL', report.metrics.net_pnl.toFixed(2), 'Includes costs; final open position marked to market')}
          ${metric('Return', `${(report.metrics.return_fraction * 100).toFixed(2)}%`, 'Single trailing holdout')}
          ${metric('Max drawdown', `${(report.metrics.max_drawdown_fraction * 100).toFixed(2)}%`, report.metrics.max_drawdown_amount.toFixed(2))}
          ${metric('Closed trades', String(report.metrics.closed_trades), `${report.metrics.winning_trades} wins · ${report.metrics.losing_trades} losses`)}
        </div>
        <div class="result-detail-grid">
          <div><span>Orders</span><strong>${report.metrics.submitted_orders} submitted / ${report.metrics.filled_orders} filled / ${report.metrics.rejected_orders} rejected</strong></div>
          <div><span>Costs</span><strong>Fees ${report.metrics.fees.toFixed(2)} · estimated slippage ${report.metrics.slippage.toFixed(2)}</strong></div>
          <div><span>Strategy</span><strong>${escapeHtml(report.strategy_name)} v${escapeHtml(report.strategy_version)}</strong></div>
          <div><span>Evaluation</span><strong>${escapeHtml(report.evaluation.first_timestamp)} — ${escapeHtml(report.evaluation.last_timestamp)}</strong></div>
        </div>
        <div class="chart-panel"><div class="chart-heading"><div><p class="eyebrow">EQUITY CURVE</p><strong>Marked portfolio equity</strong></div><span>${report.equity_curve.length} points</span></div>${renderEquityChart(report)}</div>
        ${report.trades.length ? `<div class="table-wrap"><table><thead><tr><th>Entry</th><th>Exit</th><th>Qty</th><th>Entry</th><th>Exit</th><th>Net PnL</th></tr></thead><tbody>${report.trades.map((trade) => `<tr><td>${escapeHtml(trade.entry_timestamp)}</td><td>${escapeHtml(trade.exit_timestamp)}</td><td>${escapeHtml(trade.quantity)}</td><td>${escapeHtml(Number(trade.entry_price).toFixed(2))}</td><td>${escapeHtml(Number(trade.exit_price).toFixed(2))}</td><td>${escapeHtml(Number(trade.net_pnl).toFixed(2))}</td></tr>`).join('')}</tbody></table></div>` : '<div class="empty-state"><strong>No closed trades in this run</strong><p>This is an actual engine result; no trade statistics have been invented.</p></div>'}
        <div class="assumption-box"><strong>Simulation limitations</strong><ul>${report.assumptions.map((assumption) => `<li>${escapeHtml(assumption)}</li>`).join('')}</ul></div>
        <button class="button button-secondary" data-page="reports">Open report exports →</button>
      </section>`
    : `<section class="panel result-placeholder"><span class="empty-symbol">—</span><h2>No backtest result</h2><p>Run a simulation on validated session data to see actual metrics, equity points, orders and trades.</p></section>`;
  return `<section class="two-column content-grid">
    <form class="panel form-panel" id="backtest-form">
      <div class="panel-heading"><p class="eyebrow">ENGINE CONFIGURATION</p><h2>Simulation settings</h2><p>Only parameters accepted by the existing Rust engine are shown.</p></div>
      ${inspection ? `<div class="selected-dataset"><span>Selected input</span><strong>${escapeHtml(inspection.instrument)} · ${inspection.row_count} rows</strong><small>${escapeHtml(inspection.data_class)} · ${inspection.interval_seconds}s</small></div>` : '<div class="warning-strip">Import and validate a CSV before configuring a run.</div>'}
      <div class="form-grid">
        <label>Initial capital<input name="initial_capital" type="number" min="0.01" step="any" value="${options.initial_capital}" required /></label>
        <label>Fee estimate (bps)<input name="fee_bps" type="number" min="0" max="9999.99" step="any" value="${options.fee_bps}" required /></label>
        <label>Slippage estimate (bps)<input name="slippage_bps" type="number" min="0" max="9999.99" step="any" value="${options.slippage_bps}" required /></label>
        <label>Maximum quantity<input name="max_quantity" type="number" min="1" step="1" value="${options.max_quantity}" required /></label>
        <label>Maximum exposure<input name="max_exposure" type="number" min="0.01" step="any" value="${options.max_exposure}" required /></label>
        <label>Out-of-sample fraction<input name="out_of_sample_fraction" type="number" min="0.0001" max="1" step="any" value="${options.out_of_sample_fraction}" required /></label>
      </div>
      <label class="checkbox-row"><input name="buy_and_hold_benchmark" type="checkbox" ${options.buy_and_hold_benchmark ? 'checked' : ''} ${inspection?.instrument_type === 'equity' ? '' : 'disabled'} /><span><strong>Include cash-equity buy-and-hold benchmark</strong><small>Only allowed for explicitly classified cash equity data.</small></span></label>
      <div class="warning-strip"><strong>Generic costs only.</strong> Fees are not a complete India-specific taxes/duties model. The test window is a single trailing holdout, not repeated walk-forward validation.</div>
      <button class="button button-primary full-width" type="submit" ${!canRun || busy ? 'disabled' : ''}>${busy ? 'Running Rust backtest…' : 'Run offline backtest'} <span aria-hidden="true">→</span></button>
      <p class="form-note">${canRun ? 'Uses the local Rust/WASM engine. No network connection or broker account.' : 'Import and validate a CSV, then enable the required data, strategy, and backtest features.'}</p>
    </form>
    <div class="result-column">${results}</div>
  </section>`;
}

function renderEquityChart(report: BacktestReport): string {
  const values = report.equity_curve
    .map((point) => Number(point.equity))
    .filter(Number.isFinite);
  if (values.length < 2) {
    return '<div class="chart-empty">Not enough points to draw a curve.</div>';
  }
  const width = 760;
  const height = 180;
  const min = Math.min(...values);
  const max = Math.max(...values);
  const range = max - min || 1;
  const points = values
    .map((value, index) => {
      const x = (index / (values.length - 1)) * width;
      const y = height - ((value - min) / range) * (height - 18) - 9;
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(' ');
  return `<svg class="equity-chart" viewBox="0 0 ${width} ${height}" role="img" aria-label="Portfolio equity from ${escapeHtml(min.toFixed(2))} to ${escapeHtml(max.toFixed(2))}">
    <line x1="0" y1="${height - 1}" x2="${width}" y2="${height - 1}" class="chart-axis" />
    <polyline points="${points}" class="chart-line" />
  </svg><div class="chart-scale"><span>${min.toFixed(2)}</span><span>${max.toFixed(2)}</span></div>`;
}
