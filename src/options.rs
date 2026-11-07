use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionKind {
    Call,
    Put,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct BlackScholesResult {
    pub price: f64,
    pub delta: f64,
    pub gamma: f64,
    pub theta_per_year: f64,
    pub vega_per_volatility_unit: f64,
    pub rho_per_rate_unit: f64,
    pub d1: f64,
    pub d2: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionMathError {
    InvalidPrice,
    InvalidMaturity,
    InvalidVolatility,
    InvalidRate,
    InvalidMarketPrice,
    NoImpliedVolatility,
    NonFiniteResult,
}

impl std::fmt::Display for OptionMathError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidPrice => "Spot and strike must be finite positive prices.",
            Self::InvalidMaturity => "Maturity must be finite and greater than zero years.",
            Self::InvalidVolatility => "Volatility must be finite and in (0, 10].",
            Self::InvalidRate => "Risk-free rate must be finite and in [-1, 1].",
            Self::InvalidMarketPrice => "Market option price must be finite and non-negative.",
            Self::NoImpliedVolatility => {
                "The market price is outside Black-Scholes no-arbitrage bounds."
            }
            Self::NonFiniteResult => "Option calculation produced a non-finite result.",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for OptionMathError {}

fn validate_inputs(
    spot: f64,
    strike: f64,
    maturity_years: f64,
    risk_free_rate: f64,
    volatility: f64,
) -> Result<(), OptionMathError> {
    if !spot.is_finite()
        || !strike.is_finite()
        || spot <= 0.0
        || strike <= 0.0
        || spot > 1.0e12
        || strike > 1.0e12
    {
        return Err(OptionMathError::InvalidPrice);
    }
    if !maturity_years.is_finite()
        || !(0.0..=100.0).contains(&maturity_years)
        || maturity_years == 0.0
    {
        return Err(OptionMathError::InvalidMaturity);
    }
    if !volatility.is_finite() || !(0.0..=10.0).contains(&volatility) || volatility == 0.0 {
        return Err(OptionMathError::InvalidVolatility);
    }
    if !risk_free_rate.is_finite() || !(-1.0..=1.0).contains(&risk_free_rate) {
        return Err(OptionMathError::InvalidRate);
    }
    Ok(())
}

fn normal_pdf(value: f64) -> f64 {
    (-0.5 * value * value).exp() / (2.0 * std::f64::consts::PI).sqrt()
}

fn normal_cdf(value: f64) -> f64 {
    let absolute = value.abs();
    let t = 1.0 / (1.0 + 0.231_641_9 * absolute);
    let polynomial =
        (((1.330_274_429 * t - 1.821_255_978) * t + 1.781_477_937) * t - 0.356_563_782) * t
            + 0.319_381_530;
    let tail = normal_pdf(absolute) * t * polynomial;
    if value >= 0.0 { 1.0 - tail } else { tail }
}

pub fn black_scholes(
    kind: OptionKind,
    spot: f64,
    strike: f64,
    maturity_years: f64,
    risk_free_rate: f64,
    volatility: f64,
) -> Result<BlackScholesResult, OptionMathError> {
    validate_inputs(spot, strike, maturity_years, risk_free_rate, volatility)?;
    let sqrt_time = maturity_years.sqrt();
    let discounted_strike = strike * (-risk_free_rate * maturity_years).exp();
    let d1 = ((spot / strike).ln()
        + (risk_free_rate + 0.5 * volatility * volatility) * maturity_years)
        / (volatility * sqrt_time);
    let d2 = d1 - volatility * sqrt_time;
    let density = normal_pdf(d1);
    let (price, delta, theta, rho) = match kind {
        OptionKind::Call => (
            spot * normal_cdf(d1) - discounted_strike * normal_cdf(d2),
            normal_cdf(d1),
            -spot * density * volatility / (2.0 * sqrt_time)
                - risk_free_rate * discounted_strike * normal_cdf(d2),
            strike * maturity_years * (-risk_free_rate * maturity_years).exp() * normal_cdf(d2),
        ),
        OptionKind::Put => (
            discounted_strike * normal_cdf(-d2) - spot * normal_cdf(-d1),
            normal_cdf(d1) - 1.0,
            -spot * density * volatility / (2.0 * sqrt_time)
                + risk_free_rate * discounted_strike * normal_cdf(-d2),
            -strike * maturity_years * (-risk_free_rate * maturity_years).exp() * normal_cdf(-d2),
        ),
    };
    let result = BlackScholesResult {
        price,
        delta,
        gamma: density / (spot * volatility * sqrt_time),
        theta_per_year: theta,
        vega_per_volatility_unit: spot * density * sqrt_time,
        rho_per_rate_unit: rho,
        d1,
        d2,
    };
    if [
        result.price,
        result.delta,
        result.gamma,
        result.theta_per_year,
        result.vega_per_volatility_unit,
        result.rho_per_rate_unit,
        result.d1,
        result.d2,
    ]
    .iter()
    .all(|value| value.is_finite())
    {
        Ok(result)
    } else {
        Err(OptionMathError::NonFiniteResult)
    }
}

pub fn implied_volatility(
    kind: OptionKind,
    market_price: f64,
    spot: f64,
    strike: f64,
    maturity_years: f64,
    risk_free_rate: f64,
) -> Result<f64, OptionMathError> {
    if !market_price.is_finite() || market_price < 0.0 {
        return Err(OptionMathError::InvalidMarketPrice);
    }
    validate_inputs(spot, strike, maturity_years, risk_free_rate, 0.2)?;
    let discounted_strike = strike * (-risk_free_rate * maturity_years).exp();
    let (lower, upper) = match kind {
        OptionKind::Call => ((spot - discounted_strike).max(0.0), spot),
        OptionKind::Put => ((discounted_strike - spot).max(0.0), discounted_strike),
    };
    if market_price < lower || market_price >= upper {
        return Err(OptionMathError::NoImpliedVolatility);
    }
    let (mut low, mut high) = (1.0e-6, 10.0);
    for _ in 0..120 {
        let mid = low + (high - low) / 2.0;
        let price = black_scholes(kind, spot, strike, maturity_years, risk_free_rate, mid)?.price;
        if price < market_price {
            low = mid;
        } else {
            high = mid;
        }
    }
    let result = low + (high - low) / 2.0;
    if result.is_finite() {
        Ok(result)
    } else {
        Err(OptionMathError::NonFiniteResult)
    }
}

#[cfg(test)]
mod tests {
    use super::{OptionKind, OptionMathError, black_scholes, implied_volatility};

    fn close(actual: f64, expected: f64, tolerance: f64) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn known_call_price_and_greeks_match_reference_values() {
        let result = black_scholes(OptionKind::Call, 100.0, 100.0, 1.0, 0.05, 0.2).unwrap();
        close(result.price, 10.450_583_572_185_565, 1.0e-5);
        close(result.delta, 0.636_830_651_175_619_1, 1.0e-6);
        close(result.gamma, 0.018_762_017_345_846_895, 1.0e-7);
        close(
            result.vega_per_volatility_unit,
            37.524_034_691_693_79,
            1.0e-6,
        );
    }

    #[test]
    fn put_call_prices_satisfy_put_call_parity() {
        let call = black_scholes(OptionKind::Call, 100.0, 105.0, 0.75, 0.03, 0.27).unwrap();
        let put = black_scholes(OptionKind::Put, 100.0, 105.0, 0.75, 0.03, 0.27).unwrap();
        let parity = 100.0 - 105.0 * (-0.03_f64 * 0.75).exp();
        close(call.price - put.price, parity, 1.0e-7);
        assert!((0.0..=1.0).contains(&call.delta));
        assert!((-1.0..=0.0).contains(&put.delta));
    }

    #[test]
    fn implied_volatility_solver_recovers_input_volatility() {
        let price = black_scholes(OptionKind::Put, 100.0, 95.0, 0.5, 0.04, 0.32)
            .unwrap()
            .price;
        close(
            implied_volatility(OptionKind::Put, price, 100.0, 95.0, 0.5, 0.04).unwrap(),
            0.32,
            1.0e-8,
        );
    }

    #[test]
    fn implied_volatility_solver_covers_the_supported_volatility_limit() {
        let price = black_scholes(OptionKind::Call, 100.0, 100.0, 1.0, 0.02, 6.0)
            .unwrap()
            .price;
        close(
            implied_volatility(OptionKind::Call, price, 100.0, 100.0, 1.0, 0.02).unwrap(),
            6.0,
            1.0e-7,
        );
    }

    #[test]
    fn option_calculator_rejects_invalid_and_non_finite_inputs() {
        assert_eq!(
            black_scholes(OptionKind::Call, 0.0, 100.0, 1.0, 0.0, 0.2),
            Err(OptionMathError::InvalidPrice)
        );
        assert_eq!(
            black_scholes(OptionKind::Call, 100.0, 100.0, 0.0, 0.0, 0.2),
            Err(OptionMathError::InvalidMaturity)
        );
        assert_eq!(
            black_scholes(OptionKind::Call, 100.0, 100.0, 1.0, 0.0, f64::NAN),
            Err(OptionMathError::InvalidVolatility)
        );
        assert_eq!(
            implied_volatility(OptionKind::Call, 150.0, 100.0, 100.0, 1.0, 0.0),
            Err(OptionMathError::NoImpliedVolatility)
        );
    }
}
