use super::portfolio::{EquityPoint, TradeRecord};

#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceMetrics {
    pub submitted_orders: usize,
    pub filled_orders: usize,
    pub rejected_orders: usize,
    pub closed_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub gross_profit: f64,
    pub gross_loss: f64,
    pub gross_pnl: f64,
    pub fees: f64,
    pub slippage: f64,
    pub net_pnl: f64,
    pub ending_equity: f64,
    pub return_fraction: f64,
    pub win_rate: Option<f64>,
    pub average_winner: Option<f64>,
    pub average_loser: Option<f64>,
    pub profit_factor: Option<f64>,
    pub expectancy: Option<f64>,
    pub max_drawdown_amount: f64,
    pub max_drawdown_fraction: f64,
    pub open_positions: usize,
    pub unrealized_gross_pnl: f64,
    pub exposure_fraction: f64,
    pub turnover: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkMetrics {
    pub name: String,
    pub ending_equity: f64,
    pub return_fraction: f64,
    pub max_drawdown_fraction: f64,
    pub fees: f64,
    pub exposure_fraction: f64,
}

pub(super) struct MetricInputs<'a> {
    pub initial_capital: f64,
    pub ending_equity: f64,
    pub trades: &'a [TradeRecord],
    pub curve: &'a [EquityPoint],
    pub submitted_orders: usize,
    pub filled_orders: usize,
    pub rejected_orders: usize,
    pub fees: f64,
    pub slippage: f64,
    pub open_positions: usize,
    pub unrealized_gross_pnl: f64,
    pub turnover: f64,
}

pub(super) fn calculate_metrics(input: MetricInputs<'_>) -> PerformanceMetrics {
    let MetricInputs {
        initial_capital,
        ending_equity,
        trades,
        curve,
        submitted_orders,
        filled_orders,
        rejected_orders,
        fees,
        slippage,
        open_positions,
        unrealized_gross_pnl,
        turnover,
    } = input;
    let gross_profit: f64 = trades
        .iter()
        .filter(|trade| trade.gross_pnl > 0.0)
        .map(|trade| trade.gross_pnl)
        .sum();
    let gross_loss: f64 = trades
        .iter()
        .filter(|trade| trade.gross_pnl < 0.0)
        .map(|trade| trade.gross_pnl.abs())
        .sum();
    let net_pnl = ending_equity - initial_capital;
    let closed_net_pnl: f64 = trades.iter().map(|trade| trade.net_pnl).sum();
    let winners: Vec<f64> = trades
        .iter()
        .filter(|trade| trade.net_pnl > 0.0)
        .map(|trade| trade.net_pnl)
        .collect();
    let losers: Vec<f64> = trades
        .iter()
        .filter(|trade| trade.net_pnl < 0.0)
        .map(|trade| trade.net_pnl)
        .collect();
    let max_drawdown_amount = curve
        .iter()
        .map(|point| point.drawdown_amount)
        .fold(0.0, f64::max);
    let max_drawdown_fraction = curve.iter().map(|point| point.drawdown).fold(0.0, f64::max);
    let exposure_fraction = if curve.is_empty() {
        0.0
    } else {
        curve
            .iter()
            .map(|point| point.position_value / point.equity.max(f64::EPSILON))
            .sum::<f64>()
            / curve.len() as f64
    };

    PerformanceMetrics {
        submitted_orders,
        filled_orders,
        rejected_orders,
        closed_trades: trades.len(),
        winning_trades: winners.len(),
        losing_trades: losers.len(),
        gross_profit,
        gross_loss,
        gross_pnl: gross_profit - gross_loss,
        fees,
        slippage,
        net_pnl,
        ending_equity,
        return_fraction: (ending_equity - initial_capital) / initial_capital,
        win_rate: (!trades.is_empty()).then(|| winners.len() as f64 / trades.len() as f64),
        average_winner: average(&winners),
        average_loser: average(&losers),
        profit_factor: (gross_loss > 0.0).then_some(gross_profit / gross_loss),
        expectancy: (!trades.is_empty()).then(|| closed_net_pnl / trades.len() as f64),
        max_drawdown_amount,
        max_drawdown_fraction,
        open_positions,
        unrealized_gross_pnl,
        exposure_fraction,
        turnover,
    }
}

fn average(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::{MetricInputs, calculate_metrics};
    use crate::backtest::TradeRecord;

    fn trade(net: f64, gross: f64) -> TradeRecord {
        TradeRecord {
            entry_timestamp: 0,
            exit_timestamp: 1,
            quantity: 1,
            entry_price: 1.0,
            exit_price: 1.0,
            gross_pnl: gross,
            fees: gross - net,
            slippage: 0.0,
            net_pnl: net,
            exit_reason: "test".to_owned(),
        }
    }

    #[test]
    fn computes_win_rate_profit_factor_and_expectancy() {
        let metrics = calculate_metrics(MetricInputs {
            initial_capital: 100.0,
            ending_equity: 108.0,
            trades: &[trade(12.0, 12.0), trade(-4.0, -4.0)],
            curve: &[],
            submitted_orders: 4,
            filled_orders: 4,
            rejected_orders: 0,
            fees: 0.0,
            slippage: 0.0,
            open_positions: 0,
            unrealized_gross_pnl: 0.0,
            turnover: 200.0,
        });
        assert_eq!(metrics.win_rate, Some(0.5));
        assert_eq!(metrics.profit_factor, Some(3.0));
        assert_eq!(metrics.expectancy, Some(4.0));
        assert_eq!(metrics.return_fraction, 0.08);
    }

    #[test]
    fn no_trades_and_no_losses_have_explicit_undefined_metrics() {
        let no_trades = calculate_metrics(MetricInputs {
            initial_capital: 100.0,
            ending_equity: 100.0,
            trades: &[],
            curve: &[],
            submitted_orders: 0,
            filled_orders: 0,
            rejected_orders: 0,
            fees: 0.0,
            slippage: 0.0,
            open_positions: 0,
            unrealized_gross_pnl: 0.0,
            turnover: 0.0,
        });
        assert_eq!(no_trades.win_rate, None);
        assert_eq!(no_trades.expectancy, None);
        let all_wins = calculate_metrics(MetricInputs {
            initial_capital: 100.0,
            ending_equity: 110.0,
            trades: &[trade(10.0, 10.0)],
            curve: &[],
            submitted_orders: 2,
            filled_orders: 2,
            rejected_orders: 0,
            fees: 0.0,
            slippage: 0.0,
            open_positions: 0,
            unrealized_gross_pnl: 0.0,
            turnover: 100.0,
        });
        assert_eq!(all_wins.profit_factor, None);
    }

    #[test]
    fn all_losing_trades_have_zero_profit_factor_and_negative_expectancy() {
        let metrics = calculate_metrics(MetricInputs {
            initial_capital: 100.0,
            ending_equity: 90.0,
            trades: &[trade(-4.0, -4.0), trade(-6.0, -6.0)],
            curve: &[],
            submitted_orders: 4,
            filled_orders: 4,
            rejected_orders: 0,
            fees: 0.0,
            slippage: 0.0,
            open_positions: 0,
            unrealized_gross_pnl: 0.0,
            turnover: 200.0,
        });
        assert_eq!(metrics.win_rate, Some(0.0));
        assert_eq!(metrics.profit_factor, Some(0.0));
        assert_eq!(metrics.expectancy, Some(-5.0));
    }
}
