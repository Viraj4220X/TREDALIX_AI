import type {
  BacktestOptions,
  BacktestReport,
  BaselineAnalysisOptions,
  BaselineAnalysisReport,
  DatasetInspection,
  DatasetOptions,
  LocalAnalysisReport,
} from '../types.ts';
import {
  BACKTEST_REQUIRED_FEATURES,
  requireFeatures,
} from '../settings/storage.ts';
import type { AppSettings } from '../types.ts';

const MAX_CSV_BYTES = 10 * 1024 * 1024;

interface WasmEngine {
  default: () => Promise<unknown>;
  inspect_csv: (
    csv: string,
    options: string,
    csvImportEnabled: boolean,
    dataValidationEnabled: boolean,
    indicatorsEnabled: boolean,
  ) => string;
  run_backtest: (
    csv: string,
    datasetOptions: string,
    runOptions: string,
    featureFlags: string,
  ) => string;
  run_baseline_analysis: (
    csv: string,
    datasetOptions: string,
    analysisOptions: string,
    featureFlags: string,
  ) => string;
  run_local_analysis: (
    csv: string,
    datasetOptions: string,
    question: string,
    featureFlags: string,
  ) => string;
}

let enginePromise: Promise<WasmEngine> | undefined;

async function engine(): Promise<WasmEngine> {
  enginePromise ??= (async () => {
    const url = `${import.meta.env.BASE_URL}wasm/tredalix_wasm.js`;
    const module = (await import(/* @vite-ignore */ url)) as WasmEngine;
    await module.default();
    return module;
  })();
  return enginePromise;
}

export async function warmEngine(): Promise<void> {
  await engine();
}

function validateCsvSize(csv: string): void {
  if (new TextEncoder().encode(csv).byteLength > MAX_CSV_BYTES) {
    throw new Error('Choose a CSV file no larger than 10 MiB.');
  }
}

export async function inspectCsv(
  csv: string,
  options: DatasetOptions,
  settings: AppSettings,
): Promise<DatasetInspection> {
  requireFeatures(settings, ['csv_import', 'data_validation']);
  validateCsvSize(csv);
  const wasm = await engine();
  const result = wasm.inspect_csv(
    csv,
    JSON.stringify(options),
    settings.features.csv_import.enabled,
    settings.features.data_validation.enabled,
    settings.features.technical_indicators.enabled,
  );
  return JSON.parse(result) as DatasetInspection;
}

export async function runBacktest(
  csv: string,
  datasetOptions: DatasetOptions,
  options: BacktestOptions,
  settings: AppSettings,
): Promise<BacktestReport> {
  requireFeatures(settings, BACKTEST_REQUIRED_FEATURES);
  validateCsvSize(csv);
  const wasm = await engine();
  const result = wasm.run_backtest(
    csv,
    JSON.stringify(datasetOptions),
    JSON.stringify(options),
    JSON.stringify({
      csv_import: settings.features.csv_import.enabled,
      data_validation: settings.features.data_validation.enabled,
      technical_indicators: settings.features.technical_indicators.enabled,
      signal_generation: settings.features.signal_generation.enabled,
      backtesting: settings.features.backtesting.enabled,
    }),
  );
  return JSON.parse(result) as BacktestReport;
}

export async function runBaselineAnalysis(
  csv: string,
  datasetOptions: DatasetOptions,
  options: BaselineAnalysisOptions,
  settings: AppSettings,
): Promise<BaselineAnalysisReport> {
  requireFeatures(settings, ['csv_import', 'data_validation', 'naive_forecast', 'walk_forward']);
  validateCsvSize(csv);
  const wasm = await engine();
  const result = wasm.run_baseline_analysis(
    csv,
    JSON.stringify(datasetOptions),
    JSON.stringify(options),
    JSON.stringify({
      csv_import: settings.features.csv_import.enabled,
      data_validation: settings.features.data_validation.enabled,
      naive_forecast: settings.features.naive_forecast.enabled,
      walk_forward: settings.features.walk_forward.enabled,
    }),
  );
  return JSON.parse(result) as BaselineAnalysisReport;
}

export async function runLocalAnalysis(
  csv: string,
  datasetOptions: DatasetOptions,
  question: string,
  settings: AppSettings,
): Promise<LocalAnalysisReport> {
  requireFeatures(settings, ['csv_import', 'data_validation', 'local_analyst']);
  validateCsvSize(csv);
  const wasm = await engine();
  const result = wasm.run_local_analysis(
    csv,
    JSON.stringify(datasetOptions),
    question,
    JSON.stringify({
      csv_import: settings.features.csv_import.enabled,
      data_validation: settings.features.data_validation.enabled,
      local_analyst: settings.features.local_analyst.enabled,
    }),
  );
  return JSON.parse(result) as LocalAnalysisReport;
}

export function resetEngineForTests(): void {
  enginePromise = undefined;
}
