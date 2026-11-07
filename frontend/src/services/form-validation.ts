import type {
  BacktestOptions,
  BaselineAnalysisOptions,
  DatasetOptions,
} from '../types.ts';

export function formData(form: HTMLFormElement): FormData {
  return new FormData(form);
}

export function datasetOptionsFrom(form: HTMLFormElement): DatasetOptions {
  const values = formData(form);
  const interval = Number(values.get('interval_seconds'));
  const instrument = String(values.get('instrument') ?? '').trim();
  const source = String(values.get('source') ?? '').trim();
  const type = String(values.get('instrument_type'));
  if (!instrument || instrument.length > 32) throw new Error('Enter an instrument name (1–32 characters).');
  if (!source || source.length > 120) throw new Error('Enter a source description (1–120 characters).');
  if (!Number.isSafeInteger(interval) || interval <= 0) {
    throw new Error('Candle interval must be a positive whole number of seconds.');
  }
  if (type !== 'equity' && type !== 'index' && type !== 'unknown') {
    throw new Error('Choose a valid instrument type.');
  }
  return {
    instrument,
    instrument_type: type,
    source,
    interval_seconds: interval,
    synthetic: values.get('synthetic') === 'on',
  };
}

export function backtestOptionsFrom(
  form: HTMLFormElement,
  instrumentType: DatasetOptions['instrument_type'],
): BacktestOptions {
  const values = formData(form);
  const options: BacktestOptions = {
    initial_capital: Number(values.get('initial_capital')),
    fee_bps: Number(values.get('fee_bps')),
    slippage_bps: Number(values.get('slippage_bps')),
    max_quantity: Number(values.get('max_quantity')),
    max_exposure: Number(values.get('max_exposure')),
    out_of_sample_fraction: Number(values.get('out_of_sample_fraction')),
    buy_and_hold_benchmark: values.get('buy_and_hold_benchmark') === 'on',
  };
  if (
    !Number.isFinite(options.initial_capital) ||
    options.initial_capital <= 0 ||
    !Number.isFinite(options.max_exposure) ||
    options.max_exposure <= 0 ||
    !Number.isSafeInteger(options.max_quantity) ||
    options.max_quantity <= 0
  ) {
    throw new Error('Capital and exposure must be positive; quantity must be a positive whole number.');
  }
  for (const [label, value] of [
    ['Fee', options.fee_bps],
    ['Slippage', options.slippage_bps],
  ] as const) {
    if (!Number.isFinite(value) || value < 0 || value >= 10_000) {
      throw new Error(`${label} must be between 0 and 10,000 basis points.`);
    }
  }
  if (
    !Number.isFinite(options.out_of_sample_fraction) ||
    options.out_of_sample_fraction <= 0 ||
    options.out_of_sample_fraction > 1
  ) {
    throw new Error('Out-of-sample fraction must be greater than 0 and at most 1.');
  }
  if (options.buy_and_hold_benchmark && instrumentType !== 'equity') {
    throw new Error('The buy-and-hold benchmark requires explicit cash-equity classification.');
  }
  return options;
}

export function baselineOptionsFrom(form: HTMLFormElement): BaselineAnalysisOptions {
  const values = formData(form);
  const options: BaselineAnalysisOptions = {
    exchange: String(values.get('exchange') ?? '').trim(),
    currency: String(values.get('currency') ?? '').trim(),
    forecast_steps: Number(values.get('forecast_steps')),
    folds: Number(values.get('folds')),
    minimum_training_bars: Number(values.get('minimum_training_bars')),
    test_bars_per_fold: Number(values.get('test_bars_per_fold')),
    horizons_bars: String(values.get('horizons_bars') ?? '')
      .split(',')
      .map((value) => Number(value.trim())),
  };
  if (!options.exchange || options.exchange.length > 16) {
    throw new Error('Enter the exchange code (1–16 characters).');
  }
  if (!/^[A-Za-z]{3}$/.test(options.currency)) {
    throw new Error('Enter a three-letter currency code, such as INR.');
  }
  for (const [label, value, minimum, maximum] of [
    ['Forecast bars', options.forecast_steps, 1, 20],
    ['Walk-forward folds', options.folds, 1, 20],
    ['Minimum training bars', options.minimum_training_bars, 2, 100_000],
    ['Test bars per fold', options.test_bars_per_fold, 1, 100_000],
  ] as const) {
    if (!Number.isSafeInteger(value) || value < minimum || value > maximum) {
      throw new Error(`${label} must be a whole number from ${minimum} to ${maximum}.`);
    }
  }
  if (
    options.horizons_bars.length === 0 ||
    options.horizons_bars.length > 10 ||
    options.horizons_bars.some((value) => !Number.isSafeInteger(value) || value < 1 || value > 1_000) ||
    new Set(options.horizons_bars).size !== options.horizons_bars.length
  ) {
    throw new Error('Enter 1–10 unique forecast horizons as comma-separated whole bars (each 1–1,000).');
  }
  const requiredBars =
    options.minimum_training_bars +
    options.folds * options.test_bars_per_fold +
    Math.max(...options.horizons_bars) -
    1;
  if (requiredBars > 100_000) {
    throw new Error('The requested folds require more than 100,000 candles.');
  }
  return options;
}
