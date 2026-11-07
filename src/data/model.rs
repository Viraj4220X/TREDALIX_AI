#[derive(Debug, Clone, PartialEq)]
pub struct Candle {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataClass {
    Unknown,
    Synthetic,
}

impl DataClass {
    pub fn label(self) -> &'static str {
        match self {
            Self::Unknown => "UNKNOWN / UNVERIFIED",
            Self::Synthetic => "SYNTHETIC / TEST",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimestampMeaning {
    CandleOpen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstrumentType {
    Unknown,
    CashEquity,
    Index,
}

impl InstrumentType {
    pub fn label(self) -> &'static str {
        match self {
            Self::Unknown => "UNKNOWN",
            Self::CashEquity => "CASH EQUITY",
            Self::Index => "INDEX",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimestampTimezone {
    AsiaKolkata,
}

impl TimestampTimezone {
    pub fn label(self) -> &'static str {
        match self {
            Self::AsiaKolkata => "Asia/Kolkata (exchange-local, UTC+05:30)",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetMetadata {
    pub instrument: String,
    pub instrument_type: InstrumentType,
    pub source: String,
    pub interval_seconds: Option<u64>,
    pub data_class: DataClass,
    pub timestamp_timezone: TimestampTimezone,
    pub timestamp_meaning: TimestampMeaning,
}

impl DatasetMetadata {
    pub fn unverified(instrument: impl Into<String>) -> Self {
        Self {
            instrument: instrument.into(),
            instrument_type: InstrumentType::Unknown,
            source: "UNKNOWN".to_owned(),
            interval_seconds: None,
            data_class: DataClass::Unknown,
            timestamp_timezone: TimestampTimezone::AsiaKolkata,
            timestamp_meaning: TimestampMeaning::CandleOpen,
        }
    }
}

#[derive(Debug)]
pub struct Dataset {
    pub metadata: DatasetMetadata,
    pub candles: Vec<Candle>,
    pub quality: crate::data::DataQualitySummary,
}
