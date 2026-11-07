use crate::data::Dataset;

use super::config::BacktestConfig;
use super::execution::simulate_market_fill;
use super::metrics::{BenchmarkMetrics, MetricInputs, PerformanceMetrics, calculate_metrics};
use super::portfolio::{OrderRecord, OrderStatus, Portfolio, Side};
use super::risk::position_quantity;
use super::strategy::{Action, Strategy};

#[derive(Debug)]
pub struct BacktestResult {
    pub instrument: String,
    pub instrument_type: String,
    pub source: String,
    pub data_class: String,
    pub interval_seconds: Option<u64>,
    pub candle_count: usize,
    pub timestamp_timezone: String,
    pub timestamp_meaning: String,
    pub strategy_name: String,
    pub strategy_version: String,
    pub dataset_fingerprint: String,
    pub config: BacktestConfig,
    pub test_start_index: usize,
    pub first_test_timestamp: i64,
    pub last_test_timestamp: i64,
    pub metrics: PerformanceMetrics,
    pub cash_benchmark: BenchmarkMetrics,
    pub buy_and_hold_benchmark: Option<BenchmarkMetrics>,
    pub orders: Vec<OrderRecord>,
    pub trades: Vec<super::portfolio::TradeRecord>,
    pub equity_curve: Vec<super::portfolio::EquityPoint>,
}

pub fn run_backtest(
    dataset: &Dataset,
    strategy: &impl Strategy,
    config: &BacktestConfig,
) -> Result<BacktestResult, String> {
    config.validate()?;
    if dataset.candles.is_empty() {
        return Err("Cannot backtest an empty dataset".to_owned());
    }
    if dataset.metadata.instrument.trim().is_empty() {
        return Err("Instrument identifier cannot be empty".to_owned());
    }
    let interval = dataset
        .metadata
        .interval_seconds
        .ok_or_else(|| "A declared dataset interval is required for backtesting".to_owned())?;
    if interval == 0 || interval > i64::MAX as u64 {
        return Err("Dataset interval exceeds the supported timestamp range".to_owned());
    }
    crate::data::validate_dataset(&dataset.candles, Some(interval))?;
    if config.buy_and_hold_benchmark
        && dataset.metadata.instrument_type != crate::data::InstrumentType::CashEquity
    {
        return Err(
            "Buy-and-hold execution benchmark requires instrument type CASH EQUITY; do not treat index values as executable"
                .to_owned(),
        );
    }
    let test_start = config.test_start_index(dataset.candles.len());
    if test_start >= dataset.candles.len() {
        return Err("Out-of-sample window contains no candles".to_owned());
    }

    let candles = &dataset.candles;
    let mut portfolio = Portfolio::new(config.initial_capital);
    portfolio.mark(candles[test_start].timestamp, candles[test_start].close)?;

    for decision_index in test_start..candles.len() {
        let action = strategy.try_action(&candles[..=decision_index])?;
        if action == Action::Hold {
            if let Some(next) = candles.get(decision_index + 1) {
                portfolio.mark(next.timestamp, next.close)?;
            }
            continue;
        }
        let decision = &candles[decision_index];
        let order_id = portfolio.orders.len() + 1;
        if decision_index + 1 >= candles.len() {
            record_rejection(
                &mut portfolio,
                order_id,
                decision.timestamp,
                None,
                action_side(action),
                0,
                "no next candle available for execution",
            );
            continue;
        }

        let next = &candles[decision_index + 1];
        if let Some(interval) = dataset.metadata.interval_seconds
            && next.timestamp - decision.timestamp > interval as i64
        {
            record_rejection(
                &mut portfolio,
                order_id,
                decision.timestamp,
                Some(next.timestamp),
                action_side(action),
                0,
                "next candle exceeds the declared interval; session status is unknown",
            );
            portfolio.mark(next.timestamp, next.close)?;
            continue;
        }

        match action {
            Action::EnterLong => {
                if portfolio.position.is_some() {
                    record_rejection(
                        &mut portfolio,
                        order_id,
                        decision.timestamp,
                        Some(next.timestamp),
                        Side::Buy,
                        0,
                        "position already open; duplicate entry prevented",
                    );
                } else {
                    match position_quantity(next.open, config, &portfolio) {
                        Ok(quantity) => {
                            match simulate_market_fill(next.open, Side::Buy, quantity, config) {
                                Ok(fill) => match portfolio.enter(
                                    quantity,
                                    next.timestamp,
                                    fill.price,
                                    fill.fee,
                                    fill.slippage,
                                ) {
                                    Ok(()) => portfolio.orders.push(OrderRecord {
                                        id: order_id,
                                        decision_timestamp: decision.timestamp,
                                        execution_timestamp: Some(next.timestamp),
                                        side: Side::Buy,
                                        quantity,
                                        status: OrderStatus::Filled,
                                        fill_price: Some(fill.price),
                                        fee: fill.fee,
                                        slippage: fill.slippage,
                                        rejection_reason: None,
                                    }),
                                    Err(reason) => record_rejection(
                                        &mut portfolio,
                                        order_id,
                                        decision.timestamp,
                                        Some(next.timestamp),
                                        Side::Buy,
                                        quantity,
                                        &reason,
                                    ),
                                },
                                Err(reason) => record_rejection(
                                    &mut portfolio,
                                    order_id,
                                    decision.timestamp,
                                    Some(next.timestamp),
                                    Side::Buy,
                                    quantity,
                                    &reason,
                                ),
                            }
                        }
                        Err(reason) => record_rejection(
                            &mut portfolio,
                            order_id,
                            decision.timestamp,
                            Some(next.timestamp),
                            Side::Buy,
                            config.max_quantity,
                            &reason,
                        ),
                    }
                }
            }
            Action::ExitLong => {
                if let Some(position) = portfolio.position {
                    let quantity = position.quantity;
                    match simulate_market_fill(next.open, Side::Sell, quantity, config) {
                        Ok(fill) => match portfolio.exit(
                            next.timestamp,
                            fill.price,
                            fill.fee,
                            fill.slippage,
                            "strategy exit",
                        ) {
                            Ok(()) => portfolio.orders.push(OrderRecord {
                                id: order_id,
                                decision_timestamp: decision.timestamp,
                                execution_timestamp: Some(next.timestamp),
                                side: Side::Sell,
                                quantity,
                                status: OrderStatus::Filled,
                                fill_price: Some(fill.price),
                                fee: fill.fee,
                                slippage: fill.slippage,
                                rejection_reason: None,
                            }),
                            Err(reason) => record_rejection(
                                &mut portfolio,
                                order_id,
                                decision.timestamp,
                                Some(next.timestamp),
                                Side::Sell,
                                quantity,
                                &reason,
                            ),
                        },
                        Err(reason) => record_rejection(
                            &mut portfolio,
                            order_id,
                            decision.timestamp,
                            Some(next.timestamp),
                            Side::Sell,
                            quantity,
                            &reason,
                        ),
                    }
                } else {
                    record_rejection(
                        &mut portfolio,
                        order_id,
                        decision.timestamp,
                        Some(next.timestamp),
                        Side::Sell,
                        0,
                        "no open position to exit",
                    );
                }
            }
            Action::Hold => {}
        }
        portfolio.mark(next.timestamp, next.close)?;
    }

    let last = candles.last().expect("nonempty checked above");
    let ending_equity = portfolio.equity(last.close);
    if !ending_equity.is_finite() {
        return Err("ending portfolio equity is not finite".to_owned());
    }
    let filled_orders = portfolio
        .orders
        .iter()
        .filter(|order| order.status == OrderStatus::Filled)
        .count();
    let rejected_orders = portfolio
        .orders
        .iter()
        .filter(|order| order.status == OrderStatus::Rejected)
        .count();
    let turnover = portfolio
        .orders
        .iter()
        .filter(|order| order.status == OrderStatus::Filled)
        .map(|order| order.fill_price.unwrap_or_default() * order.quantity as f64)
        .sum();
    let metrics = calculate_metrics(MetricInputs {
        initial_capital: config.initial_capital,
        ending_equity,
        trades: &portfolio.trades,
        curve: &portfolio.equity_curve,
        submitted_orders: portfolio.orders.len(),
        filled_orders,
        rejected_orders,
        fees: portfolio.fees,
        slippage: portfolio.slippage,
        open_positions: usize::from(portfolio.position.is_some()),
        unrealized_gross_pnl: portfolio.unrealized_gross_pnl(last.close),
        turnover,
    });
    let cash_benchmark = benchmark_cash(config.initial_capital);
    let buy_and_hold_benchmark = config
        .buy_and_hold_benchmark
        .then(|| benchmark_buy_and_hold(candles, test_start, config))
        .flatten();

    let _end_policy = config.end_of_data_policy;
    Ok(BacktestResult {
        instrument: dataset.metadata.instrument.clone(),
        instrument_type: dataset.metadata.instrument_type.label().to_owned(),
        source: dataset.metadata.source.clone(),
        data_class: dataset.metadata.data_class.label().to_owned(),
        interval_seconds: dataset.metadata.interval_seconds,
        candle_count: candles.len(),
        timestamp_timezone: dataset.metadata.timestamp_timezone.label().to_owned(),
        timestamp_meaning: "candle open".to_owned(),
        strategy_name: strategy.name().to_owned(),
        strategy_version: strategy.version().to_owned(),
        dataset_fingerprint: dataset_fingerprint(dataset),
        config: config.clone(),
        test_start_index: test_start,
        first_test_timestamp: candles[test_start].timestamp,
        last_test_timestamp: last.timestamp,
        metrics,
        cash_benchmark,
        buy_and_hold_benchmark,
        orders: portfolio.orders,
        trades: portfolio.trades,
        equity_curve: portfolio.equity_curve,
    })
}

fn dataset_fingerprint(dataset: &Dataset) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    let mut absorb = |bytes: &[u8]| {
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    };
    absorb(dataset.metadata.instrument.as_bytes());
    absorb(dataset.metadata.instrument_type.label().as_bytes());
    absorb(dataset.metadata.source.as_bytes());
    absorb(dataset.metadata.data_class.label().as_bytes());
    absorb(dataset.metadata.timestamp_timezone.label().as_bytes());
    absorb(b"candle-open");
    if let Some(interval) = dataset.metadata.interval_seconds {
        absorb(&interval.to_le_bytes());
    }
    for candle in &dataset.candles {
        absorb(&candle.timestamp.to_le_bytes());
        absorb(&candle.open.to_bits().to_le_bytes());
        absorb(&candle.high.to_bits().to_le_bytes());
        absorb(&candle.low.to_bits().to_le_bytes());
        absorb(&candle.close.to_bits().to_le_bytes());
        absorb(&candle.volume.unwrap_or(f64::NAN).to_bits().to_le_bytes());
    }
    format!("fnv1a64:{hash:016x}")
}

fn action_side(action: Action) -> Side {
    match action {
        Action::EnterLong | Action::Hold => Side::Buy,
        Action::ExitLong => Side::Sell,
    }
}

fn record_rejection(
    portfolio: &mut Portfolio,
    id: usize,
    decision_timestamp: i64,
    execution_timestamp: Option<i64>,
    side: Side,
    quantity: u64,
    reason: &str,
) {
    portfolio.orders.push(OrderRecord {
        id,
        decision_timestamp,
        execution_timestamp,
        side,
        quantity,
        status: OrderStatus::Rejected,
        fill_price: None,
        fee: 0.0,
        slippage: 0.0,
        rejection_reason: Some(reason.to_owned()),
    });
}

fn benchmark_cash(capital: f64) -> BenchmarkMetrics {
    BenchmarkMetrics {
        name: "cash".to_owned(),
        ending_equity: capital,
        return_fraction: 0.0,
        max_drawdown_fraction: 0.0,
        fees: 0.0,
        exposure_fraction: 0.0,
    }
}

fn benchmark_buy_and_hold(
    candles: &[crate::data::Candle],
    start: usize,
    config: &BacktestConfig,
) -> Option<BenchmarkMetrics> {
    let entry = candles.get(start + 1)?;
    let last = candles.last()?;
    let fill = simulate_market_fill(entry.open, Side::Buy, 1, config).ok()?;
    let per_unit = fill.price + fill.fee;
    let benchmark_budget = config.initial_capital.min(config.max_exposure);
    let quantity = (benchmark_budget / per_unit)
        .floor()
        .min(config.max_quantity as f64) as u64;
    if quantity == 0 {
        return None;
    }
    let total_entry_cost =
        fill.price * quantity as f64 + fill.price * quantity as f64 * config.fee_bps / 10_000.0;
    let ending_equity = config.initial_capital - total_entry_cost + last.close * quantity as f64;
    let mut peak = config.initial_capital;
    let mut max_drawdown: f64 = 0.0;
    for candle in candles.iter().skip(start + 1) {
        let equity = config.initial_capital - total_entry_cost + candle.close * quantity as f64;
        peak = peak.max(equity);
        if peak > 0.0 {
            max_drawdown = max_drawdown.max((peak - equity) / peak);
        }
    }
    Some(BenchmarkMetrics {
        name: "buy-and-hold (long-only; estimated costs)".to_owned(),
        ending_equity,
        return_fraction: (ending_equity - config.initial_capital) / config.initial_capital,
        max_drawdown_fraction: max_drawdown,
        fees: fill.fee * quantity as f64,
        exposure_fraction: candles
            .iter()
            .skip(start + 1)
            .map(|candle| {
                (candle.close * quantity as f64
                    / (config.initial_capital - total_entry_cost + candle.close * quantity as f64)
                        .max(f64::EPSILON))
                .min(1.0)
            })
            .sum::<f64>()
            / (candles.len() - start - 1) as f64,
    })
}

#[cfg(test)]
mod tests {
    use super::run_backtest;
    use crate::backtest::{Action, BacktestConfig, SmaRsiStrategy, Strategy};
    use crate::data::{
        Candle, DataClass, DataQualitySummary, Dataset, DatasetMetadata, inspect_gaps,
    };

    fn full_sample_config() -> BacktestConfig {
        BacktestConfig {
            out_of_sample_fraction: 1.0,
            ..BacktestConfig::default()
        }
    }

    struct AlwaysEnter;

    impl Strategy for AlwaysEnter {
        fn name(&self) -> &'static str {
            "always-enter"
        }
        fn version(&self) -> &'static str {
            "test"
        }
        fn action(&self, _: &[Candle]) -> Action {
            Action::EnterLong
        }
    }

    struct EnterOnFirstObservation;

    impl Strategy for EnterOnFirstObservation {
        fn name(&self) -> &'static str {
            "first-observation-only"
        }
        fn version(&self) -> &'static str {
            "test"
        }
        fn action(&self, history: &[Candle]) -> Action {
            if history.len() == 1 {
                Action::EnterLong
            } else {
                Action::Hold
            }
        }
    }

    struct NoSignals;

    impl Strategy for NoSignals {
        fn name(&self) -> &'static str {
            "no-signals"
        }
        fn version(&self) -> &'static str {
            "test"
        }
        fn action(&self, _: &[Candle]) -> Action {
            Action::Hold
        }
    }

    fn dataset(closes: &[f64]) -> Dataset {
        let candles: Vec<Candle> = closes
            .iter()
            .enumerate()
            .map(|(index, close)| Candle {
                timestamp: index as i64 * 60,
                open: *close,
                high: *close,
                low: *close,
                close: *close,
                volume: None,
            })
            .collect();
        let quality = DataQualitySummary {
            row_count: candles.len(),
            first_timestamp: candles[0].timestamp,
            last_timestamp: candles.last().unwrap().timestamp,
            gaps: inspect_gaps(&candles, Some(60)),
        };
        Dataset {
            metadata: DatasetMetadata {
                instrument: "TEST".to_owned(),
                instrument_type: crate::data::InstrumentType::CashEquity,
                source: "fixture".to_owned(),
                interval_seconds: Some(60),
                data_class: DataClass::Synthetic,
                timestamp_timezone: crate::data::TimestampTimezone::AsiaKolkata,
                timestamp_meaning: crate::data::TimestampMeaning::CandleOpen,
            },
            candles,
            quality,
        }
    }

    #[test]
    fn enters_on_next_candle_open_and_marks_open_position_at_end() {
        let data = dataset(&[100.0, 100.0, 110.0]);
        let config = full_sample_config();
        let result = run_backtest(&data, &AlwaysEnter, &config).unwrap();
        assert_eq!(result.orders[0].decision_timestamp, 0);
        assert_eq!(result.orders[0].execution_timestamp, Some(60));
        assert_eq!(result.orders[0].fill_price, Some(100.0));
        assert_eq!(result.metrics.open_positions, 1);
        assert_eq!(result.metrics.ending_equity, 101_000.0);
        assert_eq!(result.metrics.net_pnl, 1_000.0);
        assert!(
            result
                .orders
                .last()
                .unwrap()
                .rejection_reason
                .as_deref()
                .unwrap()
                .contains("no next candle")
        );
    }

    #[test]
    fn rejects_empty_dataset_without_panicking() {
        let mut data = dataset(&[100.0]);
        data.candles.clear();
        let error = run_backtest(&data, &AlwaysEnter, &BacktestConfig::default()).unwrap_err();
        assert!(error.contains("empty dataset"));
    }

    #[test]
    fn rejects_invalid_public_dataset_and_indicator_numeric_overflow() {
        let mut invalid = dataset(&[100.0, 101.0]);
        invalid.candles[1].high = 99.0;
        assert!(
            run_backtest(&invalid, &NoSignals, &BacktestConfig::default())
                .unwrap_err()
                .contains("OHLC")
        );

        let extreme = dataset(&[f64::MAX; 25]);
        let config = full_sample_config();
        assert!(
            run_backtest(&extreme, &SmaRsiStrategy, &config)
                .unwrap_err()
                .contains("non-finite")
        );
    }

    #[test]
    fn no_signal_run_keeps_cash_and_reports_no_orders() {
        let data = dataset(&[100.0, 101.0, 102.0]);
        let config = full_sample_config();
        let result = run_backtest(&data, &NoSignals, &config).unwrap();
        assert!(result.orders.is_empty());
        assert!(result.trades.is_empty());
        assert_eq!(result.metrics.ending_equity, config.initial_capital);
        assert_eq!(result.metrics.net_pnl, 0.0);
        assert_eq!(result.metrics.win_rate, None);
        assert_eq!(result.metrics.profit_factor, None);
    }

    #[test]
    fn rejects_execution_across_a_gap_and_prevents_lookahead() {
        let mut data = dataset(&[100.0, 100.0, 110.0]);
        data.candles[1].timestamp = 300;
        data.candles[2].timestamp = 360;
        data.quality.gaps = inspect_gaps(&data.candles, Some(60));
        let config = full_sample_config();
        let result = run_backtest(&data, &AlwaysEnter, &config).unwrap();
        assert_eq!(
            result.orders[0].status,
            super::super::portfolio::OrderStatus::Rejected
        );
        assert!(
            result.orders[0]
                .rejection_reason
                .as_deref()
                .unwrap()
                .contains("exceeds")
        );
    }

    #[test]
    fn future_candle_changes_do_not_rewrite_prior_decision_or_fill() {
        let original = dataset(&[100.0, 101.0, 102.0]);
        let altered_future = dataset(&[100.0, 101.0, 9_999.0]);
        let strategy = EnterOnFirstObservation;
        let config = full_sample_config();
        let first = run_backtest(&original, &strategy, &config).unwrap();
        let second = run_backtest(&altered_future, &strategy, &config).unwrap();
        assert_eq!(first.orders[0], second.orders[0]);
        assert_eq!(first.orders[0].decision_timestamp, 0);
        assert_eq!(first.orders[0].fill_price, Some(101.0));
    }

    #[test]
    fn out_of_sample_split_uses_prior_history_only_as_warmup() {
        let data = dataset(&[100.0, 100.0, 100.0, 100.0]);
        let config = BacktestConfig {
            out_of_sample_fraction: 0.5,
            ..BacktestConfig::default()
        };
        let result = run_backtest(&data, &AlwaysEnter, &config).unwrap();
        assert_eq!(result.test_start_index, 2);
        assert_eq!(result.first_test_timestamp, data.candles[2].timestamp);
        assert_eq!(
            result.orders[0].decision_timestamp,
            data.candles[2].timestamp
        );
    }

    #[test]
    fn cash_equity_buy_and_hold_benchmark_is_computed_with_costs() {
        let mut data = dataset(&[100.0, 100.0, 110.0]);
        data.metadata.instrument_type = crate::data::InstrumentType::CashEquity;
        let config = BacktestConfig {
            out_of_sample_fraction: 1.0,
            buy_and_hold_benchmark: true,
            ..BacktestConfig::default()
        };
        let result = run_backtest(&data, &EnterOnFirstObservation, &config).unwrap();
        let benchmark = result.buy_and_hold_benchmark.unwrap();
        assert_eq!(benchmark.name, "buy-and-hold (long-only; estimated costs)");
        assert!(benchmark.ending_equity > config.initial_capital);
    }

    #[test]
    fn buy_and_hold_benchmark_reports_drawdown_from_peak_equity() {
        let mut data = dataset(&[100.0, 100.0, 120.0, 80.0]);
        data.metadata.instrument_type = crate::data::InstrumentType::CashEquity;
        let config = BacktestConfig {
            out_of_sample_fraction: 1.0,
            buy_and_hold_benchmark: true,
            ..BacktestConfig::default()
        };
        let result = run_backtest(&data, &NoSignals, &config).unwrap();
        let benchmark = result.buy_and_hold_benchmark.unwrap();
        assert!((benchmark.max_drawdown_fraction - 4_000.0 / 102_000.0).abs() < 1e-10);
    }
}
