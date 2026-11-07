use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use tredalix_ai::data::{DataClass, DatasetMetadata, load_dataset, parse_csv};

static TEMP_FILE_ID: AtomicUsize = AtomicUsize::new(0);

const VALID_CSV: &str = "\
  CLOSE , timestamp , low, high, open, volume, note
101,\"2026-10-20 09:15\",99,102,100,1500,\"comma, inside field\"
102,2026-10-20 09:16,100,103,101,,\"volume unavailable\"
";

#[test]
fn parses_reordered_headers_and_optional_volume() {
    let candles = parse_csv(VALID_CSV).unwrap();
    assert_eq!(candles.len(), 2);
    assert_eq!(candles[0].close, 101.0);
    assert_eq!(candles[0].volume, Some(1500.0));
    assert_eq!(candles[1].volume, None);
    assert!(candles[1].timestamp > candles[0].timestamp);
}

#[test]
fn rejects_missing_or_ambiguous_required_columns() {
    assert!(
        parse_csv("timestamp,close\n2026-10-20 09:15,101\n")
            .unwrap_err()
            .contains("Required CSV column missing: open")
    );
    assert!(
        parse_csv("timestamp,open,high,low,close, close\n2026-10-20 09:15,100,102,99,101,101\n")
            .unwrap_err()
            .contains("Duplicate CSV header: close")
    );
}

#[test]
fn rejects_empty_data_rows_and_malformed_values() {
    assert!(parse_csv("").unwrap_err().contains("empty"));
    assert!(
        parse_csv("timestamp,open,high,low,close\n")
            .unwrap_err()
            .contains("no data rows")
    );
    assert!(
        parse_csv("timestamp,open,high,low,close\n2026-10-20 09:15,100,102,99,NaN\n")
            .unwrap_err()
            .contains("close must be finite")
    );
    assert!(
        parse_csv("timestamp,open,high,low,close\n2026-10-20 09:15,100,99,98,101\n")
            .unwrap_err()
            .contains("OHLC relationship")
    );
}

#[test]
fn rejects_duplicate_and_out_of_order_timestamps() {
    let duplicate = VALID_CSV.replace("2026-10-20 09:16", "2026-10-20 09:15");
    assert!(
        parse_csv(&duplicate)
            .unwrap_err()
            .contains("duplicate timestamp")
    );
    let descending = VALID_CSV.replace("2026-10-20 09:16", "2026-10-20 09:14");
    assert!(
        parse_csv(&descending)
            .unwrap_err()
            .contains("not ascending")
    );
}

#[test]
fn metadata_defaults_to_unverified_and_loader_does_not_require_strategy_history() {
    let metadata = DatasetMetadata::unverified("TEST");
    assert_eq!(metadata.data_class, DataClass::Unknown);
    assert_eq!(metadata.source, "UNKNOWN");
    let path = std::env::temp_dir().join(format!(
        "tredalix-data-test-{}-{}.csv",
        std::process::id(),
        TEMP_FILE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, VALID_CSV).unwrap();
    let dataset = load_dataset(&path, metadata).unwrap();
    fs::remove_file(&path).unwrap();
    assert_eq!(dataset.candles.len(), 2);
    assert_eq!(dataset.metadata.source, "UNKNOWN");
}

#[test]
fn rejects_zero_interval_before_opening_the_csv() {
    let mut metadata = DatasetMetadata::unverified("TEST");
    metadata.interval_seconds = Some(0);
    assert!(
        load_dataset("missing-file.csv", metadata)
            .unwrap_err()
            .contains("greater than zero")
    );
}
