use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use tredalix_ai::backtest::{BacktestConfig, write_report};
use tredalix_ai::data::{
    DataClass, DatasetMetadata, InstrumentType, load_dataset, print_quality_report,
};
use tredalix_ai::{config, horizons, system};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    println!("==============================");
    println!("       {} v{}", config::APP_NAME, config::VERSION);
    println!("==============================");
    system::show_status();
    horizons::show_horizons();

    let result = run_command(&args[1..]);
    if let Err(error) = result {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

fn run_command(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("backtest" | "inspect") if args.len() >= 3 => {
            let (metadata, config, report_dir) = parse_options(&args[1], &args[3..])?;
            let dataset = load_dataset(&args[2], metadata)?;
            if args[0] == "inspect" {
                print_quality_report(&dataset);
                Ok(())
            } else {
                print_quality_report(&dataset);
                let result = tredalix_ai::backtest::run_backtest(
                    &dataset,
                    &tredalix_ai::backtest::SmaRsiStrategy,
                    &config,
                )?;
                print_run_summary(&result);
                let report_path = write_report(&result, report_dir)?;
                println!("Report saved: {}", report_path.display());
                Ok(())
            }
        }
        _ => Err(
            "Usage:\n\
             cargo run -- inspect SYMBOL PATH [--interval-seconds N] [--source NAME] [--synthetic]\n\
             cargo run -- backtest SYMBOL PATH [--interval-seconds N] [--source NAME] [--synthetic]\n\
             [--capital N] [--fee-bps N] [--slippage-bps N] [--max-quantity N]\n\
             [--max-exposure N] [--test-fraction N] [--instrument-type equity|index]\n\
             [--buy-and-hold] [--report-dir PATH]"
                .to_owned(),
        ),
    }
}

fn parse_options(
    instrument: &str,
    args: &[String],
) -> Result<(DatasetMetadata, BacktestConfig, PathBuf), String> {
    let mut metadata = DatasetMetadata::unverified(instrument);
    let mut config = BacktestConfig::default();
    let mut report_dir = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--synthetic" => {
                metadata.data_class = DataClass::Synthetic;
                index += 1;
            }
            "--buy-and-hold" => {
                config.buy_and_hold_benchmark = true;
                index += 1;
            }
            "--interval-seconds" | "--source" | "--instrument-type" | "--capital" | "--fee-bps"
            | "--slippage-bps" | "--max-quantity" | "--max-exposure" | "--test-fraction"
            | "--report-dir" => {
                let option = args[index].as_str();
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| format!("Missing value for {option}"))?;
                match option {
                    "--interval-seconds" => {
                        let interval = parse_number::<u64>(value, option)?;
                        if interval == 0 {
                            return Err("Interval must be greater than zero".to_owned());
                        }
                        metadata.interval_seconds = Some(interval);
                    }
                    "--source" => metadata.source = value.clone(),
                    "--instrument-type" => {
                        metadata.instrument_type = match value.to_ascii_lowercase().as_str() {
                            "equity" => InstrumentType::CashEquity,
                            "index" => InstrumentType::Index,
                            _ => {
                                return Err(
                                    "Instrument type must be 'equity' or 'index'".to_owned()
                                );
                            }
                        }
                    }
                    "--capital" => config.initial_capital = parse_number(value, option)?,
                    "--fee-bps" => config.fee_bps = parse_number(value, option)?,
                    "--slippage-bps" => config.slippage_bps = parse_number(value, option)?,
                    "--max-quantity" => config.max_quantity = parse_number(value, option)?,
                    "--max-exposure" => config.max_exposure = parse_number(value, option)?,
                    "--test-fraction" => {
                        config.out_of_sample_fraction = parse_number(value, option)?
                    }
                    "--report-dir" => report_dir = Some(PathBuf::from(value)),
                    _ => unreachable!(),
                }
                index += 2;
            }
            option => return Err(format!("Unknown option: {option}")),
        }
    }
    config.validate()?;
    let report_dir = report_dir.unwrap_or_else(|| default_report_dir(instrument));
    Ok((metadata, config, report_dir))
}

fn parse_number<T: std::str::FromStr>(value: &str, option: &str) -> Result<T, String> {
    value
        .parse()
        .map_err(|_| format!("Invalid numeric value for {option}: {value}"))
}

fn default_report_dir(instrument: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    PathBuf::from("reports").join(format!(
        "backtest-{}-{timestamp}-{}",
        instrument.replace(['\\', '/', ':', '*', '?', '"', '<', '>', '|'], "_"),
        std::process::id()
    ))
}

fn print_run_summary(result: &tredalix_ai::backtest::BacktestResult) {
    let metrics = &result.metrics;
    println!("\n===== PORTFOLIO BACKTEST =====");
    println!(
        "Strategy: {} v{}",
        result.strategy_name, result.strategy_version
    );
    println!(
        "Orders: {} submitted, {} filled, {} rejected",
        metrics.submitted_orders, metrics.filled_orders, metrics.rejected_orders
    );
    println!("Closed trades: {}", metrics.closed_trades);
    println!("Gross closed-trade PnL: {:.2}", metrics.gross_pnl);
    println!(
        "Fees: {:.2}; estimated slippage: {:.2}",
        metrics.fees, metrics.slippage
    );
    println!("Net portfolio PnL: {:.2}", metrics.net_pnl);
    println!("Ending equity: {:.2}", metrics.ending_equity);
    println!("Total return: {:.2}%", metrics.return_fraction * 100.0);
    println!(
        "Maximum drawdown: {:.2} ({:.2}%)",
        metrics.max_drawdown_amount,
        metrics.max_drawdown_fraction * 100.0
    );
    println!("Open positions at end: {}", metrics.open_positions);
    println!(
        "Cash benchmark return: {:.2}%",
        result.cash_benchmark.return_fraction * 100.0
    );
    if let Some(benchmark) = &result.buy_and_hold_benchmark {
        println!(
            "Buy-and-hold benchmark return: {:.2}%",
            benchmark.return_fraction * 100.0
        );
    }
}
