#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn label(self) -> &'static str {
        match self {
            Self::Buy => "BUY",
            Self::Sell => "SELL",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderStatus {
    Submitted,
    Accepted,
    Filled,
    Rejected,
    Cancelled,
}

impl OrderStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Submitted => "SUBMITTED",
            Self::Accepted => "ACCEPTED",
            Self::Filled => "FILLED",
            Self::Rejected => "REJECTED",
            Self::Cancelled => "CANCELLED",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderRecord {
    pub id: usize,
    pub decision_timestamp: i64,
    pub execution_timestamp: Option<i64>,
    pub side: Side,
    pub quantity: u64,
    pub status: OrderStatus,
    pub fill_price: Option<f64>,
    pub fee: f64,
    pub slippage: f64,
    pub rejection_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TradeRecord {
    pub entry_timestamp: i64,
    pub exit_timestamp: i64,
    pub quantity: u64,
    pub entry_price: f64,
    pub exit_price: f64,
    pub gross_pnl: f64,
    pub fees: f64,
    pub slippage: f64,
    pub net_pnl: f64,
    pub exit_reason: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EquityPoint {
    pub timestamp: i64,
    pub cash: f64,
    pub position_value: f64,
    pub equity: f64,
    pub peak_equity: f64,
    pub drawdown: f64,
    pub drawdown_amount: f64,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Position {
    pub quantity: u64,
    pub entry_timestamp: i64,
    pub entry_price: f64,
    pub entry_fee: f64,
    pub entry_slippage: f64,
}

#[derive(Debug)]
pub(super) struct Portfolio {
    pub cash: f64,
    pub position: Option<Position>,
    pub fees: f64,
    pub slippage: f64,
    pub orders: Vec<OrderRecord>,
    pub trades: Vec<TradeRecord>,
    pub equity_curve: Vec<EquityPoint>,
}

impl Portfolio {
    pub fn new(initial_cash: f64) -> Self {
        Self {
            cash: initial_cash,
            position: None,
            fees: 0.0,
            slippage: 0.0,
            orders: Vec::new(),
            trades: Vec::new(),
            equity_curve: Vec::new(),
        }
    }

    pub fn enter(
        &mut self,
        quantity: u64,
        timestamp: i64,
        price: f64,
        fee: f64,
        slippage: f64,
    ) -> Result<(), String> {
        if self.position.is_some() {
            return Err("position already open".to_owned());
        }
        let cost = price * quantity as f64 + fee;
        if !cost.is_finite() || !fee.is_finite() || !slippage.is_finite() {
            return Err("entry notional or costs are not finite".to_owned());
        }
        if cost > self.cash {
            return Err("insufficient cash".to_owned());
        }
        let cash_after = self.cash - cost;
        let fees_after = self.fees + fee;
        let slippage_after = self.slippage + slippage;
        if !cash_after.is_finite() || !fees_after.is_finite() || !slippage_after.is_finite() {
            return Err("entry accounting overflowed".to_owned());
        }
        self.cash = cash_after;
        self.fees = fees_after;
        self.slippage = slippage_after;
        self.position = Some(Position {
            quantity,
            entry_timestamp: timestamp,
            entry_price: price,
            entry_fee: fee,
            entry_slippage: slippage,
        });
        Ok(())
    }

    pub fn exit(
        &mut self,
        timestamp: i64,
        price: f64,
        fee: f64,
        slippage: f64,
        reason: &str,
    ) -> Result<(), String> {
        let position = self
            .position
            .ok_or_else(|| "no open position to exit".to_owned())?;
        let gross = (price - position.entry_price) * position.quantity as f64;
        let fees = position.entry_fee + fee;
        let proceeds = price * position.quantity as f64 - fee;
        let net_pnl = gross - fees;
        let slippage_total = position.entry_slippage + slippage;
        let cash_after = self.cash + proceeds;
        let fees_after = self.fees + fee;
        let slippage_after = self.slippage + slippage;
        if !gross.is_finite()
            || !fee.is_finite()
            || !slippage.is_finite()
            || !fees.is_finite()
            || !proceeds.is_finite()
            || !net_pnl.is_finite()
            || !slippage_total.is_finite()
            || !cash_after.is_finite()
            || !fees_after.is_finite()
            || !slippage_after.is_finite()
        {
            return Err("exit accounting overflowed".to_owned());
        }
        self.position = None;
        self.cash = cash_after;
        self.fees = fees_after;
        self.slippage = slippage_after;
        self.trades.push(TradeRecord {
            entry_timestamp: position.entry_timestamp,
            exit_timestamp: timestamp,
            quantity: position.quantity,
            entry_price: position.entry_price,
            exit_price: price,
            gross_pnl: gross,
            fees,
            slippage: slippage_total,
            net_pnl,
            exit_reason: reason.to_owned(),
        });
        Ok(())
    }

    pub fn mark(&mut self, timestamp: i64, close: f64) -> Result<(), String> {
        let position_value = self
            .position
            .map_or(0.0, |position| close * position.quantity as f64);
        let equity = self.cash + position_value;
        if !position_value.is_finite() || !equity.is_finite() {
            return Err("portfolio valuation overflowed".to_owned());
        }
        let peak = self
            .equity_curve
            .iter()
            .map(|point| point.equity)
            .fold(equity, f64::max);
        let drawdown = if peak > 0.0 {
            (peak - equity) / peak
        } else {
            0.0
        };
        self.equity_curve.push(EquityPoint {
            timestamp,
            cash: self.cash,
            position_value,
            equity,
            peak_equity: peak,
            drawdown,
            drawdown_amount: peak - equity,
        });
        Ok(())
    }

    pub fn equity(&self, mark: f64) -> f64 {
        self.cash
            + self
                .position
                .map_or(0.0, |position| position.quantity as f64 * mark)
    }

    pub fn unrealized_gross_pnl(&self, mark: f64) -> f64 {
        self.position.map_or(0.0, |position| {
            (mark - position.entry_price) * position.quantity as f64
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Portfolio;

    #[test]
    fn entry_exit_reconciles_cash_gross_and_net_pnl() {
        let mut portfolio = Portfolio::new(1_000.0);
        portfolio.enter(2, 10, 100.0, 2.0, 0.0).unwrap();
        assert_eq!(portfolio.cash, 798.0);
        assert_eq!(portfolio.equity(110.0), 1_018.0);
        portfolio
            .exit(20, 110.0, 2.2, 0.0, "strategy exit")
            .unwrap();
        assert_eq!(portfolio.cash, 1_015.8);
        assert_eq!(portfolio.trades[0].gross_pnl, 20.0);
        assert!((portfolio.trades[0].net_pnl - 15.8).abs() < 1e-10);
    }

    #[test]
    fn rejects_cash_shortfall_without_mutating_portfolio() {
        let mut portfolio = Portfolio::new(50.0);
        assert!(portfolio.enter(1, 1, 100.0, 0.0, 0.0).is_err());
        assert_eq!(portfolio.cash, 50.0);
        assert!(portfolio.position.is_none());
    }

    #[test]
    fn tracks_peak_equity_and_absolute_drawdown() {
        let mut portfolio = Portfolio::new(1_000.0);
        portfolio.enter(5, 1, 100.0, 0.0, 0.0).unwrap();
        portfolio.mark(2, 100.0).unwrap();
        portfolio.mark(3, 80.0).unwrap();
        let trough = portfolio.equity_curve.last().unwrap();
        assert_eq!(trough.equity, 900.0);
        assert_eq!(trough.peak_equity, 1_000.0);
        assert_eq!(trough.drawdown_amount, 100.0);
        assert_eq!(trough.drawdown, 0.1);
    }

    #[test]
    fn overflowed_exit_does_not_mutate_portfolio() {
        let mut portfolio = Portfolio::new(f64::MAX);
        portfolio.enter(1, 1, 1.0, 0.0, 0.0).unwrap();
        let cash_before = portfolio.cash;
        assert!(portfolio.exit(2, f64::MAX, 0.0, 0.0, "test").is_err());
        assert_eq!(portfolio.cash, cash_before);
        assert!(portfolio.position.is_some());
        assert!(portfolio.trades.is_empty());
    }

    #[test]
    fn rejects_nonfinite_mark_to_market_values() {
        let mut portfolio = Portfolio::new(100.0);
        portfolio.enter(2, 1, 1.0, 0.0, 0.0).unwrap();
        assert!(portfolio.mark(2, f64::MAX).is_err());
        assert!(portfolio.equity_curve.is_empty());
    }
}
