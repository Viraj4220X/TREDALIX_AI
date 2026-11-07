use serde::{Deserialize, Serialize};

use super::{CurrencyCode, InstrumentId, SourceMetadata};

pub const FORECAST_NOT_OBSERVED_LABEL: &str = "MODEL ESTIMATE — NOT OBSERVED MARKET DATA";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForecastTarget {
    ClosePrice,
    SimpleReturn,
    LogReturn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForecastHorizon {
    pub steps: u32,
    pub interval_seconds: u64,
}

impl ForecastHorizon {
    pub fn validate(&self) -> Result<(), String> {
        if self.steps == 0 || self.interval_seconds == 0 {
            return Err("Forecast horizon steps and interval must be positive".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct QuantileEstimate {
    pub quantile: f64,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForecastPoint {
    pub timestamp_utc: i64,
    pub point_estimate: Option<f64>,
    pub quantiles: Vec<QuantileEstimate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForecastSeries {
    pub instrument_id: InstrumentId,
    pub currency: CurrencyCode,
    pub origin_timestamp_utc: i64,
    pub input_data_end_timestamp_utc: i64,
    pub generated_at_utc: i64,
    pub horizon: ForecastHorizon,
    pub target: ForecastTarget,
    pub model_id: String,
    pub model_version: String,
    pub uncertainty_method: Option<String>,
    pub source: SourceMetadata,
    pub validation_report_id: Option<String>,
    pub limitations: Vec<String>,
    pub points: Vec<ForecastPoint>,
}

impl ForecastSeries {
    pub fn validate(&self) -> Result<(), String> {
        self.horizon.validate()?;
        if self.input_data_end_timestamp_utc > self.origin_timestamp_utc {
            return Err("Forecast input data cannot end after its origin timestamp".into());
        }
        if self.generated_at_utc < self.origin_timestamp_utc {
            return Err("Forecast generation time cannot precede its origin".into());
        }
        if self.model_id.trim().is_empty() || self.model_version.trim().is_empty() {
            return Err("Forecast model ID and version are required".into());
        }
        if self.points.len() != self.horizon.steps as usize {
            return Err("Forecast point count must match the configured horizon".into());
        }
        self.source.validate()?;

        let mut previous_timestamp = self.origin_timestamp_utc;
        for point in &self.points {
            if point.timestamp_utc <= previous_timestamp {
                return Err("Forecast timestamps must be strictly after origin and ordered".into());
            }
            if point.point_estimate.is_some_and(|value| !value.is_finite()) {
                return Err("Forecast point estimates must be finite or explicitly missing".into());
            }
            let mut previous_quantile = None;
            for estimate in &point.quantiles {
                if !estimate.quantile.is_finite()
                    || !(0.0..=1.0).contains(&estimate.quantile)
                    || !estimate.value.is_finite()
                    || previous_quantile.is_some_and(|q| estimate.quantile <= q)
                {
                    return Err(
                        "Forecast quantiles must be finite, ordered, and within [0, 1]".into(),
                    );
                }
                previous_quantile = Some(estimate.quantile);
            }
            previous_timestamp = point.timestamp_utc;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FORECAST_NOT_OBSERVED_LABEL, ForecastHorizon, ForecastPoint, ForecastSeries,
        ForecastTarget, QuantileEstimate,
    };
    use crate::domain::{CurrencyCode, DataFreshness, InstrumentId, SourceMetadata};

    fn forecast() -> ForecastSeries {
        ForecastSeries {
            instrument_id: InstrumentId::new("NSE", "TCS").unwrap(),
            currency: CurrencyCode::new("INR").unwrap(),
            origin_timestamp_utc: 100,
            input_data_end_timestamp_utc: 100,
            generated_at_utc: 101,
            horizon: ForecastHorizon {
                steps: 2,
                interval_seconds: 60,
            },
            target: ForecastTarget::ClosePrice,
            model_id: "naive-baseline".into(),
            model_version: "1".into(),
            uncertainty_method: None,
            source: SourceMetadata {
                source_name: "test-only".into(),
                provider_id: None,
                retrieved_at_utc: 100,
                freshness: DataFreshness::SyntheticTest,
                attribution: None,
            },
            validation_report_id: None,
            limitations: vec!["Test contract only; no forecast is produced.".into()],
            points: vec![
                ForecastPoint {
                    timestamp_utc: 160,
                    point_estimate: Some(100.0),
                    quantiles: vec![
                        QuantileEstimate {
                            quantile: 0.1,
                            value: 98.0,
                        },
                        QuantileEstimate {
                            quantile: 0.9,
                            value: 102.0,
                        },
                    ],
                },
                ForecastPoint {
                    timestamp_utc: 220,
                    point_estimate: None,
                    quantiles: vec![],
                },
            ],
        }
    }

    #[test]
    fn forecast_contract_round_trips_without_becoming_observed_data() {
        let forecast = forecast();
        forecast.validate().unwrap();
        let encoded = serde_json::to_string(&forecast).unwrap();
        let decoded: ForecastSeries = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, forecast);
        assert!(FORECAST_NOT_OBSERVED_LABEL.contains("NOT OBSERVED"));
    }

    #[test]
    fn forecast_rejects_origin_leakage_unordered_quantiles_and_wrong_horizon() {
        let mut invalid = forecast();
        invalid.points[0].timestamp_utc = 100;
        assert!(invalid.validate().is_err());

        let mut invalid = forecast();
        invalid.points[0].quantiles.reverse();
        assert!(invalid.validate().is_err());

        let mut invalid = forecast();
        invalid.points.pop();
        assert!(invalid.validate().is_err());
    }
}
