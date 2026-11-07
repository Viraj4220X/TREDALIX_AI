use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct ExchangeCode(String);

impl ExchangeCode {
    pub fn new(value: impl AsRef<str>) -> Result<Self, String> {
        let value = value.as_ref().trim();
        if value.is_empty()
            || value.len() > 16
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'.' || byte == b'-')
        {
            return Err(
                "Exchange code must be 1-16 ASCII letters, digits, dots, or hyphens".into(),
            );
        }
        Ok(Self(value.to_ascii_uppercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ExchangeCode {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct Symbol(String);

impl Symbol {
    pub fn new(value: impl AsRef<str>) -> Result<Self, String> {
        let value = value.as_ref().trim();
        if value.is_empty()
            || value.len() > 32
            || !value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b'^')
            })
        {
            return Err("Symbol must be 1-32 supported ASCII symbol characters".into());
        }
        Ok(Self(value.to_ascii_uppercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Symbol {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstrumentId {
    pub exchange: ExchangeCode,
    pub symbol: Symbol,
}

impl InstrumentId {
    pub fn new(exchange: impl AsRef<str>, symbol: impl AsRef<str>) -> Result<Self, String> {
        Ok(Self {
            exchange: ExchangeCode::new(exchange)?,
            symbol: Symbol::new(symbol)?,
        })
    }

    pub fn canonical_key(&self) -> String {
        format!("{}:{}", self.exchange.as_str(), self.symbol.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct CurrencyCode(String);

impl CurrencyCode {
    pub fn new(value: impl AsRef<str>) -> Result<Self, String> {
        let value = value.as_ref().trim();
        if value.len() != 3 || !value.bytes().all(|byte| byte.is_ascii_alphabetic()) {
            return Err("Currency must be a three-letter ISO-style ASCII code".into());
        }
        Ok(Self(value.to_ascii_uppercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for CurrencyCode {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    Equity,
    Index,
    Etf,
    Fund,
    Bond,
    Currency,
    Commodity,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListingStatus {
    Active,
    Suspended,
    Delisted,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderInstrumentId {
    pub provider: String,
    pub provider_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstrumentIdentity {
    pub id: InstrumentId,
    pub display_name: String,
    pub country_code: Option<String>,
    pub currency: Option<CurrencyCode>,
    pub asset_type: AssetType,
    pub listing_status: ListingStatus,
    pub primary_listing: Option<bool>,
    pub provider_ids: Vec<ProviderInstrumentId>,
}

impl InstrumentIdentity {
    pub fn validate(&self) -> Result<(), String> {
        if self.display_name.trim().is_empty() || self.display_name.len() > 200 {
            return Err("Instrument display name must be 1-200 characters".into());
        }
        if self.country_code.as_ref().is_some_and(|country| {
            country.len() != 2 || !country.bytes().all(|byte| byte.is_ascii_alphabetic())
        }) {
            return Err("Country code must contain two ASCII letters".into());
        }
        let mut provider_keys = std::collections::HashSet::new();
        for provider_id in &self.provider_ids {
            if provider_id.provider.trim().is_empty()
                || provider_id.provider_id.trim().is_empty()
                || !provider_keys.insert((
                    provider_id.provider.to_ascii_lowercase(),
                    provider_id.provider_id.clone(),
                ))
            {
                return Err("Provider instrument IDs must be nonempty and unique".into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{InstrumentId, InstrumentIdentity, Symbol};

    #[test]
    fn canonical_identity_keeps_same_symbol_on_exchanges_distinct() {
        let nse = InstrumentId::new("nse", "tcs").unwrap();
        let bse = InstrumentId::new("BSE", "TCS").unwrap();

        assert_eq!(nse.canonical_key(), "NSE:TCS");
        assert_ne!(nse, bse);
    }

    #[test]
    fn symbols_reject_unsupported_or_ambiguous_input() {
        assert!(Symbol::new("  ").is_err());
        assert!(Symbol::new("TCS/INR").is_err());
        assert!(Symbol::new("東京").is_err());
    }

    #[test]
    fn identity_json_round_trip_preserves_provider_mapping() {
        let identity = InstrumentIdentity {
            id: InstrumentId::new("NSE", "TCS").unwrap(),
            display_name: "Tata Consultancy Services".into(),
            country_code: Some("IN".into()),
            currency: Some(super::CurrencyCode::new("inr").unwrap()),
            asset_type: super::AssetType::Equity,
            listing_status: super::ListingStatus::Active,
            primary_listing: Some(true),
            provider_ids: vec![super::ProviderInstrumentId {
                provider: "test-provider".into(),
                provider_id: "equity-123".into(),
            }],
        };
        let encoded = serde_json::to_string(&identity).unwrap();
        let decoded: InstrumentIdentity = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded, identity);
        decoded.validate().unwrap();
    }

    #[test]
    fn deserialization_cannot_bypass_symbol_validation() {
        assert!(serde_json::from_str::<Symbol>(r#""TCS/INR""#).is_err());
    }

    #[test]
    fn identity_rejects_invalid_country_and_duplicate_provider_mapping() {
        let mut identity = InstrumentIdentity {
            id: InstrumentId::new("NSE", "TCS").unwrap(),
            display_name: "TCS".into(),
            country_code: Some("India".into()),
            currency: None,
            asset_type: super::AssetType::Equity,
            listing_status: super::ListingStatus::Active,
            primary_listing: None,
            provider_ids: vec![],
        };
        assert!(identity.validate().is_err());
        identity.country_code = Some("IN".into());
        identity.provider_ids = vec![
            super::ProviderInstrumentId {
                provider: "source".into(),
                provider_id: "123".into(),
            },
            super::ProviderInstrumentId {
                provider: "SOURCE".into(),
                provider_id: "123".into(),
            },
        ];
        assert!(identity.validate().is_err());
    }
}
