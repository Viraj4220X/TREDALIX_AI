import assert from 'node:assert/strict';
import test from 'node:test';
import {
  decodeSettings,
  defaultSettings,
  canRunBacktest,
  requireFeatures,
  setFeature,
} from '../src/settings/storage.ts';
import { reportDownloads, saveRunHistory, loadRunHistory } from '../src/services/reports.ts';

function storage(initial: Record<string, string> = {}): Storage {
  const values = new Map(Object.entries(initial));
  return {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value),
    removeItem: (key: string) => values.delete(key),
    clear: () => values.clear(),
    key: (index: number) => [...values.keys()][index] ?? null,
    get length() {
      return values.size;
    },
  } as Storage;
}

test('defaults enable only implemented local features', () => {
  const settings = defaultSettings();
  assert.equal(settings.features.csv_import.enabled, true);
  assert.equal(settings.features.backtesting.enabled, true);
  assert.equal(settings.features.live_market_data.enabled, false);
  assert.equal(settings.features.prediction_models.enabled, false);
  assert.equal(settings.features.naive_forecast.enabled, true);
  assert.equal(settings.features.walk_forward.enabled, true);
});

test('corrupt settings fall back without rewriting the stored value', () => {
  const decoded = decodeSettings('{not-json');
  assert.equal(decoded.settings.theme, 'dark');
  assert.match(decoded.warning ?? '', /stored value was not changed/);
});

test('feature prerequisites block activation and operation', () => {
  const initial = defaultSettings();
  const withoutImport = setFeature(initial, 'csv_import', false);
  assert.throws(
    () => setFeature(withoutImport, 'backtesting', true),
    /Enable CSV import first/,
  );
  assert.throws(
    () => requireFeatures(withoutImport, ['backtesting']),
    /Enable CSV import first/,
  );
  assert.throws(
    () => requireFeatures(withoutImport, ['naive_forecast', 'walk_forward']),
    /Enable CSV import first/,
  );
});

test('backtest availability requires validated data and all required features', () => {
  const initial = defaultSettings();
  assert.equal(canRunBacktest(initial, false), false);
  assert.equal(canRunBacktest(initial, true), true);

  const disabledBacktest = setFeature(initial, 'backtesting', false);
  assert.equal(canRunBacktest(disabledBacktest, true), false);

  const disabledIndicators = setFeature(initial, 'technical_indicators', false);
  assert.equal(canRunBacktest(disabledIndicators, true), false);
});

test('baseline research is locally feature-gated and can be disabled independently', () => {
  const initial = defaultSettings();
  requireFeatures(initial, ['naive_forecast', 'walk_forward']);
  const disabled = setFeature(initial, 'walk_forward', false);
  assert.throws(
    () => requireFeatures(disabled, ['naive_forecast', 'walk_forward']),
    /Repeated walk-forward is disabled/,
  );
});

test('offline analyst requires CSV validation and can be disabled independently', () => {
  const initial = defaultSettings();
  requireFeatures(initial, ['local_analyst']);
  const withoutValidation = setFeature(initial, 'data_validation', false);
  assert.throws(
    () => requireFeatures(withoutValidation, ['local_analyst']),
    /Enable Data validation first/,
  );
  const disabled = setFeature(initial, 'local_analyst', false);
  assert.throws(
    () => requireFeatures(disabled, ['local_analyst']),
    /Offline deterministic analyst is disabled/,
  );
});

test('unavailable features cannot be enabled', () => {
  assert.throws(
    () => setFeature(defaultSettings(), 'live_market_data', true),
    /No provider adapter/,
  );
});

test('run history contains actual summary metrics and report exports are gated', () => {
  const store = storage();
  const report = {
    instrument: 'SBIN',
    data_class: 'SYNTHETIC / TEST',
    dataset_fingerprint: 'fnv1a64:test',
    metrics: { net_pnl: 12.5, return_fraction: 0.01, closed_trades: 2 },
    orders: [{ id: 1, status: 'FILLED' }],
    trades: [{ net_pnl: 12.5 }],
    equity_curve: [{ equity: 1012.5 }],
  } as BacktestReport;
  const history = saveRunHistory(report, store, '2026-10-25T00:00:00.000Z');
  assert.equal(history[0].net_pnl, 12.5);
  assert.equal(loadRunHistory(store).length, 1);
  const disabled = setFeature(defaultSettings(), 'report_export', false);
  assert.throws(() => reportDownloads(report, disabled), /Report export is disabled/);
});

import type { BacktestReport } from '../src/types.ts';
