use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndicatorError {
    InvalidPeriod,
    InvalidParameters,
    NonFiniteInput,
    NonFiniteResult,
    InvalidCandle,
}

impl std::fmt::Display for IndicatorError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidPeriod => "Indicator period must be positive",
            Self::InvalidParameters => "Indicator parameters are incompatible",
            Self::NonFiniteInput => "Indicator input contains a non-finite value",
            Self::NonFiniteResult => "Indicator calculation produced a non-finite value",
            Self::InvalidCandle => "ATR input contains invalid OHLC values",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for IndicatorError {}

pub(super) fn validate_period(period: usize) -> Result<(), IndicatorError> {
    if period == 0 {
        Err(IndicatorError::InvalidPeriod)
    } else {
        Ok(())
    }
}

pub(super) fn validate_values(values: &[Option<f64>]) -> Result<(), IndicatorError> {
    if values.iter().flatten().any(|value| !value.is_finite()) {
        return Err(IndicatorError::NonFiniteInput);
    }
    Ok(())
}

pub(super) fn finite_result(value: f64) -> Result<f64, IndicatorError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(IndicatorError::NonFiniteResult)
    }
}

pub fn sma(values: &[Option<f64>], period: usize) -> Result<Vec<Option<f64>>, IndicatorError> {
    validate_period(period)?;
    validate_values(values)?;

    let mut result = vec![None; values.len()];
    if period > values.len() {
        return Ok(result);
    }
    let mut window = VecDeque::with_capacity(period);
    let mut sum = 0.0;
    for (index, value) in values.iter().enumerate() {
        match value {
            Some(value) => {
                window.push_back(*value);
                sum = finite_result(sum + value)?;
                if window.len() > period {
                    sum = finite_result(sum - window.pop_front().expect("window is nonempty"))?;
                }
                if window.len() == period {
                    result[index] = Some(finite_result(sum / period as f64)?);
                }
            }
            None => {
                window.clear();
                sum = 0.0;
            }
        }
    }
    Ok(result)
}

pub fn ema(values: &[Option<f64>], period: usize) -> Result<Vec<Option<f64>>, IndicatorError> {
    validate_period(period)?;
    validate_values(values)?;

    let mut result = vec![None; values.len()];
    let mut seed_count = 0;
    let mut seed_sum = 0.0;
    let mut current = None;
    let alpha = 2.0 / (period as f64 + 1.0);
    for (index, value) in values.iter().enumerate() {
        match value {
            Some(value) => {
                if let Some(previous) = current {
                    let next = finite_result(alpha * value + (1.0 - alpha) * previous)?;
                    current = Some(next);
                    result[index] = Some(next);
                } else {
                    seed_count += 1;
                    seed_sum = if seed_count == 1 {
                        *value
                    } else {
                        let count = seed_count as f64;
                        finite_result(seed_sum * ((count - 1.0) / count) + value / count)?
                    };
                    if seed_count == period {
                        let first = seed_sum;
                        current = Some(first);
                        result[index] = Some(first);
                    }
                }
            }
            None => {
                seed_count = 0;
                seed_sum = 0.0;
                current = None;
            }
        }
    }
    Ok(result)
}

pub fn rsi(values: &[Option<f64>], period: usize) -> Result<Vec<Option<f64>>, IndicatorError> {
    validate_period(period)?;
    validate_values(values)?;

    let mut result = vec![None; values.len()];
    if period >= values.len() {
        return Ok(result);
    }
    let mut changes = VecDeque::with_capacity(period);
    let mut gains = 0.0;
    let mut losses = 0.0;
    let mut previous = None;

    for (index, value) in values.iter().enumerate() {
        let Some(value) = value else {
            changes.clear();
            gains = 0.0;
            losses = 0.0;
            previous = None;
            continue;
        };
        if let Some(previous_value) = previous {
            let change = finite_result(value - previous_value)?;
            let gain = change.max(0.0);
            let loss = (-change).max(0.0);
            changes.push_back((gain, loss));
            gains = finite_result(gains + gain)?;
            losses = finite_result(losses + loss)?;
            if changes.len() > period {
                let (old_gain, old_loss) = changes.pop_front().expect("change window is nonempty");
                gains = finite_result(gains - old_gain)?;
                losses = finite_result(losses - old_loss)?;
            }
            if changes.len() == period {
                let value = if gains == 0.0 && losses == 0.0 {
                    50.0
                } else if losses == 0.0 {
                    100.0
                } else if gains == 0.0 {
                    0.0
                } else {
                    100.0 - 100.0 / (1.0 + gains / losses)
                };
                result[index] = Some(finite_result(value)?);
            }
        }
        previous = Some(*value);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{IndicatorError, ema, rsi, sma};

    #[test]
    fn sma_ema_and_cutler_rsi_match_hand_calculated_values() {
        let values = [1.0, 2.0, 3.0, 4.0, 5.0].map(Some);
        assert_eq!(
            sma(&values, 3).unwrap(),
            [None, None, Some(2.0), Some(3.0), Some(4.0)]
        );
        assert_eq!(
            ema(&values, 3).unwrap(),
            [None, None, Some(2.0), Some(3.0), Some(4.0)]
        );
        assert_eq!(
            rsi(&[Some(1.0), Some(2.0), Some(3.0), Some(2.0), Some(2.0)], 2).unwrap(),
            [None, None, Some(100.0), Some(50.0), Some(0.0)]
        );
    }

    #[test]
    fn missing_inputs_reset_warmup_and_flat_rsi_is_neutral() {
        assert_eq!(
            sma(&[Some(1.0), None, Some(2.0), Some(4.0)], 2).unwrap(),
            [None, None, None, Some(3.0)]
        );
        assert_eq!(
            rsi(&[Some(10.0), Some(10.0), Some(10.0)], 2).unwrap(),
            [None, None, Some(50.0)]
        );
    }

    #[test]
    fn rejects_invalid_periods_nonfinite_values_and_overflow() {
        assert_eq!(sma(&[Some(1.0)], 0), Err(IndicatorError::InvalidPeriod));
        assert_eq!(
            ema(&[Some(f64::INFINITY)], 1),
            Err(IndicatorError::NonFiniteInput)
        );
        assert_eq!(
            sma(&[Some(f64::MAX), Some(f64::MAX)], 2),
            Err(IndicatorError::NonFiniteResult)
        );
    }
}
