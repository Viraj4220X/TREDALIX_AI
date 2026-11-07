use crate::data::Candle;

use super::IndicatorError;
use super::series::{finite_result, validate_period, validate_values};

#[derive(Debug, Clone, PartialEq)]
pub struct BollingerBands {
    pub middle: Vec<Option<f64>>,
    pub upper: Vec<Option<f64>>,
    pub lower: Vec<Option<f64>>,
}

pub fn bollinger_bands(
    values: &[Option<f64>],
    period: usize,
    standard_deviations: f64,
) -> Result<BollingerBands, IndicatorError> {
    validate_period(period)?;
    validate_values(values)?;
    if !standard_deviations.is_finite() || standard_deviations < 0.0 {
        return Err(IndicatorError::InvalidParameters);
    }
    let mut middle = vec![None; values.len()];
    let mut upper = vec![None; values.len()];
    let mut lower = vec![None; values.len()];
    for (index, window) in values.windows(period).enumerate() {
        let Some(window) = window.iter().copied().collect::<Option<Vec<_>>>() else {
            continue;
        };
        let mean = window.iter().sum::<f64>() / period as f64;
        let variance = window
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>()
            / period as f64;
        let deviation = finite_result(variance.sqrt())?;
        let spread = finite_result(standard_deviations * deviation)?;
        let center = finite_result(mean)?;
        middle[index + period - 1] = Some(center);
        upper[index + period - 1] = Some(finite_result(center + spread)?);
        lower[index + period - 1] = Some(finite_result(center - spread)?);
    }
    Ok(BollingerBands {
        middle,
        upper,
        lower,
    })
}

pub fn atr(candles: &[Candle], period: usize) -> Result<Vec<Option<f64>>, IndicatorError> {
    validate_period(period)?;
    let mut true_ranges = Vec::with_capacity(candles.len());
    for (index, candle) in candles.iter().enumerate() {
        if ![candle.open, candle.high, candle.low, candle.close]
            .into_iter()
            .all(|value| value.is_finite() && value > 0.0)
            || candle.high < candle.open.max(candle.close).max(candle.low)
            || candle.low > candle.open.min(candle.close).min(candle.high)
        {
            return Err(IndicatorError::InvalidCandle);
        }
        let range = match index
            .checked_sub(1)
            .and_then(|previous| candles.get(previous))
        {
            Some(previous) => (candle.high - candle.low)
                .max((candle.high - previous.close).abs())
                .max((candle.low - previous.close).abs()),
            None => candle.high - candle.low,
        };
        true_ranges.push(finite_result(range)?);
    }

    let mut result = vec![None; candles.len()];
    if candles.len() < period {
        return Ok(result);
    }
    let first_average = finite_result(true_ranges[..period].iter().sum::<f64>() / period as f64)?;
    result[period - 1] = Some(first_average);
    let mut previous_atr = first_average;
    for index in period..candles.len() {
        let next = finite_result(
            (previous_atr * (period as f64 - 1.0) + true_ranges[index]) / period as f64,
        )?;
        result[index] = Some(next);
        previous_atr = next;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::data::Candle;

    use super::{IndicatorError, atr, bollinger_bands};

    #[test]
    fn bollinger_bands_match_population_standard_deviation_fixture() {
        let bands = bollinger_bands(&[Some(1.0), Some(2.0), Some(3.0)], 3, 2.0).unwrap();
        let deviation = (2.0_f64 / 3.0).sqrt();
        assert_eq!(bands.middle, [None, None, Some(2.0)]);
        assert!((bands.upper[2].unwrap() - (2.0 + 2.0 * deviation)).abs() < 1e-12);
        assert!((bands.lower[2].unwrap() - (2.0 - 2.0 * deviation)).abs() < 1e-12);
    }

    #[test]
    fn atr_uses_true_range_and_wilder_smoothing() {
        let candles = [
            Candle {
                timestamp: 0,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: None,
            },
            Candle {
                timestamp: 1,
                open: 11.0,
                high: 13.0,
                low: 10.0,
                close: 12.0,
                volume: None,
            },
            Candle {
                timestamp: 2,
                open: 13.0,
                high: 14.0,
                low: 12.0,
                close: 13.0,
                volume: None,
            },
        ];
        assert_eq!(atr(&candles, 2).unwrap(), [None, Some(2.5), Some(2.25)]);
    }

    #[test]
    fn volatility_indicators_reject_invalid_parameters_and_candles() {
        assert_eq!(
            bollinger_bands(&[Some(1.0)], 1, f64::NAN),
            Err(IndicatorError::InvalidParameters)
        );
        let invalid = [Candle {
            timestamp: 0,
            open: 10.0,
            high: 9.0,
            low: 8.0,
            close: 10.0,
            volume: None,
        }];
        assert_eq!(atr(&invalid, 1), Err(IndicatorError::InvalidCandle));
    }
}
