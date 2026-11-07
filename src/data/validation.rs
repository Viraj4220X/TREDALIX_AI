use super::model::Candle;
use super::session::{GapSummary, inspect_gaps};

#[derive(Debug)]
pub struct DataQualitySummary {
    pub row_count: usize,
    pub first_timestamp: i64,
    pub last_timestamp: i64,
    pub gaps: GapSummary,
}

pub fn validate_dataset(
    candles: &[Candle],
    interval_seconds: Option<u64>,
) -> Result<DataQualitySummary, String> {
    if candles.is_empty() {
        return Err("Dataset contains no candles".to_owned());
    }
    for (index, candle) in candles.iter().enumerate() {
        validate_candle(candle).map_err(|error| format!("Row {}: {error}", index + 2))?;
        if let Some(previous) = index.checked_sub(1).map(|previous| &candles[previous]) {
            if candle.timestamp == previous.timestamp {
                return Err(format!("Row {}: duplicate timestamp", index + 2));
            }
            if candle.timestamp < previous.timestamp {
                return Err(format!("Row {}: timestamps are not ascending", index + 2));
            }
        }
    }
    Ok(DataQualitySummary {
        row_count: candles.len(),
        first_timestamp: candles[0].timestamp,
        last_timestamp: candles[candles.len() - 1].timestamp,
        gaps: inspect_gaps(candles, interval_seconds),
    })
}

pub(super) fn validate_candle(candle: &Candle) -> Result<(), String> {
    for (name, value) in [
        ("open", candle.open),
        ("high", candle.high),
        ("low", candle.low),
        ("close", candle.close),
    ] {
        if !value.is_finite() || value <= 0.0 {
            return Err(format!("{name} must be finite and greater than zero"));
        }
    }
    if candle.high < candle.open
        || candle.high < candle.close
        || candle.high < candle.low
        || candle.low > candle.open
        || candle.low > candle.close
    {
        return Err("invalid OHLC relationship".to_owned());
    }
    if candle
        .volume
        .is_some_and(|volume| !volume.is_finite() || volume < 0.0)
    {
        return Err("volume must be finite and nonnegative".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_candle, validate_dataset};
    use crate::data::Candle;

    fn candle(timestamp: i64) -> Candle {
        Candle {
            timestamp,
            open: 10.0,
            high: 12.0,
            low: 9.0,
            close: 11.0,
            volume: Some(0.0),
        }
    }

    #[test]
    fn validates_ohlc_and_optional_volume() {
        assert!(validate_candle(&candle(1)).is_ok());
        let mut bad = candle(1);
        bad.high = 10.5;
        assert!(validate_candle(&bad).unwrap_err().contains("OHLC"));
        let mut bad_volume = candle(1);
        bad_volume.volume = Some(-1.0);
        assert!(validate_candle(&bad_volume).unwrap_err().contains("volume"));
        let mut missing_volume = candle(1);
        missing_volume.volume = None;
        assert!(validate_candle(&missing_volume).is_ok());
    }

    #[test]
    fn reports_dataset_gap_summary() {
        let summary = validate_dataset(&[candle(0), candle(120)], Some(60)).unwrap();
        assert_eq!(summary.gaps.unobserved_intervals, 1);
    }
}
