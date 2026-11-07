use serde::{Deserialize, Serialize};

use super::{CurrencyCode, InstrumentId, SourceMetadata};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FundamentalPeriod {
    Annual,
    Quarterly,
    TrailingTwelveMonths,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompanyProfile {
    pub legal_name: String,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub website: Option<String>,
    pub currency: CurrencyCode,
    pub source: SourceMetadata,
}

impl CompanyProfile {
    pub fn validate(&self) -> Result<(), String> {
        if self.legal_name.trim().is_empty() || self.legal_name.len() > 240 {
            return Err("Company name must be 1-240 characters".into());
        }
        for value in [self.sector.as_ref(), self.industry.as_ref()]
            .into_iter()
            .flatten()
        {
            if value.trim().is_empty() || value.len() > 120 {
                return Err("Company classification must be 1-120 characters".into());
            }
        }
        if self.website.as_ref().is_some_and(|url| {
            url.len() > 2048 || !url.starts_with("https://") || url.chars().any(char::is_control)
        }) {
            return Err("Company website must be a bounded HTTPS URL".into());
        }
        self.source.validate()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FundamentalFact {
    pub metric: String,
    pub value: f64,
    pub unit: String,
    pub period: FundamentalPeriod,
    pub period_end_utc: i64,
    pub reported_at_utc: Option<i64>,
    pub source: SourceMetadata,
}

impl FundamentalFact {
    pub fn validate(&self) -> Result<(), String> {
        if self.metric.trim().is_empty()
            || self.metric.len() > 120
            || self.unit.trim().is_empty()
            || self.unit.len() > 32
        {
            return Err("Fundamental metric and unit must be present and bounded".into());
        }
        if !self.value.is_finite() {
            return Err("Fundamental value must be finite".into());
        }
        if self
            .reported_at_utc
            .is_some_and(|reported| reported < self.period_end_utc)
        {
            return Err("Fundamental report timestamp cannot precede its period end".into());
        }
        self.source.validate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsArticle {
    pub title: String,
    pub url: String,
    pub publisher: String,
    pub published_at_utc: i64,
    pub source: SourceMetadata,
}

impl NewsArticle {
    pub fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty()
            || self.title.len() > 500
            || self.publisher.trim().is_empty()
            || self.publisher.len() > 160
            || self.url.len() > 2048
            || !self.url.starts_with("https://")
            || self.url.chars().any(char::is_control)
        {
            return Err("News title, publisher, and HTTPS URL must be valid and bounded".into());
        }
        self.source.validate()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompanyResearchSnapshot {
    pub instrument_id: InstrumentId,
    pub retrieved_at_utc: i64,
    pub profile: Option<CompanyProfile>,
    pub fundamentals: Vec<FundamentalFact>,
    pub news: Vec<NewsArticle>,
}

impl CompanyResearchSnapshot {
    pub fn validate(&self) -> Result<(), String> {
        if self.fundamentals.len() > 10_000 || self.news.len() > 1_000 {
            return Err("Company research response exceeds supported item limits".into());
        }
        if let Some(profile) = &self.profile {
            profile.validate()?;
        }
        for fact in &self.fundamentals {
            fact.validate()?;
        }
        for article in &self.news {
            article.validate()?;
        }
        Ok(())
    }
}

pub fn safe_ratio(numerator: f64, denominator: f64) -> Option<f64> {
    if !numerator.is_finite() || !denominator.is_finite() || denominator == 0.0 {
        return None;
    }
    let ratio = numerator / denominator;
    ratio.is_finite().then_some(ratio)
}

#[cfg(test)]
mod tests {
    use super::{FundamentalFact, FundamentalPeriod, NewsArticle, safe_ratio};
    use crate::domain::{DataFreshness, SourceMetadata};

    fn source() -> SourceMetadata {
        SourceMetadata {
            source_name: "fixture provider".into(),
            provider_id: Some("fixture".into()),
            retrieved_at_utc: 100,
            freshness: DataFreshness::EndOfDay,
            attribution: None,
        }
    }

    #[test]
    fn sourced_financial_facts_require_finite_values_and_valid_periods() {
        let fact = FundamentalFact {
            metric: "revenue".into(),
            value: 125.0,
            unit: "USD".into(),
            period: FundamentalPeriod::Annual,
            period_end_utc: 50,
            reported_at_utc: Some(75),
            source: source(),
        };
        fact.validate().unwrap();
        assert!(
            FundamentalFact {
                value: f64::NAN,
                ..fact.clone()
            }
            .validate()
            .is_err()
        );
        assert!(
            FundamentalFact {
                reported_at_utc: Some(49),
                ..fact
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn news_links_must_be_secure_and_ratios_never_fabricate_invalid_values() {
        let article = NewsArticle {
            title: "Company update".into(),
            url: "https://example.test/article".into(),
            publisher: "Example News".into(),
            published_at_utc: 100,
            source: source(),
        };
        article.validate().unwrap();
        assert!(
            NewsArticle {
                url: "javascript:alert(1)".into(),
                ..article
            }
            .validate()
            .is_err()
        );
        assert_eq!(safe_ratio(10.0, 2.0), Some(5.0));
        assert_eq!(safe_ratio(1.0, 0.0), None);
        assert_eq!(safe_ratio(f64::INFINITY, 2.0), None);
    }
}
