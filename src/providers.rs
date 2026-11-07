use std::collections::{HashMap, VecDeque};

use crate::domain::{
    AdjustmentMode, CompanyResearchSnapshot, InstrumentId, InstrumentIdentity, NormalizedCandle,
    Quote,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderError {
    AuthenticationRequired,
    RateLimited { retry_after_seconds: Option<u64> },
    Timeout,
    Unavailable,
    InvalidResponse(String),
    UnsupportedInstrument,
    UnsupportedInterval,
    TermsRestricted,
}

impl ProviderError {
    pub fn retryable(&self) -> bool {
        matches!(
            self,
            Self::RateLimited { .. } | Self::Timeout | Self::Unavailable
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryRequest {
    pub instrument: InstrumentId,
    pub interval_seconds: u64,
    pub start_utc: i64,
    pub end_utc: i64,
    pub adjustment: AdjustmentMode,
}

impl HistoryRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.interval_seconds == 0 || self.start_utc >= self.end_utc {
            return Err("History interval and requested time range must be valid".into());
        }
        Ok(())
    }
}

pub trait InstrumentSearchProvider {
    fn search(&self, query: &str, limit: u16) -> Result<Vec<InstrumentIdentity>, ProviderError>;
}

pub trait QuoteProvider {
    fn quote(&self, instrument: &InstrumentId) -> Result<Quote, ProviderError>;
}

pub trait HistoricalMarketDataProvider {
    fn history(&self, request: &HistoryRequest) -> Result<Vec<NormalizedCandle>, ProviderError>;
}

pub trait CompanyResearchProvider {
    fn research(&self, instrument: &InstrumentId)
    -> Result<CompanyResearchSnapshot, ProviderError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderCapabilities {
    pub provider_id: String,
    pub display_name: String,
    pub search: bool,
    pub quotes: bool,
    pub history: bool,
    pub fundamentals: bool,
    pub news: bool,
    pub supported_exchanges: Vec<String>,
    pub supported_intervals_seconds: Vec<u64>,
    pub attribution_url: Option<String>,
}

impl ProviderCapabilities {
    pub fn validate(&self) -> Result<(), String> {
        if self.provider_id.trim().is_empty()
            || self.display_name.trim().is_empty()
            || self.provider_id.len() > 64
            || self.display_name.len() > 120
        {
            return Err("Provider ID and display name are required and bounded".into());
        }
        if self.history && self.supported_intervals_seconds.is_empty() {
            return Err("History capability requires declared interval support".into());
        }
        if self.supported_intervals_seconds.contains(&0) {
            return Err("Provider intervals must be positive".into());
        }
        if self
            .attribution_url
            .as_ref()
            .is_some_and(|url| !url.starts_with("https://"))
        {
            return Err("Provider attribution links must use HTTPS".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CandleCacheKey {
    pub instrument: InstrumentId,
    pub interval_seconds: u64,
    pub start_utc: i64,
    pub end_utc: i64,
    pub adjustment: AdjustmentMode,
    pub source_policy: String,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    candles: Vec<NormalizedCandle>,
    fetched_at_utc: i64,
    expires_at_utc: i64,
}

#[derive(Debug, Clone)]
pub struct CandleCache {
    max_entries: usize,
    max_candles_per_entry: usize,
    ttl_seconds: u64,
    entries: HashMap<CandleCacheKey, CacheEntry>,
    insertion_order: VecDeque<CandleCacheKey>,
}

impl CandleCache {
    pub fn new(
        max_entries: usize,
        max_candles_per_entry: usize,
        ttl_seconds: u64,
    ) -> Result<Self, String> {
        if max_entries == 0 || max_candles_per_entry == 0 || ttl_seconds == 0 {
            return Err("Cache limits and TTL must be positive".into());
        }
        Ok(Self {
            max_entries,
            max_candles_per_entry,
            ttl_seconds,
            entries: HashMap::new(),
            insertion_order: VecDeque::new(),
        })
    }

    pub fn insert(
        &mut self,
        key: CandleCacheKey,
        candles: Vec<NormalizedCandle>,
        fetched_at_utc: i64,
        cache_permitted: bool,
    ) -> Result<(), String> {
        if !cache_permitted {
            return Err("Provider terms do not permit caching this response".into());
        }
        if key.interval_seconds == 0
            || key.start_utc >= key.end_utc
            || key.source_policy.trim().is_empty()
            || candles.is_empty()
            || candles.len() > self.max_candles_per_entry
        {
            return Err("Cache key or candle count is invalid".into());
        }
        let expires_at_utc = fetched_at_utc
            .checked_add(
                i64::try_from(self.ttl_seconds)
                    .map_err(|_| "Cache TTL exceeds timestamp range".to_owned())?,
            )
            .ok_or_else(|| "Cache expiry timestamp overflows".to_owned())?;
        let mut previous_timestamp = None;
        for candle in &candles {
            candle.validate()?;
            if candle.instrument_id != key.instrument
                || candle.interval_seconds != key.interval_seconds
                || candle.adjustment != key.adjustment
                || candle.start_utc < key.start_utc
                || candle.start_utc >= key.end_utc
                || candle.end_utc > key.end_utc
                || previous_timestamp.is_some_and(|timestamp| candle.start_utc <= timestamp)
            {
                return Err("Candle does not match the cache key or chronological range".into());
            }
            previous_timestamp = Some(candle.start_utc);
        }
        if let Some(existing) = self.entries.get_mut(&key) {
            existing.candles = candles;
            existing.fetched_at_utc = fetched_at_utc;
            existing.expires_at_utc = expires_at_utc;
            return Ok(());
        }
        while self.entries.len() >= self.max_entries {
            if let Some(oldest) = self.insertion_order.pop_front() {
                self.entries.remove(&oldest);
            }
        }
        self.insertion_order.push_back(key.clone());
        self.entries.insert(
            key,
            CacheEntry {
                candles,
                fetched_at_utc,
                expires_at_utc,
            },
        );
        Ok(())
    }

    pub fn get(
        &self,
        key: &CandleCacheKey,
        now_utc: i64,
        allow_stale: bool,
    ) -> Option<Vec<NormalizedCandle>> {
        let entry = self.entries.get(key)?;
        if now_utc > entry.expires_at_utc && !allow_stale {
            return None;
        }
        let mut candles = entry.candles.clone();
        let freshness = if now_utc > entry.expires_at_utc {
            crate::domain::DataFreshness::Stale
        } else {
            crate::domain::DataFreshness::Cached
        };
        for candle in &mut candles {
            candle.source.retrieved_at_utc = entry.fetched_at_utc;
            candle.source.freshness = freshness;
        }
        Some(candles)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{CandleCache, CandleCacheKey, ProviderCapabilities};
    use crate::domain::{
        AdjustmentMode, CurrencyCode, DataFreshness, InstrumentId, NormalizedCandle, SourceMetadata,
    };

    fn key(symbol: &str) -> CandleCacheKey {
        CandleCacheKey {
            instrument: InstrumentId::new("NSE", symbol).unwrap(),
            interval_seconds: 60,
            start_utc: 0,
            end_utc: 600,
            adjustment: AdjustmentMode::Unadjusted,
            source_policy: "licensed-test-provider".into(),
        }
    }

    fn candle(symbol: &str) -> NormalizedCandle {
        NormalizedCandle {
            instrument_id: InstrumentId::new("NSE", symbol).unwrap(),
            start_utc: 0,
            end_utc: 60,
            interval_seconds: 60,
            open: 10.0,
            high: 11.0,
            low: 9.0,
            close: 10.5,
            volume: None,
            currency: CurrencyCode::new("INR").unwrap(),
            adjustment: AdjustmentMode::Unadjusted,
            source: SourceMetadata {
                source_name: "test-provider".into(),
                provider_id: Some("test".into()),
                retrieved_at_utc: 1,
                freshness: DataFreshness::ProviderReportedLive,
                attribution: None,
            },
        }
    }

    #[test]
    fn cache_marks_fresh_and_stale_results_and_enforces_ttl() {
        let mut cache = CandleCache::new(2, 10, 5).unwrap();
        cache
            .insert(key("TCS"), vec![candle("TCS")], 100, true)
            .unwrap();
        assert_eq!(
            cache.get(&key("TCS"), 105, false).unwrap()[0]
                .source
                .freshness,
            DataFreshness::Cached
        );
        assert!(cache.get(&key("TCS"), 106, false).is_none());
        assert_eq!(
            cache.get(&key("TCS"), 106, true).unwrap()[0]
                .source
                .freshness,
            DataFreshness::Stale
        );
    }

    #[test]
    fn cache_is_bounded_deterministic_and_respects_provider_terms() {
        let mut cache = CandleCache::new(1, 10, 30).unwrap();
        cache
            .insert(key("TCS"), vec![candle("TCS")], 100, true)
            .unwrap();
        cache
            .insert(key("INFY"), vec![candle("INFY")], 100, true)
            .unwrap();
        assert_eq!(cache.len(), 1);
        assert!(cache.get(&key("TCS"), 101, true).is_none());
        assert!(
            cache
                .insert(key("SBIN"), vec![candle("SBIN")], 100, false)
                .unwrap_err()
                .contains("do not permit caching")
        );
    }

    #[test]
    fn cache_rejects_mismatched_instrument_and_invalid_provider_capabilities() {
        let mut cache = CandleCache::new(1, 10, 30).unwrap();
        assert!(
            cache
                .insert(key("TCS"), vec![candle("INFY")], 100, true)
                .is_err()
        );
        let mut short_range = key("TCS");
        short_range.end_utc = 30;
        assert!(
            cache
                .insert(short_range, vec![candle("TCS")], 100, true)
                .is_err()
        );
        let capabilities = ProviderCapabilities {
            provider_id: "provider".into(),
            display_name: "Provider".into(),
            search: false,
            quotes: false,
            history: true,
            fundamentals: false,
            news: false,
            supported_exchanges: vec![],
            supported_intervals_seconds: vec![],
            attribution_url: Some("http://invalid.example".into()),
        };
        assert!(capabilities.validate().is_err());
    }
}
