use std::path::Path;

use crate::backtest::{BacktestConfig, BacktestResult, SmaRsiStrategy, run_backtest};
use crate::data::{DatasetMetadata, load_dataset};

pub fn backtest(
    path: impl AsRef<Path>,
    metadata: DatasetMetadata,
    config: &BacktestConfig,
) -> Result<BacktestResult, String> {
    let dataset = load_dataset(path, metadata)?;
    run_backtest(&dataset, &SmaRsiStrategy, config)
}
