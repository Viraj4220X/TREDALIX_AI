use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::data::format_timestamp;

use super::engine::BacktestResult;

pub fn write_report(
    result: &BacktestResult,
    destination: impl AsRef<Path>,
) -> Result<PathBuf, String> {
    let directory = destination.as_ref();
    if let Some(parent) = directory
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "Cannot create report parent '{}': {error}",
                parent.display()
            )
        })?;
    }
    fs::create_dir(directory).map_err(|error| {
        format!(
            "Cannot create report directory '{}': {error}",
            directory.display()
        )
    })?;

    write_file(directory, "summary.txt", &summary(result))?;
    write_file(directory, "metrics.csv", &metrics_csv(result))?;
    write_file(directory, "orders.csv", &orders_csv(result))?;
    write_file(directory, "trades.csv", &trades_csv(result))?;
    write_file(directory, "equity_curve.csv", &equity_csv(result))?;
    write_file(directory, "effective_config.json", &config_json(result))?;
    Ok(directory.to_path_buf())
}

fn write_file(directory: &Path, name: &str, contents: &str) -> Result<(), String> {
    let path = directory.join(name);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| format!("Cannot create report file '{}': {error}", path.display()))?;
    file.write_all(contents.as_bytes())
        .map_err(|error| format!("Cannot write report file '{}': {error}", path.display()))
}

fn summary(result: &BacktestResult) -> String {
    let metrics = &result.metrics;
    let mut text = format!(
        "TREDALIX AI backtest report\nInstrument: {} ({})\nSource: {}\nData classification: {}\nDataset fingerprint: {}\nCandles: {}\nInterval seconds: {}\nTimestamp timezone: {}\nTimestamp meaning: {}\nEvaluation window: {} to {}\nStrategy: {} v{}\n",
        result.instrument,
        result.instrument_type,
        result.source,
        result.data_class,
        result.dataset_fingerprint,
        result.candle_count,
        result
            .interval_seconds
            .map_or_else(|| "UNKNOWN".to_owned(), |value| value.to_string()),
        result.timestamp_timezone,
        result.timestamp_meaning,
        format_timestamp(result.first_test_timestamp),
        format_timestamp(result.last_test_timestamp),
        result.strategy_name,
        result.strategy_version,
    );
    text.push_str(&format!(
        "Execution: signal after candle close; market order at next candle open\nEnd-of-data: mark open positions to market; do not force-close\nFees and slippage are configured estimates, not historical broker charges.\n\
        Gross PnL uses simulated fill prices before fees; slippage is already reflected in those fills.\n\
        This simulator does not model spread, order-book depth, queue priority, or partial fills.\n\
         Session calendar is not configured; execution across declared-interval gaps is rejected.\n\
         No live orders are sent. Results are not a guarantee of future performance.\n\n\
         Submitted orders: {}\nFilled orders: {}\nRejected orders: {}\nClosed trades: {}\nWinning / losing trades: {} / {}\n\
         Out-of-sample fraction: {:.4}% (warm-up history is read chronologically before the test window)\n\
         Gross closed-trade PnL: {:.4}\nFees: {:.4}\nEstimated slippage: {:.4}\nNet portfolio PnL: {:.4}\n\
         Ending equity: {:.4}\nTotal return: {:.4}%\nWin rate: {}\nAverage winner: {}\nAverage loser: {}\n\
         Profit factor: {}\nExpectancy per closed trade: {}\nMaximum drawdown: {:.4} ({:.4}%)\n\
         Average exposure: {:.4}%\nTurnover: {:.4}\nOpen positions at end: {}\nUnrealized gross PnL: {:.4}\n",
        metrics.submitted_orders,
        metrics.filled_orders,
        metrics.rejected_orders,
        metrics.closed_trades,
        metrics.winning_trades,
        metrics.losing_trades,
        result.config.out_of_sample_fraction * 100.0,
        metrics.gross_pnl,
        metrics.fees,
        metrics.slippage,
        metrics.net_pnl,
        metrics.ending_equity,
        metrics.return_fraction * 100.0,
        display_option(metrics.win_rate.map(|value| value * 100.0), "%"),
        display_option(metrics.average_winner, ""),
        display_option(metrics.average_loser, ""),
        display_option(metrics.profit_factor, ""),
        display_option(metrics.expectancy, ""),
        metrics.max_drawdown_amount,
        metrics.max_drawdown_fraction * 100.0,
        metrics.exposure_fraction * 100.0,
        metrics.turnover,
        metrics.open_positions,
        metrics.unrealized_gross_pnl,
    ));
    text.push_str(&format!(
        "\nCash benchmark return: {:.4}%\nCash benchmark ending equity: {:.4}\n",
        result.cash_benchmark.return_fraction * 100.0,
        result.cash_benchmark.ending_equity
    ));
    if let Some(benchmark) = &result.buy_and_hold_benchmark {
        text.push_str(&format!(
            "Buy-and-hold benchmark return: {:.4}%\nBuy-and-hold ending equity: {:.4}\nBuy-and-hold maximum drawdown: {:.4}%\n",
            benchmark.return_fraction * 100.0,
            benchmark.ending_equity,
            benchmark.max_drawdown_fraction * 100.0
        ));
    } else {
        text.push_str("Buy-and-hold benchmark: not configured or unavailable\n");
    }
    text
}

fn metrics_csv(result: &BacktestResult) -> String {
    let metrics = &result.metrics;
    format!(
        "metric,value\nsubmitted_orders,{}\nfilled_orders,{}\nrejected_orders,{}\nclosed_trades,{}\nwinning_trades,{}\nlosing_trades,{}\ngross_pnl,{:.10}\nfees,{:.10}\nestimated_slippage,{:.10}\nnet_pnl,{:.10}\nending_equity,{:.10}\nreturn_fraction,{:.10}\nwin_rate,{}\naverage_winner,{}\naverage_loser,{}\nprofit_factor,{}\nexpectancy,{}\nmax_drawdown_amount,{:.10}\nmax_drawdown_fraction,{:.10}\nopen_positions,{}\nunrealized_gross_pnl,{:.10}\naverage_exposure_fraction,{:.10}\nturnover,{:.10}\n",
        metrics.submitted_orders,
        metrics.filled_orders,
        metrics.rejected_orders,
        metrics.closed_trades,
        metrics.winning_trades,
        metrics.losing_trades,
        metrics.gross_pnl,
        metrics.fees,
        metrics.slippage,
        metrics.net_pnl,
        metrics.ending_equity,
        metrics.return_fraction,
        option_number(metrics.win_rate),
        option_number(metrics.average_winner),
        option_number(metrics.average_loser),
        option_number(metrics.profit_factor),
        option_number(metrics.expectancy),
        metrics.max_drawdown_amount,
        metrics.max_drawdown_fraction,
        metrics.open_positions,
        metrics.unrealized_gross_pnl,
        metrics.exposure_fraction,
        metrics.turnover,
    ) + &format!(
        "cash_benchmark_return_fraction,{:.10}\ncash_benchmark_ending_equity,{:.10}\n",
        result.cash_benchmark.return_fraction, result.cash_benchmark.ending_equity
    )
    + &result.buy_and_hold_benchmark.as_ref().map_or_else(String::new, |benchmark| {
        format!(
            "buy_and_hold_return_fraction,{:.10}\nbuy_and_hold_ending_equity,{:.10}\nbuy_and_hold_max_drawdown_fraction,{:.10}\n",
            benchmark.return_fraction, benchmark.ending_equity, benchmark.max_drawdown_fraction
        )
    })
}

fn orders_csv(result: &BacktestResult) -> String {
    let mut csv = String::from(
        "id,decision_timestamp,execution_timestamp,side,quantity,status,fill_price,fee,slippage,rejection_reason\n",
    );
    for order in &result.orders {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{:.10},{:.10},{}\n",
            order.id,
            format_timestamp(order.decision_timestamp),
            order
                .execution_timestamp
                .map_or_else(String::new, format_timestamp),
            order.side.label(),
            order.quantity,
            order.status.label(),
            option_number(order.fill_price),
            order.fee,
            order.slippage,
            csv_escape(order.rejection_reason.as_deref().unwrap_or("")),
        ));
    }
    csv
}

fn trades_csv(result: &BacktestResult) -> String {
    let mut csv = String::from(
        "entry_timestamp,exit_timestamp,quantity,entry_price,exit_price,gross_pnl,fees,slippage,net_pnl,exit_reason\n",
    );
    for trade in &result.trades {
        csv.push_str(&format!(
            "{},{},{},{:.10},{:.10},{:.10},{:.10},{:.10},{:.10},{}\n",
            format_timestamp(trade.entry_timestamp),
            format_timestamp(trade.exit_timestamp),
            trade.quantity,
            trade.entry_price,
            trade.exit_price,
            trade.gross_pnl,
            trade.fees,
            trade.slippage,
            trade.net_pnl,
            csv_escape(&trade.exit_reason),
        ));
    }
    csv
}

fn equity_csv(result: &BacktestResult) -> String {
    let mut csv = String::from(
        "timestamp,cash,position_value,equity,peak_equity,drawdown_amount,drawdown_fraction\n",
    );
    for point in &result.equity_curve {
        csv.push_str(&format!(
            "{},{:.10},{:.10},{:.10},{:.10},{:.10},{:.10}\n",
            format_timestamp(point.timestamp),
            point.cash,
            point.position_value,
            point.equity,
            point.peak_equity,
            point.drawdown_amount,
            point.drawdown
        ));
    }
    csv
}

fn config_json(result: &BacktestResult) -> String {
    format!(
        "{{\n  \"software\": \"TREDALIX AI {}\",\n  \"instrument\": \"{}\",\n  \"instrument_type\": \"{}\",\n  \"source\": \"{}\",\n  \"data_class\": \"{}\",\n  \"dataset_fingerprint\": \"{}\",\n  \"candle_count\": {},\n  \"interval_seconds\": {},\n  \"timestamp_timezone\": \"{}\",\n  \"timestamp_meaning\": \"{}\",\n  \"evaluation_start\": \"{}\",\n  \"evaluation_end\": \"{}\",\n  \"strategy\": \"{}\",\n  \"strategy_version\": \"{}\",\n  \"initial_capital\": {:.10},\n  \"fee_bps\": {:.10},\n  \"slippage_bps\": {:.10},\n  \"max_quantity\": {},\n  \"max_exposure\": {:.10},\n  \"max_open_positions\": {},\n  \"out_of_sample_fraction\": {:.10},\n  \"test_start_index\": {},\n  \"execution_policy\": \"next candle open\",\n  \"end_of_data_policy\": \"mark to market\",\n  \"buy_and_hold_benchmark\": {}\n}}\n",
        env!("CARGO_PKG_VERSION"),
        json_escape(&result.instrument),
        json_escape(&result.instrument_type),
        json_escape(&result.source),
        json_escape(&result.data_class),
        result.dataset_fingerprint,
        result.candle_count,
        result
            .interval_seconds
            .map_or_else(|| "null".to_owned(), |value| value.to_string()),
        json_escape(&result.timestamp_timezone),
        json_escape(&result.timestamp_meaning),
        format_timestamp(result.first_test_timestamp),
        format_timestamp(result.last_test_timestamp),
        json_escape(&result.strategy_name),
        json_escape(&result.strategy_version),
        result.config.initial_capital,
        result.config.fee_bps,
        result.config.slippage_bps,
        result.config.max_quantity,
        result.config.max_exposure,
        result.config.max_open_positions,
        result.config.out_of_sample_fraction,
        result.test_start_index,
        result.config.buy_and_hold_benchmark,
    )
}

fn option_number(value: Option<f64>) -> String {
    value.map_or_else(|| "NA".to_owned(), |number| format!("{number:.10}"))
}

fn display_option(value: Option<f64>, suffix: &str) -> String {
    value.map_or_else(|| "N/A".to_owned(), |number| format!("{number:.4}{suffix}"))
}

fn csv_escape(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0c}' => escaped.push_str("\\f"),
            character if character <= '\u{1f}' => {
                escaped.push_str(&format!("\\u{:04x}", character as u32))
            }
            character => escaped.push(character),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::{csv_escape, json_escape};

    #[test]
    fn escapes_report_text_fields() {
        assert_eq!(csv_escape("a,\"b\""), "\"a,\"\"b\"\"\"");
        assert_eq!(json_escape("a\"b\\c"), "a\\\"b\\\\c");
    }
}
