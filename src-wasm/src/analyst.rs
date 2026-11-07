use serde::Deserialize;
use serde_json::to_string;
use wasm_bindgen::prelude::*;

const MAX_CSV_BYTES: usize = 10 * 1024 * 1024;

#[derive(Debug, Deserialize)]
struct DatasetOptions {
    instrument: String,
    instrument_type: String,
    source: String,
    interval_seconds: u64,
    synthetic: bool,
}

#[derive(Debug, Deserialize)]
struct AnalystFeatures {
    csv_import: bool,
    data_validation: bool,
    local_analyst: bool,
}

fn check_csv_size(csv: &str) -> Result<(), String> {
    if csv.len() > MAX_CSV_BYTES {
        return Err("CSV exceeds the 10 MiB in-memory analysis limit".into());
    }
    Ok(())
}

pub fn analyze_csv_json(
    csv: &str,
    dataset_options_json: &str,
    question: &str,
    feature_flags_json: &str,
) -> Result<String, String> {
    check_csv_size(csv)?;
    let features: AnalystFeatures = serde_json::from_str(feature_flags_json)
        .map_err(|error| format!("Invalid analyst feature flags: {error}"))?;
    if !features.csv_import {
        return Err("CSV import is disabled in Feature Manager".into());
    }
    if !features.data_validation {
        return Err("Data validation is disabled in Feature Manager".into());
    }
    if !features.local_analyst {
        return Err("Offline analyst is disabled in Feature Manager".into());
    }
    let options: DatasetOptions = serde_json::from_str(dataset_options_json)
        .map_err(|error| format!("Invalid dataset options: {error}"))?;
    let dataset = parse_dataset(csv, &options)?;
    let report = tredalix_ai::analyst::analyze_local_dataset(&dataset, question)?;
    to_string(&report).map_err(|error| format!("Cannot encode analyst response: {error}"))
}

fn parse_dataset(
    csv: &str,
    options: &DatasetOptions,
) -> Result<tredalix_ai::data::Dataset, String> {
    if options.instrument.trim().is_empty() {
        return Err("Instrument identifier cannot be empty".into());
    }
    if options.source.trim().is_empty() {
        return Err("Data source description cannot be empty".into());
    }
    if options.interval_seconds == 0 || options.interval_seconds > i64::MAX as u64 {
        return Err("Interval must be in the supported positive timestamp range".into());
    }
    let instrument_type = match options.instrument_type.as_str() {
        "equity" => tredalix_ai::data::InstrumentType::CashEquity,
        "index" => tredalix_ai::data::InstrumentType::Index,
        "unknown" => tredalix_ai::data::InstrumentType::Unknown,
        _ => return Err("Instrument type must be equity, index, or unknown".into()),
    };
    let candles = tredalix_ai::data::parse_csv(csv)?;
    let quality = tredalix_ai::data::validate_dataset(&candles, Some(options.interval_seconds))?;
    Ok(tredalix_ai::data::Dataset {
        metadata: tredalix_ai::data::DatasetMetadata {
            instrument: options.instrument.trim().to_owned(),
            instrument_type,
            source: options.source.trim().to_owned(),
            interval_seconds: Some(options.interval_seconds),
            data_class: if options.synthetic {
                tredalix_ai::data::DataClass::Synthetic
            } else {
                tredalix_ai::data::DataClass::Unknown
            },
            timestamp_timezone: tredalix_ai::data::TimestampTimezone::AsiaKolkata,
            timestamp_meaning: tredalix_ai::data::TimestampMeaning::CandleOpen,
        },
        candles,
        quality,
    })
}

#[wasm_bindgen]
pub fn run_local_analysis(
    csv: &str,
    dataset_options_json: &str,
    question: &str,
    feature_flags_json: &str,
) -> Result<String, JsError> {
    analyze_csv_json(csv, dataset_options_json, question, feature_flags_json)
        .map_err(|error| JsError::new(&error))
}

#[cfg(test)]
mod tests {
    use super::analyze_csv_json;

    const CSV: &str = "timestamp,open,high,low,close,volume\n\
        2026-10-20 09:15,100,102,99,101,1000\n\
        2026-10-20 09:16,101,103,100,102,1000\n";
    const DATASET: &str = r#"{"instrument":"TEST","instrument_type":"unknown","source":"SYNTHETIC / TEST","interval_seconds":60,"synthetic":true}"#;
    const FLAGS: &str = r#"{"csv_import":true,"data_validation":true,"local_analyst":true}"#;

    #[test]
    fn wasm_boundary_returns_a_valid_local_report_from_validated_csv() {
        let value: serde_json::Value = serde_json::from_str(
            &analyze_csv_json(CSV, DATASET, "Summarize this data", FLAGS).unwrap(),
        )
        .unwrap();
        assert_eq!(value["mode"], "local_deterministic");
        assert_eq!(value["data_class"], "SYNTHETIC / TEST");
        assert_eq!(value["validation"]["valid"], true);
        assert_eq!(value["evidence"][0]["value"], 2.0);
    }

    #[test]
    fn wasm_boundary_enforces_feature_gates_and_rejects_invalid_data() {
        let disabled = r#"{"csv_import":true,"data_validation":true,"local_analyst":false}"#;
        assert!(analyze_csv_json(CSV, DATASET, "Summarize", disabled).is_err());
        assert!(
            analyze_csv_json(
                "timestamp,close\nbad,not-a-price",
                DATASET,
                "Summarize",
                FLAGS
            )
            .is_err()
        );
    }
}
