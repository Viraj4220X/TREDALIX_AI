use crate::data::{DataClass, Dataset, format_timestamp};
use crate::horizons::HORIZONS;

pub fn print_quality_report(dataset: &Dataset) {
    let metadata = &dataset.metadata;
    let quality = &dataset.quality;
    println!("\n===== DATA QUALITY =====");
    println!("Instrument: {}", metadata.instrument);
    println!("Instrument type: {}", metadata.instrument_type.label());
    println!("Classification: {}", metadata.data_class.label());
    println!("Source: {}", metadata.source);
    println!(
        "Interval: {}",
        metadata
            .interval_seconds
            .map_or_else(|| "UNKNOWN".to_owned(), |seconds| format!("{seconds}s"))
    );
    println!(
        "Timestamp timezone: {}",
        metadata.timestamp_timezone.label()
    );
    println!("Timestamp meaning: candle open");
    println!("Rows: {}", quality.row_count);
    println!(
        "Timestamp range ({}): {} to {}",
        metadata.timestamp_timezone.label(),
        format_timestamp(quality.first_timestamp),
        format_timestamp(quality.last_timestamp)
    );

    if let Some(interval) = metadata.interval_seconds {
        for horizon in HORIZONS {
            let support = if interval <= horizon.seconds {
                "resolution may be adequate; exact target coverage not established"
            } else {
                "unsupported at this interval"
            };
            println!("Horizon {}: {support}", horizon.name);
        }
    } else {
        println!("Horizon compatibility: UNKNOWN (dataset interval not declared)");
    }

    if quality.gaps.observed_gaps > 0 {
        println!(
            "Timestamp gaps: {} observed; {} unobserved intervals; {} irregular",
            quality.gaps.observed_gaps,
            quality.gaps.unobserved_intervals,
            quality.gaps.irregular_gaps
        );
        println!("Session classification: UNKNOWN (exchange calendar not configured)");
    } else if metadata.interval_seconds.is_some() {
        println!("Timestamp gaps: none detected at declared interval");
        println!("Session calendar: not configured; holidays/session boundaries not verified");
    } else {
        println!("Timestamp gaps: not assessed (interval unknown)");
    }

    if metadata.data_class != DataClass::Synthetic || metadata.source == "UNKNOWN" {
        println!("Provenance: unverified; filename/metadata do not authenticate source data");
    }
    if quality.row_count < 22 {
        println!("Warning: fewer than 22 candles; existing strategy cannot produce signals");
    }
    println!("Data-quality status: PASS WITH LIMITATIONS");
}
