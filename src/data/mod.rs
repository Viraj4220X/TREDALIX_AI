mod csv_reader;
mod model;
mod report;
mod session;
mod timestamp;
mod validation;

pub use csv_reader::{load_csv, parse_csv};
pub use model::{
    Candle, DataClass, Dataset, DatasetMetadata, InstrumentType, TimestampMeaning,
    TimestampTimezone,
};
pub use report::print_quality_report;
pub use session::{GapClassification, GapSummary, inspect_gaps};
pub use timestamp::{format_timestamp, parse_timestamp};
pub use validation::{DataQualitySummary, validate_dataset};

use std::path::Path;

pub fn load_dataset(path: impl AsRef<Path>, metadata: DatasetMetadata) -> Result<Dataset, String> {
    if metadata.interval_seconds == Some(0) {
        return Err("Dataset interval must be greater than zero seconds".to_owned());
    }
    if metadata
        .interval_seconds
        .is_some_and(|interval| interval > i64::MAX as u64)
    {
        return Err("Dataset interval exceeds the supported timestamp range".to_owned());
    }
    let candles = load_csv(path)?;
    let quality = validate_dataset(&candles, metadata.interval_seconds)?;
    Ok(Dataset {
        metadata,
        candles,
        quality,
    })
}
