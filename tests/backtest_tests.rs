use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use tredalix_ai::backtest::{Action, BacktestConfig, Strategy, run_backtest, write_report};
use tredalix_ai::data::{DataClass, DatasetMetadata, InstrumentType, load_dataset};

static TEMP_ID: AtomicUsize = AtomicUsize::new(0);

struct EnterThenExit;

impl Strategy for EnterThenExit {
    fn name(&self) -> &'static str {
        "integration-enter-exit"
    }

    fn version(&self) -> &'static str {
        "1"
    }

    fn action(&self, history: &[tredalix_ai::data::Candle]) -> Action {
        match history.len() {
            1 => Action::EnterLong,
            2 => Action::ExitLong,
            _ => Action::Hold,
        }
    }
}

fn temp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "tredalix-phase2-{name}-{}-{}",
        std::process::id(),
        TEMP_ID.fetch_add(1, Ordering::Relaxed)
    ))
}

#[test]
fn runs_csv_to_trade_accounting_and_reproducible_reports() {
    let csv_path = temp_path("data.csv");
    fs::write(
        &csv_path,
        "timestamp,open,high,low,close,volume\n\
         2026-10-20 09:15,100,101,99,100,1000\n\
         2026-10-20 09:16,100,101,99,101,1000\n\
         2026-10-20 09:17,110,111,109,110,1000\n\
         2026-10-20 09:18,120,121,119,120,1000\n",
    )
    .unwrap();
    let mut metadata = DatasetMetadata::unverified("TEST");
    metadata.source = "synthetic integration fixture".to_owned();
    metadata.interval_seconds = Some(60);
    metadata.data_class = DataClass::Synthetic;
    metadata.instrument_type = InstrumentType::CashEquity;
    let dataset = load_dataset(&csv_path, metadata).unwrap();
    let config = BacktestConfig {
        initial_capital: 1_000.0,
        max_quantity: 10,
        max_exposure: 1_000.0,
        fee_bps: 10.0,
        slippage_bps: 10.0,
        out_of_sample_fraction: 1.0,
        ..BacktestConfig::default()
    };
    let result = run_backtest(&dataset, &EnterThenExit, &config).unwrap();
    let repeated = run_backtest(&dataset, &EnterThenExit, &config).unwrap();

    assert_eq!(result.orders.len(), 2);
    assert_eq!(result.orders, repeated.orders);
    assert_eq!(
        result.orders[0].execution_timestamp,
        Some(dataset.candles[1].timestamp)
    );
    assert_eq!(result.trades.len(), 1);
    assert_eq!(result.trades, repeated.trades);
    assert_eq!(result.trades[0].quantity, 9);
    assert!((result.trades[0].entry_price - 100.1).abs() < 1e-10);
    assert!((result.trades[0].exit_price - 109.89).abs() < 1e-10);
    assert!((result.trades[0].gross_pnl - 88.11).abs() < 1e-10);
    assert!((result.trades[0].fees - 1.88991).abs() < 1e-10);
    assert!((result.trades[0].slippage - 1.89).abs() < 1e-10);
    assert!((result.trades[0].net_pnl - 86.22009).abs() < 1e-10);
    assert!((result.metrics.ending_equity - 1_086.220_09).abs() < 1e-10);
    assert_eq!(result.equity_curve, repeated.equity_curve);
    assert_eq!(result.metrics, repeated.metrics);
    assert!(result.metrics.fees > 0.0);
    assert!(result.metrics.slippage > 0.0);
    assert_eq!(result.metrics.open_positions, 0);

    let report_dir = temp_path("report");
    write_report(&result, &report_dir).unwrap();
    let orders = fs::read_to_string(report_dir.join("orders.csv")).unwrap();
    let trades = fs::read_to_string(report_dir.join("trades.csv")).unwrap();
    let config_json = fs::read_to_string(report_dir.join("effective_config.json")).unwrap();
    assert!(orders.contains("FILLED"));
    assert!(trades.contains("strategy exit"));
    assert!(config_json.contains("\"fee_bps\": 10.0000000000"));
    assert!(write_report(&result, &report_dir).is_err());

    fs::remove_file(csv_path).unwrap();
    fs::remove_dir_all(report_dir).unwrap();
}

#[test]
fn rejects_index_buy_and_hold_execution_benchmark() {
    let csv_path = temp_path("index.csv");
    fs::write(
        &csv_path,
        "timestamp,open,high,low,close\n2026-10-20 09:15,100,100,100,100\n",
    )
    .unwrap();
    let mut metadata = DatasetMetadata::unverified("NIFTY50");
    metadata.interval_seconds = Some(60);
    metadata.instrument_type = InstrumentType::Index;
    let dataset = load_dataset(&csv_path, metadata).unwrap();
    let config = BacktestConfig {
        buy_and_hold_benchmark: true,
        ..BacktestConfig::default()
    };
    assert!(
        run_backtest(&dataset, &EnterThenExit, &config)
            .unwrap_err()
            .contains("CASH EQUITY")
    );
    fs::remove_file(csv_path).unwrap();
}
