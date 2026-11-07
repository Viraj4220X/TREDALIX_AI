import { escapeHtml } from '../components/layout.ts';
import type { DatasetInspection } from '../types.ts';
import type { BaselineAnalysisOptions, BaselineAnalysisReport } from '../types.ts';
import { renderResearchPanel } from './research.ts';
import { renderAnalystPanel } from './analyst.ts';
import type { LocalAnalysisReport } from '../types.ts';

export function renderDataPage(
  inspection: DatasetInspection | null,
  filename: string | null,
  hasCsv: boolean,
  options: {
    instrument: string;
    instrument_type: string;
    source: string;
    interval_seconds: number;
    synthetic: boolean;
  },
  research: {
    report: BaselineAnalysisReport | null;
    options: BaselineAnalysisOptions;
    busy: boolean;
    enabled: boolean;
    blockedReason: string | null;
  },
  activeIndicators: readonly string[],
  indicatorsEnabled: boolean,
  analyst: {
    report: LocalAnalysisReport | null;
    question: string;
    busy: boolean;
    enabled: boolean;
    blockedReason: string | null;
  },
): string {
  const checkedIndicator = (name: string) => activeIndicators.includes(name) ? 'checked' : '';
  const current = inspection
    ? `<section class="panel data-result">
        <div class="section-title compact"><div><p class="eyebrow">RUST VALIDATION RESULT</p><h2>${escapeHtml(inspection.instrument)} <span class="class-badge">${escapeHtml(inspection.data_class)}</span></h2></div><span class="file-name">${escapeHtml(filename)}</span></div>
        <div class="dataset-facts">
          <div><span>Rows</span><strong>${inspection.row_count.toLocaleString()}</strong></div>
          <div><span>Interval</span><strong>${inspection.interval_seconds}s</strong></div>
          <div><span>Timezone</span><strong>${escapeHtml(inspection.timestamp_timezone)}</strong></div>
          <div><span>Observed gaps</span><strong>${inspection.gaps.observed}</strong></div>
        </div>
        <p class="range-text">${escapeHtml(inspection.first_timestamp)} <span>to</span> ${escapeHtml(inspection.last_timestamp)}</p>
        <div class="warning-strip"><strong>Session calendar not configured.</strong> Gap classification does not confirm whether an interval is a market closure or missing data.</div>
        <div class="table-wrap"><table><thead><tr><th>Timestamp</th><th>Open</th><th>High</th><th>Low</th><th>Close</th><th>Volume</th></tr></thead><tbody>${inspection.preview
          .map(
            (row) => `<tr><td>${escapeHtml(row.timestamp)}</td><td>${row.open}</td><td>${row.high}</td><td>${row.low}</td><td>${row.close}</td><td>${row.volume ?? '—'}</td></tr>`,
          )
          .join('')}</tbody></table></div>
        <p class="table-caption">Showing up to 12 preview rows. The complete CSV is retained in memory only for this browser session.</p>
        <section class="ohlc-chart-panel">
          <div class="section-title compact"><div><p class="eyebrow">LOCAL OHLC VISUALIZATION</p><h2>Candlesticks & indicators</h2></div><span class="status-pill status-experimental">${inspection.chart.candles.length.toLocaleString()} bars${inspection.chart.sampled ? ' · downsampled' : ''}</span></div>
          <p class="form-note">Source: ${escapeHtml(inspection.chart.source)}. The chart uses validated CSV rows; it is not a live market feed.</p>
          ${indicatorsEnabled ? `<div class="chart-indicator-controls" aria-label="Chart indicators">
            <label><input type="checkbox" data-indicator="sma_20" ${checkedIndicator('sma_20')} /> SMA 20</label>
            <label><input type="checkbox" data-indicator="ema_20" ${checkedIndicator('ema_20')} /> EMA 20</label>
            <label><input type="checkbox" data-indicator="bollinger_20" ${checkedIndicator('bollinger_20')} /> Bollinger 20</label>
            <label><input type="checkbox" data-indicator="volume" ${checkedIndicator('volume')} /> Volume</label>
            <label><input type="checkbox" data-indicator="rsi_14" ${checkedIndicator('rsi_14')} /> RSI 14</label>
            <label><input type="checkbox" data-indicator="macd" ${checkedIndicator('macd')} /> MACD 12/26/9</label>
            <label><input type="checkbox" data-indicator="atr_14" ${checkedIndicator('atr_14')} /> ATR 14</label>
            ${research.report ? `<label><input type="checkbox" data-indicator="forecast" ${checkedIndicator('forecast')} /> Baseline forecast</label>` : ''}
          </div>` : '<p class="warning-strip">Technical indicators are disabled in Feature Manager. Enable them to calculate Rust indicator series.</p>'}
          ${!indicatorsEnabled && research.report ? `<div class="chart-indicator-controls"><label><input type="checkbox" data-indicator="forecast" ${checkedIndicator('forecast')} /> Baseline forecast estimate (not observed)</label></div>` : ''}
          <div class="chart-container" id="ohlc-chart" role="img" aria-label="Interactive OHLC candlestick chart with local CSV data"></div>
          ${indicatorsEnabled ? `<div class="chart-subpanel"><h3>RSI 14 · Rust calculated</h3><div class="chart-container chart-container-small" id="rsi-chart" role="img" aria-label="Relative strength index chart"></div></div>
          <div class="chart-subpanel"><h3>MACD 12/26/9 · Rust calculated</h3><div class="chart-container chart-container-small" id="macd-chart" role="img" aria-label="MACD indicator chart"></div></div>
          <div class="chart-subpanel"><h3>ATR 14 · Rust calculated</h3><div class="chart-container chart-container-small" id="atr-chart" role="img" aria-label="Average true range indicator chart"></div></div>
          <p class="table-caption">Indicators use the validated local candle data and causal warm-up periods. Pan/zoom with the mouse or touch; use the checkboxes to show or hide series.</p>` : ''}
          <p class="chart-attribution">TradingView Lightweight Charts™ Copyright (c) 2025 TradingView, Inc. · <a href="https://www.tradingview.com/" target="_blank" rel="noopener noreferrer">tradingview.com</a> · Apache-2.0</p>
        </section>
      </section>`
    : `<section class="panel upload-empty"><div class="empty-upload-icon">CSV</div><h2>${filename ? 'CSV loaded, validation pending' : 'No dataset in this session'}</h2><p>${filename ? `${escapeHtml(filename)} is held in memory for this session. Confirm its metadata and validate it.` : 'Select a local CSV. It is validated by the Rust core and is not uploaded.'}</p></section>`;
  return `<section class="two-column content-grid">
    <form class="panel form-panel" id="import-form">
      <div class="panel-heading"><p class="eyebrow">LOCAL FILE INPUT</p><h2>Import & validate</h2><p>Required columns: timestamp, open, high, low, close. Volume is optional.</p></div>
      <label class="file-drop" for="csv-file"><span class="file-drop-mark">↑</span><strong>Select candle CSV</strong><small>CSV only · maximum 10 MiB · never uploaded</small><input id="csv-file" name="file" type="file" accept=".csv,text/csv" /></label>
      <div class="form-grid">
        <label>Instrument<input name="instrument" value="${escapeHtml(options.instrument)}" maxlength="32" required /></label>
        <label>Instrument type<select name="instrument_type"><option value="unknown" ${options.instrument_type === 'unknown' ? 'selected' : ''}>Unknown / unverified</option><option value="equity" ${options.instrument_type === 'equity' ? 'selected' : ''}>Cash equity</option><option value="index" ${options.instrument_type === 'index' ? 'selected' : ''}>Index</option></select></label>
        <label>Interval (seconds)<input name="interval_seconds" type="number" min="1" step="1" value="${options.interval_seconds}" required /></label>
        <label>Source description<input name="source" value="${escapeHtml(options.source)}" maxlength="120" required /></label>
      </div>
      <label class="checkbox-row"><input name="synthetic" type="checkbox" ${options.synthetic ? 'checked' : ''} /><span><strong>This is synthetic/test data</strong><small>Do not use sample data to claim real-market performance.</small></span></label>
      <button class="button button-primary full-width" type="submit" data-action="validate" ${hasCsv ? '' : 'disabled'}>Validate selected CSV <span aria-hidden="true">→</span></button>
      <p class="form-note">Selecting a file triggers Rust validation. Changing metadata requires validating the selected file again.</p>
    </form>
    <div class="data-result-column">${current}</div>
  </section>${inspection ? renderResearchPanel(
    research.report,
    research.options,
    research.busy,
    research.enabled,
    research.blockedReason,
    options.interval_seconds,
  ) : ''}${inspection ? renderAnalystPanel(
    analyst.report,
    analyst.question,
    analyst.busy,
    analyst.enabled,
    analyst.blockedReason,
  ) : ''}`;
}
