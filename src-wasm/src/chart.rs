use serde_json::{Value, json};

use tredalix_ai::{
    data::Candle,
    indicators::{atr, bollinger_bands, ema, macd, rsi, sma},
};

const MAX_CHART_CANDLES: usize = 1_200;

fn sampled_chart(candles: &[Candle]) -> Result<Vec<(usize, Candle)>, String> {
    let stride = candles.len().div_ceil(MAX_CHART_CANDLES);
    candles
        .chunks(stride)
        .enumerate()
        .map(|(bucket_index, bucket)| {
            let first = &bucket[0];
            let last = bucket.last().expect("chart bucket cannot be empty");
            let volume = if bucket.iter().all(|candle| candle.volume.is_some()) {
                Some(
                    bucket
                        .iter()
                        .map(|candle| candle.volume.expect("volume presence was checked"))
                        .sum::<f64>(),
                )
            } else {
                None
            };
            if volume.is_some_and(|total| !total.is_finite()) {
                return Err("Aggregated chart volume exceeds the supported numeric range".into());
            }
            Ok((
                ((bucket_index + 1) * stride - 1).min(candles.len() - 1),
                Candle {
                    timestamp: last.timestamp,
                    open: first.open,
                    high: bucket
                        .iter()
                        .map(|candle| candle.high)
                        .fold(f64::NEG_INFINITY, f64::max),
                    low: bucket
                        .iter()
                        .map(|candle| candle.low)
                        .fold(f64::INFINITY, f64::min),
                    close: last.close,
                    volume,
                },
            ))
        })
        .collect()
}

fn indicator_points(sampled: &[(usize, Candle)], values: &[Option<f64>]) -> Vec<Value> {
    sampled
        .iter()
        .filter_map(|(index, candle)| {
            values[*index].map(|value| json!({ "time": candle.timestamp, "value": value }))
        })
        .collect()
}

pub fn build_chart(candles: &[Candle], indicators_enabled: bool) -> Result<Value, String> {
    if candles.is_empty() {
        return Err("Chart requires at least one validated candle".into());
    }
    let closes = candles
        .iter()
        .map(|candle| Some(candle.close))
        .collect::<Vec<_>>();
    let sampled = sampled_chart(candles)?;
    let candle_points = sampled
        .iter()
        .map(|(_, candle)| {
            json!({
                "time": candle.timestamp,
                "open": candle.open,
                "high": candle.high,
                "low": candle.low,
                "close": candle.close,
                "volume": candle.volume
            })
        })
        .collect::<Vec<_>>();
    if !indicators_enabled {
        return Ok(json!({
            "source": "validated local CSV",
            "sampled": sampled.len() < candles.len(),
            "candles": candle_points,
            "sma_20": [],
            "ema_20": [],
            "bollinger_upper_20": [],
            "bollinger_lower_20": [],
            "rsi_14": [],
            "macd_line": [],
            "macd_signal": [],
            "macd_histogram": [],
            "atr_14": []
        }));
    }
    let average_true_range =
        atr(candles, 14).map_err(|error| format!("Cannot calculate ATR: {error}"))?;
    let bands = bollinger_bands(&closes, 20, 2.0)
        .map_err(|error| format!("Cannot calculate Bollinger bands: {error}"))?;
    let ema_20 = ema(&closes, 20).map_err(|error| format!("Cannot calculate EMA: {error}"))?;
    let macd =
        macd(&closes, 12, 26, 9).map_err(|error| format!("Cannot calculate MACD: {error}"))?;
    let rsi_14 = rsi(&closes, 14).map_err(|error| format!("Cannot calculate RSI: {error}"))?;
    let sma_20 = sma(&closes, 20).map_err(|error| format!("Cannot calculate SMA: {error}"))?;
    Ok(json!({
        "source": "validated local CSV",
        "sampled": sampled.len() < candles.len(),
        "candles": candle_points,
        "sma_20": indicator_points(&sampled, &sma_20),
        "ema_20": indicator_points(&sampled, &ema_20),
        "bollinger_upper_20": indicator_points(&sampled, &bands.upper),
        "bollinger_lower_20": indicator_points(&sampled, &bands.lower),
        "rsi_14": indicator_points(&sampled, &rsi_14),
        "macd_line": indicator_points(&sampled, &macd.line),
        "macd_signal": indicator_points(&sampled, &macd.signal),
        "macd_histogram": indicator_points(&sampled, &macd.histogram),
        "atr_14": indicator_points(&sampled, &average_true_range)
    }))
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use tredalix_ai::data::Candle;

    use super::{MAX_CHART_CANDLES, build_chart};

    fn candles(count: usize) -> Vec<Candle> {
        (0..count)
            .map(|index| {
                let value = 100.0 + index as f64;
                Candle {
                    timestamp: index as i64 * 60,
                    open: value,
                    high: value + 1.0,
                    low: value - 1.0,
                    close: value + 0.5,
                    volume: Some(10.0),
                }
            })
            .collect()
    }

    #[test]
    fn computes_aligned_technical_series_from_validated_ohlc() {
        let chart = build_chart(&candles(30), true).unwrap();
        let chart: Value = chart;
        assert_eq!(chart["candles"].as_array().unwrap().len(), 30);
        assert_eq!(chart["sma_20"].as_array().unwrap().len(), 11);
        assert_eq!(chart["rsi_14"].as_array().unwrap().len(), 16);
        assert_eq!(chart["candles"][0]["high"], 101.0);
        assert_eq!(chart["candles"][0]["volume"], 10.0);
    }

    #[test]
    fn bounds_chart_payload_and_aggregates_ohlc_without_losing_extremes() {
        let chart = build_chart(&candles(MAX_CHART_CANDLES + 1), true).unwrap();
        assert!(chart["candles"].as_array().unwrap().len() <= MAX_CHART_CANDLES);
        assert_eq!(chart["sampled"], true);
        assert_eq!(chart["candles"][0]["high"], 102.0);
        assert_eq!(chart["candles"][0]["low"], 99.0);
    }

    #[test]
    fn disabled_indicators_return_only_candles() {
        let chart = build_chart(&candles(30), false).unwrap();
        assert_eq!(chart["candles"].as_array().unwrap().len(), 30);
        assert_eq!(chart["sma_20"].as_array().unwrap().len(), 0);
    }
}
