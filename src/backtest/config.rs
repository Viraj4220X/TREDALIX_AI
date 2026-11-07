#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndOfDataPolicy {
    MarkToMarket,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BacktestConfig {
    pub initial_capital: f64,
    pub fee_bps: f64,
    pub slippage_bps: f64,
    pub max_quantity: u64,
    pub max_exposure: f64,
    pub max_open_positions: usize,
    pub out_of_sample_fraction: f64,
    pub buy_and_hold_benchmark: bool,
    pub end_of_data_policy: EndOfDataPolicy,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: 100_000.0,
            fee_bps: 0.0,
            slippage_bps: 0.0,
            max_quantity: 100,
            max_exposure: 100_000.0,
            max_open_positions: 1,
            out_of_sample_fraction: 0.3,
            buy_and_hold_benchmark: false,
            end_of_data_policy: EndOfDataPolicy::MarkToMarket,
        }
    }
}

impl BacktestConfig {
    pub fn validate(&self) -> Result<(), String> {
        if !self.initial_capital.is_finite() || self.initial_capital <= 0.0 {
            return Err("Initial capital must be finite and greater than zero".to_owned());
        }
        if !self.fee_bps.is_finite() || !(0.0..10_000.0).contains(&self.fee_bps) {
            return Err("Fee basis points must be finite and in [0, 10000)".to_owned());
        }
        if !self.slippage_bps.is_finite() || !(0.0..10_000.0).contains(&self.slippage_bps) {
            return Err("Slippage basis points must be finite and in [0, 10000)".to_owned());
        }
        if self.max_quantity == 0 {
            return Err("Maximum quantity must be greater than zero".to_owned());
        }
        if !self.max_exposure.is_finite() || self.max_exposure <= 0.0 {
            return Err("Maximum exposure must be finite and greater than zero".to_owned());
        }
        if self.max_open_positions != 1 {
            return Err("This version supports exactly one open long position".to_owned());
        }
        if !self.out_of_sample_fraction.is_finite()
            || !(0.0..=1.0).contains(&self.out_of_sample_fraction)
            || self.out_of_sample_fraction == 0.0
        {
            return Err("Out-of-sample fraction must be in (0, 1]".to_owned());
        }
        Ok(())
    }

    pub fn test_start_index(&self, candle_count: usize) -> usize {
        if self.out_of_sample_fraction >= 1.0 {
            0
        } else {
            ((candle_count as f64) * (1.0 - self.out_of_sample_fraction)).floor() as usize
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BacktestConfig;

    #[test]
    fn validates_config_and_computes_chronological_test_window() {
        let config = BacktestConfig {
            out_of_sample_fraction: 0.25,
            ..BacktestConfig::default()
        };
        assert!(config.validate().is_ok());
        assert_eq!(config.test_start_index(100), 75);
    }

    #[test]
    fn rejects_nonfinite_and_out_of_range_settings() {
        let config = BacktestConfig {
            initial_capital: f64::NAN,
            ..BacktestConfig::default()
        };
        assert!(config.validate().is_err());
        let config = BacktestConfig {
            slippage_bps: 10_000.0,
            ..BacktestConfig::default()
        };
        assert!(config.validate().is_err());
        let config = BacktestConfig {
            out_of_sample_fraction: 0.0,
            ..BacktestConfig::default()
        };
        assert!(config.validate().is_err());
    }
}
