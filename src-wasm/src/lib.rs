mod analyst;
mod chart;

use serde::Deserialize;
use serde_json::{Value, json};
use wasm_bindgen::prelude::*;

const MAX_CSV_BYTES: usize = 10 * 1024 * 1024;

#[derive(Debug, Deserialize)]
struct DatasetOptions {
    instrument: String,
    instrument_type: String,
    source: String,
    interval_seconds: u64,
    synthetic: bool,
}

#[derive(Debug, Deserialize)]
struct RunOptions {
    initial_capital: f64,
    fee_bps: f64,
    slippage_bps: f64,
    max_quantity: u64,
    max_exposure: f64,
    out_of_sample_fraction: f64,
    buy_and_hold_benchmark: bool,
}

#[derive(Debug, Deserialize)]
struct FeatureFlags {
    csv_import: bool,
    data_validation: bool,
    technical_indicators: bool,
    signal_generation: bool,
    backtesting: bool,
}

#[derive(Debug, Deserialize)]
struct BaselineAnalysisOptions {
    exchange: String,
    currency: String,
    forecast_steps: u32,
    folds: usize,
    minimum_training_bars: usize,
    test_bars_per_fold: usize,
    #[serde(default = "default_evaluation_horizons")]
    horizons_bars: Vec<usize>,
}

fn default_evaluation_horizons() -> Vec<usize> {
    vec![1]
}

#[derive(Debug, Deserialize)]
struct BaselineAnalysisFeatures {
    csv_import: bool,
    data_validation: bool,
    naive_forecast: bool,
    walk_forward: bool,
}

fn check_csv_size(csv: &str) -> Result<(), String> {
    if csv.len() > MAX_CSV_BYTES {
        return Err("CSV exceeds the 10 MiB in-memory analysis limit".to_owned());
    }
    Ok(())
}

fn metadata(options: &DatasetOptions) -> Result<tredalix_ai::data::DatasetMetadata, String> {
    if options.instrument.trim().is_empty() {
        return Err("Instrument identifier cannot be empty".to_owned());
    }
    if options.source.trim().is_empty() {
        return Err("Data source description cannot be empty".to_owned());
    }
    if options.interval_seconds == 0 || options.interval_seconds > i64::MAX as u64 {
        return Err("Interval must be in the supported positive timestamp range".to_owned());
    }
    let instrument_type = match options.instrument_type.as_str() {
        "equity" => tredalix_ai::data::InstrumentType::CashEquity,
        "index" => tredalix_ai::data::InstrumentType::Index,
        "unknown" => tredalix_ai::data::InstrumentType::Unknown,
        _ => return Err("Instrument type must be equity, index, or unknown".to_owned()),
    };
    Ok(tredalix_ai::data::DatasetMetadata {
        instrument: options.instrument.trim().to_owned(),
        instrument_type,
        source: options.source.trim().to_owned(),
        interval_seconds: Some(options.interval_seconds),
        data_class: if options.synthetic {
            tredalix_ai::data::DataClass::Synthetic
        } else {
            tredalix_ai::data::DataClass::Unknown
        },
        timestamp_timezone: tredalix_ai::data::TimestampTimezone::AsiaKolkata,
        timestamp_meaning: tredalix_ai::data::TimestampMeaning::CandleOpen,
    })
}

fn parse_dataset(
    csv: &str,
    options: &DatasetOptions,
) -> Result<tredalix_ai::data::Dataset, String> {
    check_csv_size(csv)?;
    let candles = tredalix_ai::data::parse_csv(csv)?;
    let quality = tredalix_ai::data::validate_dataset(&candles, options.interval_seconds.into())?;
    Ok(tredalix_ai::data::Dataset {
        metadata: metadata(options)?,
        candles,
        quality,
    })
}

fn inspect(csv: &str, options: &DatasetOptions, indicators_enabled: bool) -> Result<Value, String> {
    let dataset = parse_dataset(csv, options)?;
    let chart = chart::build_chart(&dataset.candles, indicators_enabled)?;
    Ok(json!({
        "instrument": dataset.metadata.instrument,
        "instrument_type": dataset.metadata.instrument_type.label(),
        "source": dataset.metadata.source,
        "data_class": dataset.metadata.data_class.label(),
        "interval_seconds": dataset.metadata.interval_seconds,
        "timestamp_timezone": dataset.metadata.timestamp_timezone.label(),
        "timestamp_meaning": "candle open",
        "row_count": dataset.quality.row_count,
        "first_timestamp": tredalix_ai::data::format_timestamp(dataset.quality.first_timestamp),
        "last_timestamp": tredalix_ai::data::format_timestamp(dataset.quality.last_timestamp),
        "first_timestamp_epoch": dataset.quality.first_timestamp,
        "last_timestamp_epoch": dataset.quality.last_timestamp,
        "gaps": {
            "observed": dataset.quality.gaps.observed_gaps,
            "unobserved_intervals": dataset.quality.gaps.unobserved_intervals,
            "irregular": dataset.quality.gaps.irregular_gaps,
            "session_calendar_configured": false
        },
        "preview": dataset.candles.iter().take(12).map(|candle| json!({
            "timestamp": tredalix_ai::data::format_timestamp(candle.timestamp),
            "open": candle.open,
            "high": candle.high,
            "low": candle.low,
            "close": candle.close,
            "volume": candle.volume
        })).collect::<Vec<_>>(),
        "chart": chart
    }))
}

fn metrics_json(metrics: &tredalix_ai::backtest::PerformanceMetrics) -> Value {
    json!({
        "submitted_orders": metrics.submitted_orders,
        "filled_orders": metrics.filled_orders,
        "rejected_orders": metrics.rejected_orders,
        "closed_trades": metrics.closed_trades,
        "winning_trades": metrics.winning_trades,
        "losing_trades": metrics.losing_trades,
        "gross_profit": metrics.gross_profit,
        "gross_loss": metrics.gross_loss,
        "gross_pnl": metrics.gross_pnl,
        "fees": metrics.fees,
        "slippage": metrics.slippage,
        "net_pnl": metrics.net_pnl,
        "ending_equity": metrics.ending_equity,
        "return_fraction": metrics.return_fraction,
        "win_rate": metrics.win_rate,
        "average_winner": metrics.average_winner,
        "average_loser": metrics.average_loser,
        "profit_factor": metrics.profit_factor,
        "expectancy": metrics.expectancy,
        "max_drawdown_amount": metrics.max_drawdown_amount,
        "max_drawdown_fraction": metrics.max_drawdown_fraction,
        "open_positions": metrics.open_positions,
        "unrealized_gross_pnl": metrics.unrealized_gross_pnl,
        "exposure_fraction": metrics.exposure_fraction,
        "turnover": metrics.turnover
    })
}

fn benchmark_json(benchmark: &tredalix_ai::backtest::BenchmarkMetrics) -> Value {
    json!({
        "name": benchmark.name,
        "ending_equity": benchmark.ending_equity,
        "return_fraction": benchmark.return_fraction,
        "max_drawdown_fraction": benchmark.max_drawdown_fraction,
        "fees": benchmark.fees,
        "exposure_fraction": benchmark.exposure_fraction
    })
}

fn run(
    csv: &str,
    dataset_options: &DatasetOptions,
    run_options: &RunOptions,
    features: &FeatureFlags,
) -> Result<Value, String> {
    if !features.backtesting {
        return Err("Backtesting is disabled in Feature Manager".to_owned());
    }
    if !features.technical_indicators {
        return Err("Technical indicators are disabled in Feature Manager".to_owned());
    }
    if !features.signal_generation {
        return Err("Signal generation is disabled in Feature Manager".to_owned());
    }
    let dataset = parse_dataset(csv, dataset_options)?;
    let config = tredalix_ai::backtest::BacktestConfig {
        initial_capital: run_options.initial_capital,
        fee_bps: run_options.fee_bps,
        slippage_bps: run_options.slippage_bps,
        max_quantity: run_options.max_quantity,
        max_exposure: run_options.max_exposure,
        out_of_sample_fraction: run_options.out_of_sample_fraction,
        buy_and_hold_benchmark: run_options.buy_and_hold_benchmark,
        ..tredalix_ai::backtest::BacktestConfig::default()
    };
    config.validate()?;

    let result = tredalix_ai::backtest::run_backtest(
        &dataset,
        &tredalix_ai::backtest::SmaRsiStrategy,
        &config,
    )?;
    Ok(json!({
        "instrument": result.instrument,
        "instrument_type": result.instrument_type,
        "source": result.source,
        "data_class": result.data_class,
        "interval_seconds": result.interval_seconds,
        "candle_count": result.candle_count,
        "timestamp_timezone": result.timestamp_timezone,
        "timestamp_meaning": result.timestamp_meaning,
        "strategy_name": result.strategy_name,
        "strategy_version": result.strategy_version,
        "dataset_fingerprint": result.dataset_fingerprint,
        "evaluation": {
            "test_start_index": result.test_start_index,
            "first_timestamp": tredalix_ai::data::format_timestamp(result.first_test_timestamp),
            "last_timestamp": tredalix_ai::data::format_timestamp(result.last_test_timestamp)
        },
        "config": {
            "initial_capital": config.initial_capital,
            "fee_bps": config.fee_bps,
            "slippage_bps": config.slippage_bps,
            "max_quantity": config.max_quantity,
            "max_exposure": config.max_exposure,
            "out_of_sample_fraction": config.out_of_sample_fraction,
            "buy_and_hold_benchmark": config.buy_and_hold_benchmark,
            "end_of_data_policy": "mark_to_market"
        },
        "metrics": metrics_json(&result.metrics),
        "cash_benchmark": benchmark_json(&result.cash_benchmark),
        "buy_and_hold_benchmark": result.buy_and_hold_benchmark.as_ref().map(benchmark_json),
        "orders": result.orders.iter().map(|order| json!({
            "id": order.id,
            "decision_timestamp": tredalix_ai::data::format_timestamp(order.decision_timestamp),
            "execution_timestamp": order.execution_timestamp.map(tredalix_ai::data::format_timestamp),
            "side": order.side.label(),
            "quantity": order.quantity,
            "status": order.status.label(),
            "fill_price": order.fill_price,
            "fee": order.fee,
            "slippage": order.slippage,
            "rejection_reason": order.rejection_reason
        })).collect::<Vec<_>>(),
        "trades": result.trades.iter().map(|trade| json!({
            "entry_timestamp": tredalix_ai::data::format_timestamp(trade.entry_timestamp),
            "exit_timestamp": tredalix_ai::data::format_timestamp(trade.exit_timestamp),
            "quantity": trade.quantity,
            "entry_price": trade.entry_price,
            "exit_price": trade.exit_price,
            "gross_pnl": trade.gross_pnl,
            "fees": trade.fees,
            "slippage": trade.slippage,
            "net_pnl": trade.net_pnl,
            "exit_reason": trade.exit_reason
        })).collect::<Vec<_>>(),
        "equity_curve": result.equity_curve.iter().map(|point| json!({
            "timestamp": tredalix_ai::data::format_timestamp(point.timestamp),
            "cash": point.cash,
            "position_value": point.position_value,
            "equity": point.equity,
            "peak_equity": point.peak_equity,
            "drawdown": point.drawdown,
            "drawdown_amount": point.drawdown_amount
        })).collect::<Vec<_>>(),
        "assumptions": [
            "No live orders are sent.",
            "A signal uses data through candle close and executes at the next candle open.",
            "Fees are generic estimates, not a date/provider-specific Indian charges model.",
            "Out-of-sample evaluation is a single chronological trailing holdout, not repeated walk-forward validation.",
            "Exchange session and holiday calendar is not configured.",
            "Backtest results do not guarantee future performance."
        ]
    }))
}

fn baseline_analysis(
    csv: &str,
    dataset_options: &DatasetOptions,
    analysis_options: &BaselineAnalysisOptions,
) -> Result<Value, String> {
    let dataset = parse_dataset(csv, dataset_options)?;
    let interval_seconds = dataset
        .metadata
        .interval_seconds
        .ok_or_else(|| "A declared interval is required for baseline analysis".to_owned())?;
    let last = dataset
        .candles
        .last()
        .ok_or_else(|| "Baseline analysis requires at least two validated bars".to_owned())?;
    let generated_at_utc = last
        .timestamp
        .checked_add(
            i64::try_from(interval_seconds)
                .map_err(|_| "Dataset interval exceeds timestamp range".to_owned())?,
        )
        .ok_or_else(|| "Baseline analysis timestamp overflows".to_owned())?;
    let instrument_id = tredalix_ai::domain::InstrumentId::new(
        &analysis_options.exchange,
        &dataset.metadata.instrument,
    )?;
    let currency = tredalix_ai::domain::CurrencyCode::new(&analysis_options.currency)?;
    let source = tredalix_ai::forecasting::local_csv_source(
        &dataset.metadata.source,
        dataset.metadata.data_class == tredalix_ai::data::DataClass::Synthetic,
        generated_at_utc,
    );
    let forecast = tredalix_ai::forecasting::naive_random_walk(
        &dataset.candles,
        instrument_id,
        currency,
        interval_seconds,
        analysis_options.forecast_steps,
        generated_at_utc,
        source,
    )?;
    let walk_forward = tredalix_ai::evaluation::evaluate_walk_forward_with_horizons(
        &dataset.candles,
        interval_seconds,
        tredalix_ai::evaluation::WalkForwardConfig {
            folds: analysis_options.folds,
            minimum_training_bars: analysis_options.minimum_training_bars,
            test_bars_per_fold: analysis_options.test_bars_per_fold,
        },
        &analysis_options.horizons_bars,
    )?;
    let stride = dataset.candles.len().div_ceil(500);
    let mut observed_closes = dataset
        .candles
        .iter()
        .step_by(stride)
        .map(|candle| {
            json!({
                "timestamp": candle.timestamp,
                "close": candle.close
            })
        })
        .collect::<Vec<_>>();
    if let Some(last_point) = dataset.candles.last()
        && observed_closes
            .last()
            .is_none_or(|point| point["timestamp"] != last_point.timestamp)
    {
        observed_closes.push(json!({
            "timestamp": last_point.timestamp,
            "close": last_point.close
        }));
    }
    Ok(json!({
        "forecast": forecast,
        "walk_forward": walk_forward,
        "observed_closes": observed_closes,
        "data_class": dataset.metadata.data_class.label(),
        "interval_seconds": interval_seconds,
        "instrument": dataset.metadata.instrument
    }))
}

fn to_json(value: Value) -> Result<String, String> {
    serde_json::to_string(&value).map_err(|error| format!("Cannot encode result: {error}"))
}

pub fn inspect_csv_json(
    csv: &str,
    options_json: &str,
    csv_import_enabled: bool,
    data_validation_enabled: bool,
    indicators_enabled: bool,
) -> Result<String, String> {
    if !csv_import_enabled {
        return Err("CSV import is disabled in Feature Manager".to_owned());
    }
    if !data_validation_enabled {
        return Err("Data validation is disabled in Feature Manager".to_owned());
    }
    let options: DatasetOptions = serde_json::from_str(options_json)
        .map_err(|error| format!("Invalid dataset options: {error}"))?;
    to_json(inspect(csv, &options, indicators_enabled)?)
}

pub fn backtest_csv_json(
    csv: &str,
    dataset_options_json: &str,
    run_options_json: &str,
    feature_flags_json: &str,
) -> Result<String, String> {
    let features: FeatureFlags = serde_json::from_str(feature_flags_json)
        .map_err(|error| format!("Invalid feature flags: {error}"))?;
    if !features.csv_import {
        return Err("CSV import is disabled in Feature Manager".to_owned());
    }
    if !features.data_validation {
        return Err("Data validation is disabled in Feature Manager".to_owned());
    }
    let dataset_options: DatasetOptions = serde_json::from_str(dataset_options_json)
        .map_err(|error| format!("Invalid dataset options: {error}"))?;
    let run_options: RunOptions = serde_json::from_str(run_options_json)
        .map_err(|error| format!("Invalid backtest options: {error}"))?;
    to_json(run(csv, &dataset_options, &run_options, &features)?)
}

pub fn baseline_analysis_csv_json(
    csv: &str,
    dataset_options_json: &str,
    analysis_options_json: &str,
    feature_flags_json: &str,
) -> Result<String, String> {
    let features: BaselineAnalysisFeatures = serde_json::from_str(feature_flags_json)
        .map_err(|error| format!("Invalid analysis feature flags: {error}"))?;
    if !features.csv_import {
        return Err("CSV import is disabled in Feature Manager".to_owned());
    }
    if !features.data_validation {
        return Err("Data validation is disabled in Feature Manager".to_owned());
    }
    if !features.naive_forecast {
        return Err("Naive baseline forecasts are disabled in Feature Manager".to_owned());
    }
    if !features.walk_forward {
        return Err("Walk-forward evaluation is disabled in Feature Manager".to_owned());
    }
    let dataset_options: DatasetOptions = serde_json::from_str(dataset_options_json)
        .map_err(|error| format!("Invalid dataset options: {error}"))?;
    let analysis_options: BaselineAnalysisOptions = serde_json::from_str(analysis_options_json)
        .map_err(|error| format!("Invalid baseline analysis options: {error}"))?;
    check_csv_size(csv)?;
    to_json(baseline_analysis(csv, &dataset_options, &analysis_options)?)
}

#[wasm_bindgen]
pub fn inspect_csv(
    csv: &str,
    options_json: &str,
    csv_import_enabled: bool,
    data_validation_enabled: bool,
    indicators_enabled: bool,
) -> Result<String, JsError> {
    inspect_csv_json(
        csv,
        options_json,
        csv_import_enabled,
        data_validation_enabled,
        indicators_enabled,
    )
    .map_err(|error| JsError::new(&error))
}

#[wasm_bindgen]
pub fn run_backtest(
    csv: &str,
    dataset_options_json: &str,
    run_options_json: &str,
    feature_flags_json: &str,
) -> Result<String, JsError> {
    backtest_csv_json(
        csv,
        dataset_options_json,
        run_options_json,
        feature_flags_json,
    )
    .map_err(|error| JsError::new(&error))
}

#[wasm_bindgen]
pub fn run_baseline_analysis(
    csv: &str,
    dataset_options_json: &str,
    analysis_options_json: &str,
    feature_flags_json: &str,
) -> Result<String, JsError> {
    baseline_analysis_csv_json(
        csv,
        dataset_options_json,
        analysis_options_json,
        feature_flags_json,
    )
    .map_err(|error| JsError::new(&error))
}

#[cfg(test)]
mod tests {
    use super::{backtest_csv_json, baseline_analysis_csv_json, inspect_csv_json};

    const CSV: &str = "timestamp,open,high,low,close,volume\n\
        2026-10-20 09:15,100,101,99,100,1000\n\
        2026-10-20 09:16,100,101,99,101,1000\n\
        2026-10-20 09:17,110,111,109,110,1000\n\
        2026-10-20 09:18,120,121,119,120,1000\n";
    const DATASET: &str = r#"{"instrument":"SBIN","instrument_type":"equity","source":"synthetic test","interval_seconds":60,"synthetic":true}"#;
    const CONFIG: &str = r#"{"initial_capital":1000.0,"fee_bps":1.0,"slippage_bps":1.0,"max_quantity":10,"max_exposure":1000.0,"out_of_sample_fraction":1.0,"buy_and_hold_benchmark":true}"#;
    const FEATURES: &str = r#"{"csv_import":true,"data_validation":true,"technical_indicators":true,"signal_generation":true,"backtesting":true}"#;
    const BASELINE_OPTIONS: &str = r#"{"exchange":"NSE","currency":"INR","forecast_steps":2,"folds":1,"minimum_training_bars":2,"test_bars_per_fold":2,"horizons_bars":[1]}"#;
    const BASELINE_FEATURES: &str =
        r#"{"csv_import":true,"data_validation":true,"naive_forecast":true,"walk_forward":true}"#;

    #[test]
    fn exports_rust_validation_and_backtest_results_as_json() {
        let inspected = inspect_csv_json(CSV, DATASET, true, true, true).unwrap();
        let inspection: Value = serde_json::from_str(&inspected).unwrap();
        assert_eq!(inspection["row_count"], 4);
        assert_eq!(inspection["data_class"], "SYNTHETIC / TEST");
        assert_eq!(inspection["chart"]["candles"].as_array().unwrap().len(), 4);
        assert_eq!(inspection["chart"]["sma_20"].as_array().unwrap().len(), 0);
        let rows = (0..30)
            .map(|index| {
                let minute = 15 + index;
                format!(
                    "2026-10-20 {:02}:{:02},{},{},{},{},10",
                    9 + minute / 60,
                    minute % 60,
                    100 + index,
                    101 + index,
                    99 + index,
                    100 + index
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let long_csv = format!("timestamp,open,high,low,close,volume\n{rows}");
        let disabled: Value =
            serde_json::from_str(&inspect_csv_json(&long_csv, DATASET, true, true, false).unwrap())
                .unwrap();
        assert_eq!(disabled["chart"]["sma_20"].as_array().unwrap().len(), 0);
        let enabled: Value =
            serde_json::from_str(&inspect_csv_json(&long_csv, DATASET, true, true, true).unwrap())
                .unwrap();
        assert_eq!(enabled["chart"]["sma_20"].as_array().unwrap().len(), 11);

        let result = backtest_csv_json(CSV, DATASET, CONFIG, FEATURES).unwrap();
        let report: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(report["strategy_name"], "SMA9-SMA21-RSI14");
        assert_eq!(report["config"]["initial_capital"], 1000.0);
        assert!(report["equity_curve"].as_array().unwrap().len() >= 2);
        assert!(report["buy_and_hold_benchmark"].is_object());
    }

    #[test]
    fn exports_baseline_forecast_and_chronological_walk_forward_results() {
        let result =
            baseline_analysis_csv_json(CSV, DATASET, BASELINE_OPTIONS, BASELINE_FEATURES).unwrap();
        let report: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(report["forecast"]["model_id"], "naive_random_walk");
        assert_eq!(report["forecast"]["points"][0]["point_estimate"], 120.0);
        assert_eq!(report["walk_forward"]["random_walk"]["samples"], 2);
        assert_eq!(
            report["walk_forward"]["horizon_evaluations"][0]["horizon_bars"],
            1
        );
        assert_eq!(report["observed_closes"].as_array().unwrap().len(), 4);
    }

    #[test]
    fn baseline_features_are_enforced_at_the_rust_boundary() {
        let disabled = r#"{"csv_import":true,"data_validation":true,"naive_forecast":false,"walk_forward":true}"#;
        assert!(
            baseline_analysis_csv_json(CSV, DATASET, BASELINE_OPTIONS, disabled)
                .unwrap_err()
                .contains("Naive baseline forecasts are disabled")
        );
        let missing_identity = r#"{"exchange":"","currency":"INR","forecast_steps":2,"folds":1,"minimum_training_bars":2,"test_bars_per_fold":2,"horizons_bars":[1]}"#;
        assert!(
            baseline_analysis_csv_json(CSV, DATASET, missing_identity, BASELINE_FEATURES)
                .unwrap_err()
                .contains("Exchange code")
        );
    }

    #[test]
    fn feature_toggles_block_operations_at_the_rust_boundary() {
        assert!(
            inspect_csv_json(CSV, DATASET, false, true, true)
                .unwrap_err()
                .contains("CSV import is disabled")
        );
        assert!(
            inspect_csv_json(CSV, DATASET, true, false, true)
                .unwrap_err()
                .contains("Data validation is disabled")
        );
        let disabled_backtest = r#"{"csv_import":true,"data_validation":true,"technical_indicators":true,"signal_generation":true,"backtesting":false}"#;
        let disabled_indicators = r#"{"csv_import":true,"data_validation":true,"technical_indicators":false,"signal_generation":true,"backtesting":true}"#;
        let disabled_signals = r#"{"csv_import":true,"data_validation":true,"technical_indicators":true,"signal_generation":false,"backtesting":true}"#;
        assert!(
            backtest_csv_json(CSV, DATASET, CONFIG, disabled_backtest)
                .unwrap_err()
                .contains("Backtesting is disabled")
        );
        assert!(
            backtest_csv_json(CSV, DATASET, CONFIG, disabled_indicators)
                .unwrap_err()
                .contains("Technical indicators are disabled")
        );
        assert!(
            backtest_csv_json(CSV, DATASET, CONFIG, disabled_signals)
                .unwrap_err()
                .contains("Signal generation is disabled")
        );
    }

    #[test]
    fn rejects_oversized_csv_and_invalid_metadata() {
        assert!(
            inspect_csv_json(&" ".repeat(10 * 1024 * 1024 + 1), DATASET, true, true, true)
                .unwrap_err()
                .contains("10 MiB")
        );
        let invalid = r#"{"instrument":"","instrument_type":"equity","source":"fixture","interval_seconds":60,"synthetic":true}"#;
        assert!(
            inspect_csv_json(CSV, invalid, true, true, true)
                .unwrap_err()
                .contains("Instrument identifier")
        );
    }

    use serde_json::Value;
}
