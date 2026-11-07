use super::{IndicatorError, ema};

#[derive(Debug, Clone, PartialEq)]
pub struct MacdSeries {
    pub line: Vec<Option<f64>>,
    pub signal: Vec<Option<f64>>,
    pub histogram: Vec<Option<f64>>,
}

pub fn macd(
    values: &[Option<f64>],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> Result<MacdSeries, IndicatorError> {
    if fast_period == 0 || slow_period == 0 || signal_period == 0 {
        return Err(IndicatorError::InvalidPeriod);
    }
    if fast_period >= slow_period {
        return Err(IndicatorError::InvalidParameters);
    }
    let fast = ema(values, fast_period)?;
    let slow = ema(values, slow_period)?;
    let line = fast
        .iter()
        .zip(&slow)
        .map(|(fast, slow)| match (fast, slow) {
            (Some(fast), Some(slow)) => {
                let value = fast - slow;
                value.is_finite().then_some(value)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    if line.iter().any(Option::is_none)
        && line
            .iter()
            .zip(fast.iter().zip(&slow))
            .any(|(line, (fast, slow))| line.is_none() && fast.is_some() && slow.is_some())
    {
        return Err(IndicatorError::NonFiniteResult);
    }
    let signal = ema(&line, signal_period)?;
    let histogram = line
        .iter()
        .zip(&signal)
        .map(|(line, signal)| match (line, signal) {
            (Some(line), Some(signal)) => {
                let value = line - signal;
                value.is_finite().then_some(value)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    if histogram.iter().any(Option::is_none)
        && histogram
            .iter()
            .zip(line.iter().zip(&signal))
            .any(|(histogram, (line, signal))| {
                histogram.is_none() && line.is_some() && signal.is_some()
            })
    {
        return Err(IndicatorError::NonFiniteResult);
    }

    Ok(MacdSeries {
        line,
        signal,
        histogram,
    })
}

#[cfg(test)]
mod tests {
    use super::{IndicatorError, macd};

    #[test]
    fn macd_aligns_line_signal_and_histogram_without_future_values() {
        let result = macd(&[1.0, 2.0, 3.0, 4.0, 5.0].map(Some), 2, 3, 2).unwrap();
        assert_eq!(result.line, [None, None, Some(0.5), Some(0.5), Some(0.5)]);
        assert_eq!(result.signal, [None, None, None, Some(0.5), Some(0.5)]);
        assert_eq!(result.histogram, [None, None, None, Some(0.0), Some(0.0)]);
    }

    #[test]
    fn macd_rejects_incompatible_periods_and_detects_overflow() {
        assert_eq!(
            macd(&[Some(1.0)], 3, 2, 1),
            Err(IndicatorError::InvalidParameters)
        );
        let mut extreme = vec![Some(-f64::MAX); 10];
        extreme.push(Some(f64::MAX));
        assert_eq!(
            macd(&extreme, 1, 10, 2),
            Err(IndicatorError::NonFiniteResult)
        );
    }
}
