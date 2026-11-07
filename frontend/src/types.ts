export type Theme = 'dark' | 'light';

export interface FeatureSetting {
  enabled: boolean;
}

export interface AppSettings {
  schemaVersion: 1;
  theme: Theme;
  features: Record<string, FeatureSetting>;
}

export interface DesktopDiagnostics {
  status: 'ready';
  app_version: string;
  api_version: number;
  transport: 'tauri_ipc';
  build_mode: 'development' | 'release';
  capabilities: string[];
}

export interface CandlePreview {
  timestamp: string;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number | null;
}

export interface DatasetInspection {
  instrument: string;
  instrument_type: string;
  source: string;
  data_class: string;
  interval_seconds: number;
  timestamp_timezone: string;
  timestamp_meaning: string;
  row_count: number;
  first_timestamp: string;
  last_timestamp: string;
  first_timestamp_epoch: number;
  last_timestamp_epoch: number;
  gaps: {
    observed: number;
    unobserved_intervals: number;
    irregular: number;
    session_calendar_configured: boolean;
  };
  preview: CandlePreview[];
  chart: {
    source: string;
    sampled: boolean;
    candles: ChartCandle[];
    sma_20: ChartLinePoint[];
    ema_20: ChartLinePoint[];
    bollinger_upper_20: ChartLinePoint[];
    bollinger_lower_20: ChartLinePoint[];
    rsi_14: ChartLinePoint[];
    macd_line: ChartLinePoint[];
    macd_signal: ChartLinePoint[];
    macd_histogram: ChartLinePoint[];
    atr_14: ChartLinePoint[];
  };
}

export interface ChartCandle {
  time: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number | null;
}

export interface ChartLinePoint {
  time: number;
  value: number;
}

export interface DatasetOptions {
  instrument: string;
  instrument_type: 'equity' | 'index' | 'unknown';
  source: string;
  interval_seconds: number;
  synthetic: boolean;
}

export interface BacktestOptions {
  initial_capital: number;
  fee_bps: number;
  slippage_bps: number;
  max_quantity: number;
  max_exposure: number;
  out_of_sample_fraction: number;
  buy_and_hold_benchmark: boolean;
}

export interface BaselineAnalysisOptions {
  exchange: string;
  currency: string;
  forecast_steps: number;
  folds: number;
  minimum_training_bars: number;
  test_bars_per_fold: number;
  horizons_bars: number[];
}

export interface ForecastMetrics {
  samples: number;
  mae: number;
  rmse: number;
  directional_accuracy: number;
}

export interface BaselineAnalysisReport {
  forecast: {
    instrument_id: { exchange: string; symbol: string };
    currency: string;
    origin_timestamp_utc: number;
    generated_at_utc: number;
    horizon: { steps: number; interval_seconds: number };
    model_id: string;
    model_version: string;
    source: { source_name: string; freshness: string };
    limitations: string[];
    points: Array<{ timestamp_utc: number; point_estimate: number | null; quantiles: unknown[] }>;
  };
  walk_forward: {
    model_id: string;
    model_version: string;
    folds: Array<{
      fold: number;
      training_bars_at_start: number;
      first_test_timestamp: number;
      last_test_timestamp: number;
      random_walk: ForecastMetrics;
      historical_drift: ForecastMetrics;
    }>;
    random_walk: ForecastMetrics;
    historical_drift: ForecastMetrics;
    horizon_evaluations: Array<{
      horizon_bars: number;
      folds: Array<{
        fold: number;
        training_bars_at_start: number;
        first_target_timestamp: number;
        last_target_timestamp: number;
        random_walk: ForecastMetrics;
        historical_drift: ForecastMetrics;
      }>;
      random_walk: ForecastMetrics;
      historical_drift: ForecastMetrics;
    }>;
    warnings: string[];
  };
  observed_closes: Array<{ timestamp: number; close: number }>;
  data_class: string;
  interval_seconds: number;
  instrument: string;
}

export interface LocalAnalysisReport {
  schema_version: number;
  mode: 'local_deterministic';
  intent: string;
  selected_instrument: string;
  data_class: string;
  answer_text: string;
  claims: Array<{
    id: string;
    text: string;
    kind: string;
    evidence_ids: string[];
    counter_evidence_ids: string[];
    assumptions: string[];
    uncertainty: string;
  }>;
  evidence: Array<{
    id: string;
    kind: string;
    label: string;
    value: number;
    unit: string;
    period_start_utc: number;
    period_end_utc: number;
    source_id: string;
    source_name: string;
    as_of_utc: number;
  }>;
  citations: Array<{
    evidence_id: string;
    source_id: string;
    source_name: string;
    as_of_utc: number;
  }>;
  assumptions: string[];
  uncertainty: string;
  missing_data: string[];
  validation: { valid: boolean; warnings: string[] };
  audit: {
    analyst_version: string;
    context_fingerprint: string;
    output_fingerprint: string;
    calculation_versions: string[];
    persisted: boolean;
  };
  disclaimer: string;
}

export interface NumericMetrics {
  submitted_orders: number;
  filled_orders: number;
  rejected_orders: number;
  closed_trades: number;
  winning_trades: number;
  losing_trades: number;
  gross_profit: number;
  gross_loss: number;
  gross_pnl: number;
  fees: number;
  slippage: number;
  net_pnl: number;
  ending_equity: number;
  return_fraction: number;
  win_rate: number | null;
  average_winner: number | null;
  average_loser: number | null;
  profit_factor: number | null;
  expectancy: number | null;
  max_drawdown_amount: number;
  max_drawdown_fraction: number;
  open_positions: number;
  unrealized_gross_pnl: number;
  exposure_fraction: number;
  turnover: number;
}

export interface BacktestReport {
  instrument: string;
  instrument_type: string;
  source: string;
  data_class: string;
  interval_seconds: number;
  candle_count: number;
  timestamp_timezone: string;
  timestamp_meaning: string;
  strategy_name: string;
  strategy_version: string;
  dataset_fingerprint: string;
  evaluation: {
    test_start_index: number;
    first_timestamp: string;
    last_timestamp: string;
  };
  config: BacktestOptions & { end_of_data_policy: string };
  metrics: NumericMetrics;
  cash_benchmark: Record<string, number | string>;
  buy_and_hold_benchmark: Record<string, number | string> | null;
  orders: Array<Record<string, unknown>>;
  trades: Array<Record<string, unknown>>;
  equity_curve: Array<Record<string, unknown>>;
  assumptions: string[];
}

export interface RunHistoryItem {
  id: string;
  created_at: string;
  instrument: string;
  data_class: string;
  dataset_fingerprint: string;
  net_pnl: number;
  return_fraction: number;
  closed_trades: number;
}
