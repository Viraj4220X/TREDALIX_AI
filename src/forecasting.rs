use crate::{
    data::{Candle, validate_dataset},
    domain::{
        CurrencyCode, DataFreshness, ForecastHorizon, ForecastPoint, ForecastSeries,
        ForecastTarget, InstrumentId, SourceMetadata,
    },
};

pub fn naive_random_walk(
    candles: &[Candle],
    instrument_id: InstrumentId,
    currency: CurrencyCode,
    interval_seconds: u64,
    steps: u32,
    generated_at_utc: i64,
    source: SourceMetadata,
) -> Result<ForecastSeries, String> {
    if steps == 0 || steps > 20 {
        return Err("Naive forecast horizon must be between 1 and 20 bars".into());
    }
    validate_dataset(candles, Some(interval_seconds))?;
    if candles.len() < 2 {
        return Err("Naive random-walk baseline requires at least two validated bars".into());
    }
    let interval = i64::try_from(interval_seconds)
        .map_err(|_| "Forecast interval exceeds supported timestamp range".to_owned())?;
    let last = candles
        .last()
        .ok_or_else(|| "Forecast input contains no candles".to_owned())?;
    let origin_timestamp_utc = last
        .timestamp
        .checked_add(interval)
        .ok_or_else(|| "Forecast origin timestamp overflows".to_owned())?;
    if generated_at_utc < origin_timestamp_utc {
        return Err("Forecast generation time cannot precede the data origin".into());
    }
    source.validate()?;

    let mut points = Vec::with_capacity(steps as usize);
    for step in 1..=steps {
        let offset = interval
            .checked_mul(i64::from(step))
            .ok_or_else(|| "Forecast horizon timestamp overflows".to_owned())?;
        let timestamp_utc = origin_timestamp_utc
            .checked_add(offset)
            .ok_or_else(|| "Forecast point timestamp overflows".to_owned())?;
        points.push(ForecastPoint {
            timestamp_utc,
            point_estimate: Some(last.close),
            quantiles: vec![],
        });
    }

    let forecast = ForecastSeries {
        instrument_id,
        currency,
        origin_timestamp_utc,
        input_data_end_timestamp_utc: origin_timestamp_utc,
        generated_at_utc,
        horizon: ForecastHorizon {
            steps,
            interval_seconds,
        },
        target: ForecastTarget::ClosePrice,
        model_id: "naive_random_walk".into(),
        model_version: "1".into(),
        uncertainty_method: None,
        source,
        validation_report_id: None,
        limitations: vec![
            "This persistence baseline repeats the last observed close; it is not a trained prediction model.".into(),
            "No uncertainty interval, market-session calendar, corporate-action adjustment, or trading-cost model is provided.".into(),
            "Forecast timestamps advance by bar interval and may not match the next exchange session.".into(),
            "The output is an estimate, not observed market data or investment advice.".into(),
        ],
        points,
    };
    forecast.validate()?;
    Ok(forecast)
}

pub fn local_csv_source(
    source_name: impl Into<String>,
    synthetic: bool,
    retrieved_at_utc: i64,
) -> SourceMetadata {
    SourceMetadata {
        source_name: source_name.into(),
        provider_id: None,
        retrieved_at_utc,
        freshness: if synthetic {
            DataFreshness::SyntheticTest
        } else {
            DataFreshness::UserProvided
        },
        attribution: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{local_csv_source, naive_random_walk};
    use crate::data::Candle;
    use crate::domain::{CurrencyCode, InstrumentId};

    fn candles() -> Vec<Candle> {
        [10.0, 11.0]
            .into_iter()
            .enumerate()
            .map(|(index, close)| Candle {
                timestamp: index as i64 * 60,
                open: close,
                high: close,
                low: close,
                close,
                volume: None,
            })
            .collect()
    }

    #[test]
    fn random_walk_is_deterministic_and_stays_separate_from_observations() {
        let result = naive_random_walk(
            &candles(),
            InstrumentId::new("NSE", "TEST").unwrap(),
            CurrencyCode::new("INR").unwrap(),
            60,
            2,
            180,
            local_csv_source("synthetic fixture", true, 120),
        )
        .unwrap();
        assert_eq!(result.origin_timestamp_utc, 120);
        assert_eq!(result.points[0].timestamp_utc, 180);
        assert_eq!(result.points[1].timestamp_utc, 240);
        assert_eq!(result.points[0].point_estimate, Some(11.0));
        assert!(result.points.iter().all(|point| point.quantiles.is_empty()));
        result.validate().unwrap();
    }

    #[test]
    fn rejects_insufficient_input_invalid_horizon_and_generation_before_origin() {
        let one_bar = &candles()[..1];
        assert!(
            naive_random_walk(
                one_bar,
                InstrumentId::new("NSE", "TEST").unwrap(),
                CurrencyCode::new("INR").unwrap(),
                60,
                1,
                60,
                local_csv_source("fixture", true, 0),
            )
            .is_err()
        );
        assert!(
            naive_random_walk(
                &candles(),
                InstrumentId::new("NSE", "TEST").unwrap(),
                CurrencyCode::new("INR").unwrap(),
                60,
                1,
                119,
                local_csv_source("fixture", true, 0),
            )
            .is_err()
        );
    }
}
