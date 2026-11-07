use serde::{Deserialize, Serialize};

use crate::{
    data::{DataClass, Dataset},
    indicators::{atr, rsi, sma},
};

mod intent;

pub const ANALYST_VERSION: &str = "local-analyst-v1";
const MAX_QUESTION_CHARS: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisIntent {
    DatasetSummary,
    PriceChange,
    Trend,
    Risk,
    Indicators,
    Forecast,
    Causality,
    Comparison,
    CompanyResearch,
    SafetyBoundary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimKind {
    Observation,
    Calculation,
    Interpretation,
    Limitation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisEvidence {
    pub id: String,
    pub kind: ClaimKind,
    pub label: String,
    pub value: f64,
    pub unit: String,
    pub period_start_utc: i64,
    pub period_end_utc: i64,
    pub source_id: String,
    pub source_name: String,
    pub as_of_utc: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisClaim {
    pub id: String,
    pub text: String,
    pub kind: ClaimKind,
    pub evidence_ids: Vec<String>,
    pub counter_evidence_ids: Vec<String>,
    pub assumptions: Vec<String>,
    pub uncertainty: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisCitation {
    pub evidence_id: String,
    pub source_id: String,
    pub source_name: String,
    pub as_of_utc: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisAuditRecord {
    pub analyst_version: String,
    pub context_fingerprint: String,
    pub output_fingerprint: String,
    pub calculation_versions: Vec<String>,
    pub persisted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisValidation {
    pub valid: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalAnalysisReport {
    pub schema_version: u16,
    pub mode: String,
    pub intent: AnalysisIntent,
    pub selected_instrument: String,
    pub data_class: String,
    pub answer_text: String,
    pub claims: Vec<AnalysisClaim>,
    pub evidence: Vec<AnalysisEvidence>,
    pub citations: Vec<AnalysisCitation>,
    pub assumptions: Vec<String>,
    pub uncertainty: String,
    pub missing_data: Vec<String>,
    pub validation: AnalysisValidation,
    pub audit: AnalysisAuditRecord,
    pub disclaimer: String,
}

#[derive(Debug, Clone, Copy)]
struct DatasetFeatures {
    bar_count: f64,
    first_close: f64,
    last_close: f64,
    net_change: f64,
    period_return_pct: f64,
    maximum_drawdown_pct: f64,
    recent_high: f64,
    recent_low: f64,
    sma_20: Option<f64>,
    rsi_14: Option<f64>,
    atr_14: Option<f64>,
}

impl DatasetFeatures {
    fn extract(dataset: &Dataset) -> Result<Self, String> {
        let first = dataset
            .candles
            .first()
            .ok_or_else(|| "Local analysis requires a validated, non-empty dataset.".to_owned())?;
        let last = dataset
            .candles
            .last()
            .expect("non-empty dataset was checked");
        let closes = dataset
            .candles
            .iter()
            .map(|candle| Some(candle.close))
            .collect::<Vec<_>>();
        let sma_20 = sma(&closes, 20)
            .map_err(|error| format!("Could not calculate SMA-20: {error}"))?
            .last()
            .copied()
            .flatten();
        let rsi_14 = rsi(&closes, 14)
            .map_err(|error| format!("Could not calculate RSI-14: {error}"))?
            .last()
            .copied()
            .flatten();
        let atr_14 = atr(&dataset.candles, 14)
            .map_err(|error| format!("Could not calculate ATR-14: {error}"))?
            .last()
            .copied()
            .flatten();
        let period_return_pct = (last.close / first.close - 1.0) * 100.0;
        if !period_return_pct.is_finite() {
            return Err(
                "The selected data range is too large for a finite return calculation.".into(),
            );
        }

        let mut peak = first.close;
        let mut maximum_drawdown_pct: f64 = 0.0;
        for candle in &dataset.candles {
            peak = peak.max(candle.close);
            maximum_drawdown_pct = maximum_drawdown_pct.min((candle.close / peak - 1.0) * 100.0);
        }

        let recent = &dataset.candles[dataset.candles.len().saturating_sub(20)..];
        Ok(Self {
            bar_count: dataset.candles.len() as f64,
            first_close: first.close,
            last_close: last.close,
            net_change: last.close - first.close,
            period_return_pct,
            maximum_drawdown_pct,
            recent_high: recent
                .iter()
                .map(|candle| candle.high)
                .fold(f64::NEG_INFINITY, f64::max),
            recent_low: recent
                .iter()
                .map(|candle| candle.low)
                .fold(f64::INFINITY, f64::min),
            sma_20,
            rsi_14,
            atr_14,
        })
    }
}

pub fn analyze_local_dataset(
    dataset: &Dataset,
    question: &str,
) -> Result<LocalAnalysisReport, String> {
    let question = question.trim();
    if question.is_empty()
        || question.chars().count() > MAX_QUESTION_CHARS
        || question.chars().any(char::is_control)
    {
        return Err("Question must contain 1-500 characters without control characters.".into());
    }

    let features = DatasetFeatures::extract(dataset)?;
    let intent = intent::classify(question);
    let source_id = "user_provided_csv".to_owned();
    let source_name = dataset.metadata.source.clone();
    let start = dataset.quality.first_timestamp;
    let end = dataset.quality.last_timestamp;
    let mut evidence = Vec::new();

    push_evidence(
        &mut evidence,
        "bars",
        ClaimKind::Observation,
        "Validated candle count",
        features.bar_count,
        "bars",
        start,
        end,
        &source_id,
        &source_name,
    );
    push_evidence(
        &mut evidence,
        "first_close",
        ClaimKind::Observation,
        "First close",
        features.first_close,
        "price units; currency not provided",
        start,
        start,
        &source_id,
        &source_name,
    );
    push_evidence(
        &mut evidence,
        "last_close",
        ClaimKind::Observation,
        "Last close",
        features.last_close,
        "price units; currency not provided",
        end,
        end,
        &source_id,
        &source_name,
    );
    push_evidence(
        &mut evidence,
        "net_change",
        ClaimKind::Calculation,
        "Close-to-close net change over the supplied range",
        features.net_change,
        "price units; currency not provided",
        start,
        end,
        &source_id,
        &source_name,
    );
    push_evidence(
        &mut evidence,
        "period_return_pct",
        ClaimKind::Calculation,
        "Close-to-close return over the supplied range",
        features.period_return_pct,
        "percent; (last close / first close - 1) * 100",
        start,
        end,
        &source_id,
        &source_name,
    );
    push_evidence(
        &mut evidence,
        "maximum_drawdown_pct",
        ClaimKind::Calculation,
        "Maximum close-based drawdown in the supplied range",
        features.maximum_drawdown_pct,
        "percent; peak-to-subsequent-close decline",
        start,
        end,
        &source_id,
        &source_name,
    );
    push_evidence(
        &mut evidence,
        "recent_high",
        ClaimKind::Observation,
        "Highest high in the latest 20 bars or available range",
        features.recent_high,
        "price units; currency not provided",
        dataset.candles[dataset.candles.len().saturating_sub(20)].timestamp,
        end,
        &source_id,
        &source_name,
    );
    push_evidence(
        &mut evidence,
        "recent_low",
        ClaimKind::Observation,
        "Lowest low in the latest 20 bars or available range",
        features.recent_low,
        "price units; currency not provided",
        dataset.candles[dataset.candles.len().saturating_sub(20)].timestamp,
        end,
        &source_id,
        &source_name,
    );
    for (id, label, value, unit) in [
        (
            "sma_20",
            "Simple moving average, 20 bars",
            features.sma_20,
            "price units",
        ),
        (
            "rsi_14",
            "Cutler RSI, 14 bars",
            features.rsi_14,
            "index points (0-100)",
        ),
        (
            "atr_14",
            "Average true range, 14 bars",
            features.atr_14,
            "price units",
        ),
    ] {
        if let Some(value) = value {
            push_evidence(
                &mut evidence,
                id,
                ClaimKind::Calculation,
                label,
                value,
                unit,
                start,
                end,
                &source_id,
                &source_name,
            );
        }
    }

    let data_class = match dataset.metadata.data_class {
        DataClass::Synthetic => "SYNTHETIC / TEST",
        DataClass::Unknown => "UNKNOWN / UNVERIFIED",
    }
    .to_owned();
    let (answer_text, claims, missing_data) =
        draft_response(intent, dataset, &features, &evidence, &data_class);
    let citations = evidence
        .iter()
        .map(|item| AnalysisCitation {
            evidence_id: item.id.clone(),
            source_id: item.source_id.clone(),
            source_name: item.source_name.clone(),
            as_of_utc: item.as_of_utc,
        })
        .collect();
    let assumptions = vec![
        "Only the validated CSV selected for this session was analyzed.".into(),
        "CSV timestamps use the application's Asia/Kolkata interpretation.".into(),
        "The CSV source label is user-provided and has not been independently verified.".into(),
    ];
    let uncertainty = "This is descriptive analysis of the supplied historical rows. It does not establish future outcomes or explain causation.".to_owned();
    let context_fingerprint = fingerprint_dataset(dataset, question);
    let output_fingerprint = fingerprint_output(&answer_text, &claims);
    let mut report = LocalAnalysisReport {
        schema_version: 1,
        mode: "local_deterministic".into(),
        intent,
        selected_instrument: dataset.metadata.instrument.clone(),
        data_class,
        answer_text,
        claims,
        evidence,
        citations,
        assumptions,
        uncertainty,
        missing_data,
        validation: AnalysisValidation {
            valid: false,
            warnings: Vec::new(),
        },
        audit: AnalysisAuditRecord {
            analyst_version: ANALYST_VERSION.into(),
            context_fingerprint,
            output_fingerprint,
            calculation_versions: vec![
                "close-return-v1".into(),
                "close-drawdown-v1".into(),
                "sma-20".into(),
                "cutler-rsi-14".into(),
                "wilder-atr-14".into(),
            ],
            persisted: false,
        },
        disclaimer: "Research/education only. Not financial advice or a prediction.".into(),
    };
    validate_report(&mut report)?;
    Ok(report)
}

fn draft_response(
    intent: AnalysisIntent,
    dataset: &Dataset,
    features: &DatasetFeatures,
    evidence: &[AnalysisEvidence],
    data_class: &str,
) -> (String, Vec<AnalysisClaim>, Vec<String>) {
    let source = &dataset.metadata.source;
    let end = dataset.quality.last_timestamp;
    let (answer, claim_text, kind, ids, missing) = match intent {
        AnalysisIntent::SafetyBoundary => (
            "I cannot promise returns or provide a personalized buy/sell instruction. I can describe the validated CSV evidence instead.".to_owned(),
            None,
            ClaimKind::Limitation,
            Vec::new(),
            vec!["A suitable question about historical CSV data.".into()],
        ),
        AnalysisIntent::Causality => (
            format!(
                "The candle closes changed by {:.4} price units across the supplied range, but no sourced event or news evidence is available. These rows cannot establish why the price moved.",
                features.net_change
            ),
            Some(format!(
                "The close-to-close change across the supplied range was {:.4} price units.",
                features.net_change
            )),
            ClaimKind::Observation,
            vec!["first_close", "last_close", "net_change"],
            vec!["Sourced event, company, and news data.".into()],
        ),
        AnalysisIntent::Comparison => (
            "Only one validated dataset is active. A comparison requires a second independently validated instrument and a compatible time range.".into(),
            None,
            ClaimKind::Limitation,
            Vec::new(),
            vec!["A second instrument with a matching, validated time range.".into()],
        ),
        AnalysisIntent::CompanyResearch => (
            "Company fundamentals and sourced news are not connected to this local analyst. No company facts can be established from this candle CSV.".into(),
            None,
            ClaimKind::Limitation,
            Vec::new(),
            vec!["Provider-sourced company profile, financial statements, and news.".into()],
        ),
        AnalysisIntent::Forecast => (
            "This analyst does not generate a trained or validated forecast. The separate experimental baseline tool can repeat the last close, but that estimate is not a prediction of future performance.".into(),
            None,
            ClaimKind::Limitation,
            Vec::new(),
            vec!["A trained model and leakage-safe out-of-sample evaluation.".into()],
        ),
        AnalysisIntent::PriceChange => (
            format!(
                "For {} ({data_class}), the close changed by {:.4} price units ({:.4}%) across {} validated bars, from {:.4} to {:.4}. Source label: {source}; last supplied candle: {} Asia/Kolkata.",
                dataset.metadata.instrument,
                features.net_change,
                features.period_return_pct,
                dataset.candles.len(),
                features.first_close,
                features.last_close,
                crate::data::format_timestamp(end),
            ),
            Some(format!(
                "The close-to-close change over the supplied range was {:.4} price units ({:.4}%).",
                features.net_change, features.period_return_pct
            )),
            ClaimKind::Calculation,
            vec!["first_close", "last_close", "net_change", "period_return_pct"],
            Vec::new(),
        ),
        AnalysisIntent::Trend => {
            let comparison = features.sma_20.map(|average| {
                if features.last_close > average {
                    ("above", "The last close is above the 20-bar SMA; this describes the selected rows and does not predict continuation.")
                } else if features.last_close < average {
                    ("below", "The last close is below the 20-bar SMA; this describes the selected rows and does not predict continuation.")
                } else {
                    ("at", "The last close equals the 20-bar SMA; this describes the selected rows only.")
                }
            });
            match comparison {
                Some((position, text)) => (
                    format!("{text} The last close is {position} the 20-bar SMA; the selected range return was {:.4}%.", features.period_return_pct),
                    Some(text.into()),
                    ClaimKind::Interpretation,
                    vec!["last_close", "sma_20", "period_return_pct"],
                    Vec::new(),
                ),
                None => (
                    "There are fewer than 20 validated bars, so a 20-bar trend comparison is unavailable. No shorter lookback is substituted.".into(),
                    None,
                    ClaimKind::Limitation,
                    Vec::new(),
                    vec!["At least 20 validated bars for the configured SMA-20 comparison.".into()],
                ),
            }
        }
        AnalysisIntent::Risk => (
            format!(
                "The maximum close-based drawdown within the supplied range was {:.4}%. This is a historical path statistic, not a measure of future loss or complete risk.",
                features.maximum_drawdown_pct
            ),
            Some(format!(
                "The largest peak-to-subsequent-close decline in the supplied range was {:.4}%.",
                features.maximum_drawdown_pct
            )),
            ClaimKind::Calculation,
            vec!["maximum_drawdown_pct"],
            vec!["Provider-verified prices, corporate-action basis, and a market-session calendar.".into()],
        ),
        AnalysisIntent::Indicators => {
            let mut parts = Vec::new();
            let mut ids = vec!["last_close"];
            if let Some(value) = features.sma_20 {
                parts.push(format!("SMA-20 {:.4}", value));
                ids.push("sma_20");
            }
            if let Some(value) = features.rsi_14 {
                parts.push(format!("Cutler RSI-14 {:.2}", value));
                ids.push("rsi_14");
            }
            if let Some(value) = features.atr_14 {
                parts.push(format!("Wilder ATR-14 {:.4}", value));
                ids.push("atr_14");
            }
            let missing = if parts.len() < 3 {
                vec!["Longer history is needed for one or more indicator warm-up periods.".into()]
            } else {
                Vec::new()
            };
            (
                if parts.is_empty() {
                    "The available history is too short to produce the requested indicator values.".into()
                } else {
                    format!(
                        "Using Rust indicator calculations on the selected CSV: {}. These describe historical rows and are not standalone trading signals.",
                        parts.join("; ")
                    )
                },
                (!parts.is_empty()).then(|| format!("Calculated indicators from the validated CSV: {}.", parts.join("; "))),
                ClaimKind::Calculation,
                ids,
                missing,
            )
        }
        AnalysisIntent::DatasetSummary => (
            format!(
                "{} contains {} validated bars from {} to {} (Asia/Kolkata). Last close: {:.4} price units; source label: {source}; data class: {data_class}.",
                dataset.metadata.instrument,
                dataset.candles.len(),
                crate::data::format_timestamp(dataset.quality.first_timestamp),
                crate::data::format_timestamp(end),
                features.last_close,
            ),
            Some(format!(
                "The active dataset contains {} validated bars and its last close is {:.4} price units.",
                dataset.candles.len(),
                features.last_close
            )),
            ClaimKind::Observation,
            vec!["bars", "last_close"],
            Vec::new(),
        ),
    };

    let claims = claim_text
        .map(|text| {
            vec![AnalysisClaim {
                id: "claim-1".into(),
                text,
                kind,
                evidence_ids: ids
                    .into_iter()
                    .filter(|id| evidence.iter().any(|item| item.id == *id))
                    .map(str::to_owned)
                    .collect(),
                counter_evidence_ids: Vec::new(),
                assumptions: vec!["The supplied CSV is the only analyzed evidence.".into()],
                uncertainty: "Descriptive historical result; no future inference.".into(),
            }]
        })
        .unwrap_or_default();
    (answer, claims, missing)
}

#[allow(clippy::too_many_arguments)]
fn push_evidence(
    evidence: &mut Vec<AnalysisEvidence>,
    id: &str,
    kind: ClaimKind,
    label: &str,
    value: f64,
    unit: &str,
    period_start_utc: i64,
    period_end_utc: i64,
    source_id: &str,
    source_name: &str,
) {
    evidence.push(AnalysisEvidence {
        id: id.into(),
        kind,
        label: label.into(),
        value,
        unit: unit.into(),
        period_start_utc,
        period_end_utc,
        source_id: source_id.into(),
        source_name: source_name.into(),
        as_of_utc: period_end_utc,
    });
}

fn validate_report(report: &mut LocalAnalysisReport) -> Result<(), String> {
    if report.answer_text.trim().is_empty() || report.answer_text.chars().count() > 2_000 {
        return Err("Analyst response failed its output length validation.".into());
    }
    for claim in &report.claims {
        if claim.text.trim().is_empty()
            || claim.text.chars().count() > 1_000
            || claim.kind != ClaimKind::Limitation
                && (claim.evidence_ids.is_empty()
                    || claim
                        .evidence_ids
                        .iter()
                        .any(|id| !report.evidence.iter().any(|item| item.id == *id)))
        {
            return Err("Analyst response contains an unsupported or invalid claim.".into());
        }
    }
    if report.evidence.iter().any(|item| !item.value.is_finite()) {
        return Err("Analyst evidence contains a non-finite numeric value.".into());
    }
    report.validation.valid = true;
    Ok(())
}

fn fingerprint_dataset(dataset: &Dataset, question: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    update_fingerprint(&mut hash, ANALYST_VERSION.as_bytes());
    update_fingerprint(&mut hash, question.as_bytes());
    update_fingerprint(&mut hash, dataset.metadata.instrument.as_bytes());
    update_fingerprint(&mut hash, dataset.metadata.source.as_bytes());
    update_fingerprint(
        &mut hash,
        &dataset
            .metadata
            .interval_seconds
            .unwrap_or_default()
            .to_le_bytes(),
    );
    update_fingerprint(
        &mut hash,
        match dataset.metadata.data_class {
            DataClass::Synthetic => b"synthetic",
            DataClass::Unknown => b"unverified",
        },
    );
    for candle in &dataset.candles {
        update_fingerprint(&mut hash, &candle.timestamp.to_le_bytes());
        for value in [candle.open, candle.high, candle.low, candle.close] {
            update_fingerprint(&mut hash, &value.to_bits().to_le_bytes());
        }
        if let Some(volume) = candle.volume {
            update_fingerprint(&mut hash, &volume.to_bits().to_le_bytes());
        } else {
            update_fingerprint(&mut hash, b"volume-missing");
        }
    }
    format!("fnv1a64:{hash:016x}")
}

fn fingerprint_output(answer: &str, claims: &[AnalysisClaim]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    update_fingerprint(&mut hash, answer.as_bytes());
    for claim in claims {
        update_fingerprint(&mut hash, claim.text.as_bytes());
    }
    format!("fnv1a64:{hash:016x}")
}

fn update_fingerprint(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
}
