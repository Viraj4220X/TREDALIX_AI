use super::config::BacktestConfig;
use super::portfolio::Side;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SimulatedFill {
    pub price: f64,
    pub slippage: f64,
    pub fee: f64,
}

pub(super) fn simulate_market_fill(
    reference_price: f64,
    side: Side,
    quantity: u64,
    config: &BacktestConfig,
) -> Result<SimulatedFill, String> {
    if !reference_price.is_finite() || reference_price <= 0.0 || quantity == 0 {
        return Err("Fill reference price and quantity must be positive".to_owned());
    }
    let direction = match side {
        Side::Buy => 1.0,
        Side::Sell => -1.0,
    };
    let price = reference_price * (1.0 + direction * config.slippage_bps / 10_000.0);
    if !price.is_finite() || price <= 0.0 {
        return Err("Simulated fill price is invalid".to_owned());
    }
    let notional = price * quantity as f64;
    let reference_notional = reference_price * quantity as f64;
    let fee = notional * config.fee_bps / 10_000.0;
    let slippage = (notional - reference_notional).abs();
    if !notional.is_finite() || !fee.is_finite() || !slippage.is_finite() {
        return Err("Simulated notional or transaction cost overflowed".to_owned());
    }
    Ok(SimulatedFill {
        price,
        slippage,
        fee,
    })
}

#[cfg(test)]
mod tests {
    use super::simulate_market_fill;
    use crate::backtest::{BacktestConfig, Side};

    #[test]
    fn adverse_slippage_is_applied_for_buy_and_sell() {
        let config = BacktestConfig {
            slippage_bps: 100.0,
            fee_bps: 10.0,
            ..BacktestConfig::default()
        };
        let buy = simulate_market_fill(100.0, Side::Buy, 2, &config).unwrap();
        let sell = simulate_market_fill(100.0, Side::Sell, 2, &config).unwrap();
        assert_eq!(buy.price, 101.0);
        assert_eq!(sell.price, 99.0);
        assert_eq!(buy.slippage, 2.0);
        assert!((buy.fee - 0.202).abs() < 1e-12);
    }
}
