use serde::{Deserialize, Serialize};

use super::{CurrencyCode, InstrumentId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataFreshness {
    ProviderReportedLive,
    Delayed,
    EndOfDay,
    Cached,
    Stale,
    Unavailable,
    SyntheticTest,
    UserProvided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentMode {
    Unadjusted,
    SplitAdjusted,
    TotalReturnAdjusted,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceMetadata {
    pub source_name: String,
    pub provider_id: Option<String>,
    pub retrieved_at_utc: i64,
    pub freshness: DataFreshness,
    pub attribution: Option<String>,
}

impl SourceMetadata {
    pub fn validate(&self) -> Result<(), String> {
        if self.freshness == DataFreshness::Unavailable {
            return Err("Unavailable source status cannot accompany market data".into());
        }
        if self.source_name.trim().is_empty() {
            return Err("Source name cannot be empty".into());
        }
        if self
            .provider_id
            .as_ref()
            .is_some_and(|provider| provider.trim().is_empty())
        {
            return Err("Provider ID cannot be empty when provided".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedCandle {
    pub instrument_id: InstrumentId,
    pub start_utc: i64,
    pub end_utc: i64,
    pub interval_seconds: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: Option<f64>,
    pub currency: CurrencyCode,
    pub adjustment: AdjustmentMode,
    pub source: SourceMetadata,
}

impl NormalizedCandle {
    pub fn validate(&self) -> Result<(), String> {
        if self.interval_seconds == 0 || self.interval_seconds > i64::MAX as u64 {
            return Err("Candle interval must be in the supported positive timestamp range".into());
        }
        let expected_end = self
            .start_utc
            .checked_add(self.interval_seconds as i64)
            .ok_or_else(|| "Candle timestamp range overflows".to_owned())?;
        if self.end_utc != expected_end {
            return Err("Candle end timestamp must equal start plus its interval".into());
        }
        for (name, value) in [
            ("open", self.open),
            ("high", self.high),
            ("low", self.low),
            ("close", self.close),
        ] {
            if !value.is_finite() || value <= 0.0 {
                return Err(format!("Candle {name} must be finite and positive"));
            }
        }
        if self.high < self.open.max(self.close).max(self.low)
            || self.low > self.open.min(self.close).min(self.high)
        {
            return Err("Candle OHLC values are inconsistent".into());
        }
        if self
            .volume
            .is_some_and(|volume| !volume.is_finite() || volume < 0.0)
        {
            return Err("Candle volume must be finite and nonnegative".into());
        }
        self.source.validate()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct QuoteSize {
    pub bid: Option<f64>,
    pub ask: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quote {
    pub instrument_id: InstrumentId,
    pub price: f64,
    pub currency: CurrencyCode,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub size: Option<QuoteSize>,
    pub provider_event_time_utc: Option<i64>,
    pub source: SourceMetadata,
}

impl Quote {
    pub fn validate(&self) -> Result<(), String> {
        if !self.price.is_finite() || self.price <= 0.0 {
            return Err("Quote price must be finite and positive".into());
        }
        for (name, value) in [("bid", self.bid), ("ask", self.ask)] {
            if value.is_some_and(|price| !price.is_finite() || price <= 0.0) {
                return Err(format!("Quote {name} must be finite and positive"));
            }
        }
        if let (Some(bid), Some(ask)) = (self.bid, self.ask)
            && bid > ask
        {
            return Err("Quote bid cannot exceed ask".into());
        }
        if self.size.is_some_and(|size| {
            [size.bid, size.ask]
                .into_iter()
                .flatten()
                .any(|value| !value.is_finite() || value < 0.0)
        }) {
            return Err("Quote sizes must be finite and nonnegative".into());
        }
        self.source.validate()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AdjustmentMode, DataFreshness, NormalizedCandle, Quote, QuoteSize, SourceMetadata,
    };
    use crate::domain::{CurrencyCode, InstrumentId};

    fn source() -> SourceMetadata {
        SourceMetadata {
            source_name: "licensed-test-fixture".into(),
            provider_id: Some("fixture".into()),
            retrieved_at_utc: 1_800_000_000,
            freshness: DataFreshness::SyntheticTest,
            attribution: None,
        }
    }

    fn candle() -> NormalizedCandle {
        NormalizedCandle {
            instrument_id: InstrumentId::new("NSE", "TCS").unwrap(),
            start_utc: 1_800_000_000,
            end_utc: 1_800_000_060,
            interval_seconds: 60,
            open: 100.0,
            high: 103.0,
            low: 99.0,
            close: 102.0,
            volume: Some(0.0),
            currency: CurrencyCode::new("INR").unwrap(),
            adjustment: AdjustmentMode::Unadjusted,
            source: source(),
        }
    }

    #[test]
    fn normalized_candle_accepts_valid_zero_volume_and_round_trips() {
        let candle = candle();
        candle.validate().unwrap();
        let encoded = serde_json::to_string(&candle).unwrap();
        let decoded: NormalizedCandle = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, candle);
    }

    #[test]
    fn normalized_candle_rejects_invalid_ohlc_interval_and_volume() {
        let mut invalid = candle();
        invalid.high = 98.0;
        assert!(invalid.validate().is_err());

        let mut invalid = candle();
        invalid.end_utc += 1;
        assert!(invalid.validate().is_err());

        let mut invalid = candle();
        invalid.volume = Some(-1.0);
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn quote_validates_spread_sizes_and_source() {
        let quote = Quote {
            instrument_id: InstrumentId::new("NSE", "TCS").unwrap(),
            price: 101.0,
            currency: CurrencyCode::new("INR").unwrap(),
            bid: Some(100.0),
            ask: Some(102.0),
            size: Some(QuoteSize {
                bid: Some(0.0),
                ask: Some(10.0),
            }),
            provider_event_time_utc: None,
            source: source(),
        };
        quote.validate().unwrap();

        let mut crossed = quote;
        crossed.bid = Some(103.0);
        assert!(crossed.validate().is_err());
        crossed.bid = Some(100.0);
        crossed.source.freshness = DataFreshness::Unavailable;
        assert!(crossed.validate().is_err());
    }
}
