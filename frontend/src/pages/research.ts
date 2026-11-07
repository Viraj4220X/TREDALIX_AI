import { escapeHtml } from '../components/layout.ts';
import type {
  BaselineAnalysisOptions,
  BaselineAnalysisReport,
} from '../types.ts';

function metrics(label: string, values: BaselineAnalysisReport['walk_forward']['random_walk']): string {
  return `<article class="metric-card"><span>${label}</span><strong>${values.mae.toFixed(4)}</strong><small>MAE · ${values.samples} bars</small><small>RMSE ${values.rmse.toFixed(4)} · direction ${(values.directional_accuracy * 100).toFixed(1)}%</small></article>`;
}

export function forecastSvgPath(
  forecast: readonly number[],
  observedCount: number,
  x: (index: number) => number,
  y: (value: number) => number,
): string {
  return forecast
    .map((value, index) => `${index === 0 ? 'M' : 'L'} ${x(observedCount + index).toFixed(2)} ${y(value).toFixed(2)}`)
    .join(' ');
}

function forecastChart(report: BaselineAnalysisReport): string {
  const observed = report.observed_closes;
  const forecast = report.forecast.points
    .filter((point) => point.point_estimate !== null)
    .map((point) => point.point_estimate as number);
  const values = [...observed.map((point) => point.close), ...forecast];
  if (observed.length < 2 || forecast.length === 0 || values.some((value) => !Number.isFinite(value))) {
    return '<p class="form-note">Not enough finite points to draw this series.</p>';
  }
  const min = Math.min(...values);
  const max = Math.max(...values);
  const range = max - min || Math.max(Math.abs(max) * 0.01, 1);
  const x = (index: number) => 36 + (index / (values.length - 1)) * 728;
  const y = (value: number) => 198 - ((value - min) / range) * 164;
  const observedPath = observed
    .map((point, index) => `${index === 0 ? 'M' : 'L'} ${x(index).toFixed(2)} ${y(point.close).toFixed(2)}`)
    .join(' ');
  const forecastPath = forecastSvgPath(forecast, observed.length, x, y);
  const forecastStart = x(observed.length - 0.5);
  return `<svg class="forecast-chart" viewBox="0 0 800 230" role="img" aria-label="Observed historical closes and a visibly separate model scenario estimate">
    <rect class="forecast-region" x="${forecastStart.toFixed(2)}" y="24" width="${(764 - forecastStart).toFixed(2)}" height="174" />
    <text x="36" y="18" class="chart-label">${max.toFixed(2)}</text>
    <text x="36" y="220" class="chart-label">${min.toFixed(2)}</text>
    <path class="observed-series" d="${observedPath}" />
    <path class="forecast-series" d="${forecastPath}" />
    <line class="forecast-origin" x1="${x(observed.length - 1).toFixed(2)}" y1="30" x2="${x(observed.length - 1).toFixed(2)}" y2="198" />
    <text x="${Math.min(forecastStart + 8, 650).toFixed(2)}" y="44" class="chart-label">MODEL ESTIMATE · NOT OBSERVED</text>
  </svg>`;
}

function renderResult(report: BaselineAnalysisReport): string {
  const forecastValue = report.forecast.points[0]?.point_estimate;
  const folds = report.walk_forward.folds
    .map((fold) => `<tr><td>${fold.fold}</td><td>${fold.training_bars_at_start}</td><td>${fold.random_walk.samples}</td><td>${fold.random_walk.mae.toFixed(4)}</td><td>${fold.historical_drift.mae.toFixed(4)}</td></tr>`)
    .join('');
  return `<section class="panel research-result">
    <div class="section-title compact"><div><p class="eyebrow">RUST BASELINE ANALYSIS</p><h2>${escapeHtml(report.forecast.instrument_id.exchange)}:${escapeHtml(report.instrument)} · ${escapeHtml(report.forecast.currency)} · ${escapeHtml(report.data_class)}</h2></div><span class="status-pill status-experimental">EXPERIMENTAL</span></div>
    <div class="warning-strip"><strong>Baseline only.</strong> This repeats the last close; no trained AI, uncertainty interval, market-session calendar, or performance claim is provided.</div>
    <div class="forecast-legend"><span><i class="legend-observed"></i> Observed CSV closes</span><span><i class="legend-estimate"></i> Separate model estimate</span></div>
    ${forecastChart(report)}
    <div class="dataset-facts"><div><span>Model estimate</span><strong>${forecastValue === null || forecastValue === undefined ? 'Unavailable' : forecastValue.toFixed(4)}</strong></div><div><span>Forecast bars</span><strong>${report.forecast.horizon.steps}</strong></div><div><span>Source freshness</span><strong>${escapeHtml(report.forecast.source.freshness)}</strong></div><div><span>Origin (UTC)</span><strong>${new Date(report.forecast.origin_timestamp_utc * 1000).toISOString()}</strong></div></div>
    <p class="form-note">MODEL ESTIMATE — NOT OBSERVED MARKET DATA · ${report.forecast.limitations.map(escapeHtml).join(' ')}</p>
    <h3>Expanding walk-forward · one bar ahead</h3>
    <div class="metrics-grid">${metrics('Random walk · MAE', report.walk_forward.random_walk)}${metrics('Historical drift · MAE', report.walk_forward.historical_drift)}</div>
    <div class="table-wrap"><table><thead><tr><th>Fold</th><th>Training bars</th><th>Test bars</th><th>Random-walk MAE</th><th>Drift MAE</th></tr></thead><tbody>${folds}</tbody></table></div>
    <h3>Multi-horizon chronological evaluation</h3>
    <div class="table-wrap"><table><thead><tr><th>Horizon</th><th>Samples</th><th>Random-walk MAE</th><th>Random-walk RMSE</th><th>Drift MAE</th><th>Drift RMSE</th></tr></thead><tbody>${report.walk_forward.horizon_evaluations
      .map((evaluation) => `<tr><td>${evaluation.horizon_bars} bars</td><td>${evaluation.random_walk.samples}</td><td>${evaluation.random_walk.mae.toFixed(4)}</td><td>${evaluation.random_walk.rmse.toFixed(4)}</td><td>${evaluation.historical_drift.mae.toFixed(4)}</td><td>${evaluation.historical_drift.rmse.toFixed(4)}</td></tr>`)
      .join('')}</tbody></table></div>
    <ul class="limitations-list">${report.walk_forward.warnings.map((warning) => `<li>${escapeHtml(warning)}</li>`).join('')}</ul>
  </section>`;
}

export function renderResearchPanel(
  report: BaselineAnalysisReport | null,
  options: BaselineAnalysisOptions,
  busy: boolean,
  enabled: boolean,
  blockedReason: string | null,
  intervalSeconds: number,
): string {
  const horizons = [
    { seconds: 60, label: '1 minute' },
    { seconds: 120, label: '2 minutes' },
    { seconds: 180, label: '3 minutes' },
    { seconds: 300, label: '5 minutes' },
    { seconds: 600, label: '10 minutes' },
    { seconds: 900, label: '15 minutes' },
    { seconds: 1_800, label: '30 minutes' },
    { seconds: 2_700, label: '45 minutes' },
    { seconds: 3_600, label: '1 hour' },
  ].map((horizon) => {
    const steps = horizon.seconds / intervalSeconds;
    return { ...horizon, steps, supported: Number.isInteger(steps) && steps >= 1 && steps <= 20 };
  });
  const supportedHorizons = horizons.filter((horizon) => horizon.supported);
  const forecastSteps = supportedHorizons.some((horizon) => horizon.steps === options.forecast_steps)
    ? options.forecast_steps
    : supportedHorizons[0]?.steps;
  return `<section class="research-panel">
    <form class="panel form-panel" id="baseline-form">
      <div class="panel-heading"><p class="eyebrow">OFFLINE RESEARCH</p><h2>Baseline forecast & evaluation</h2><p>Uses only the validated local CSV. Enter the instrument exchange and currency; they are not inferred from the ticker.</p></div>
      <div class="form-grid">
        <label>Exchange code<input name="exchange" value="${escapeHtml(options.exchange)}" maxlength="16" placeholder="e.g. NSE" required /></label>
        <label>Currency code<input name="currency" value="${escapeHtml(options.currency)}" maxlength="3" placeholder="e.g. INR" required /></label>
        <label>Forecast time horizon<select name="forecast_steps" required ${supportedHorizons.length ? '' : 'disabled'}>
          ${horizons.map((horizon) => `<option value="${horizon.steps}" ${horizon.supported ? '' : 'disabled'} ${horizon.steps === forecastSteps ? 'selected' : ''}>${horizon.label}${horizon.supported ? '' : ' · insufficient timeframe data'}</option>`).join('')}
        </select></label>
        <label>Walk-forward folds<input name="folds" type="number" min="1" max="20" value="${options.folds}" required /></label>
        <label>Minimum training bars<input name="minimum_training_bars" type="number" min="2" max="100000" value="${options.minimum_training_bars}" required /></label>
        <label>Test bars per fold<input name="test_bars_per_fold" type="number" min="1" max="100000" value="${options.test_bars_per_fold}" required /></label>
        <label>Evaluation horizons (bars)<input name="horizons_bars" value="${options.horizons_bars.join(',')}" maxlength="48" placeholder="1,3,5" required /></label>
      </div>
      <button class="button button-primary full-width" type="submit" ${busy || !enabled ? 'disabled' : ''}>${busy ? 'Calculating…' : 'Run baseline analysis'} <span aria-hidden="true">→</span></button>
      <p class="form-note">${blockedReason ? escapeHtml(blockedReason) : supportedHorizons.length ? `Forecast horizon uses the validated ${intervalSeconds}-second CSV interval and is limited to 20 bars. Unsupported choices remain disabled. Evaluation needs at least ${options.minimum_training_bars + options.folds * options.test_bars_per_fold + Math.max(...options.horizons_bars) - 1} bars. No network access is used.` : `Insufficient timeframe data: none of the supported horizons can be represented exactly in 20 or fewer ${intervalSeconds}-second bars.`}</p>
    </form>
    ${report ? renderResult(report) : ''}
  </section>`;
}
