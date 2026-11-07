use super::config::BacktestConfig;
use super::portfolio::Portfolio;

pub(super) fn position_quantity(
    reference_price: f64,
    config: &BacktestConfig,
    portfolio: &Portfolio,
) -> Result<u64, String> {
    if portfolio.position.is_some() {
        return Err("maximum open position limit reached".to_owned());
    }
    if !reference_price.is_finite() || reference_price <= 0.0 {
        return Err("invalid execution reference price".to_owned());
    }
    let slippage_adjusted = reference_price * (1.0 + config.slippage_bps / 10_000.0);
    let fee_multiplier = 1.0 + config.fee_bps / 10_000.0;
    let per_unit_cost = slippage_adjusted * fee_multiplier;
    let cash_quantity = (portfolio.cash / per_unit_cost).floor() as u64;
    let exposure_quantity = (config.max_exposure / slippage_adjusted).floor() as u64;
    let quantity = config
        .max_quantity
        .min(cash_quantity)
        .min(exposure_quantity);
    if quantity == 0 {
        return Err("cash or maximum-exposure limit prevents a one-unit order".to_owned());
    }
    Ok(quantity)
}

#[cfg(test)]
mod tests {
    use super::position_quantity;
    use crate::backtest::{BacktestConfig, portfolio::Portfolio};

    #[test]
    fn position_size_obeys_quantity_cash_and_exposure_limits() {
        let config = BacktestConfig {
            max_quantity: 10,
            max_exposure: 250.0,
            ..BacktestConfig::default()
        };
        let portfolio = Portfolio::new(1_000.0);
        assert_eq!(position_quantity(100.0, &config, &portfolio), Ok(2));
        let poor = Portfolio::new(50.0);
        assert!(position_quantity(100.0, &config, &poor).is_err());
    }
}
