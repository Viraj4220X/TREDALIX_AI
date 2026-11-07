use serde::{Deserialize, Serialize};

use crate::data::{Candle, validate_dataset};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalkForwardConfig {
    pub folds: usize,
    pub minimum_training_bars: usize,
    pub test_bars_per_fold: usize,
}

impl WalkForwardConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.folds == 0 || self.test_bars_per_fold == 0 || self.minimum_training_bars < 2 {
            return Err(
                "Walk-forward requires positive folds/test bars and at least two training bars"
                    .into(),
            );
        }
        if self.folds > 100
            || self.minimum_training_bars > 100_000
            || self.test_bars_per_fold > 100_000
        {
            return Err("Walk-forward settings exceed the supported compute bounds".into());
        }
        if self.folds.checked_mul(self.test_bars_per_fold).is_none() {
            return Err("Walk-forward test window size overflows".into());
        }
        if self.folds * self.test_bars_per_fold > 100_000 {
            return Err("Walk-forward evaluation is limited to 100,000 test bars".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ForecastMetrics {
    pub samples: usize,
    pub mae: f64,
    pub rmse: f64,
    pub directional_accuracy: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalkForwardFold {
    pub fold: usize,
    pub training_bars_at_start: usize,
    pub training_end_timestamp: i64,
    pub first_test_timestamp: i64,
    pub last_test_timestamp: i64,
    pub random_walk: ForecastMetrics,
    pub historical_drift: ForecastMetrics,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalkForwardReport {
    pub model_id: String,
    pub model_version: String,
    pub folds: Vec<WalkForwardFold>,
    pub random_walk: ForecastMetrics,
    pub historical_drift: ForecastMetrics,
    pub horizon_evaluations: Vec<HorizonEvaluation>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HorizonEvaluation {
    pub horizon_bars: usize,
    pub folds: Vec<HorizonFold>,
    pub random_walk: ForecastMetrics,
    pub historical_drift: ForecastMetrics,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HorizonFold {
    pub fold: usize,
    pub training_bars_at_start: usize,
    pub first_target_timestamp: i64,
    pub last_target_timestamp: i64,
    pub random_walk: ForecastMetrics,
    pub historical_drift: ForecastMetrics,
}

#[derive(Default)]
struct MetricAccumulator {
    absolute_error: f64,
    squared_error: f64,
    correct_direction: usize,
    samples: usize,
}

impl MetricAccumulator {
    fn add(&mut self, actual: f64, predicted: f64, previous: f64) -> Result<(), String> {
        let error = actual - predicted;
        self.absolute_error += error.abs();
        self.squared_error += error * error;
        if !self.absolute_error.is_finite() || !self.squared_error.is_finite() {
            return Err("Walk-forward metric overflowed; price scale is unsupported".into());
        }
        let actual_direction = (actual - previous).total_cmp(&0.0);
        let predicted_direction = (predicted - previous).total_cmp(&0.0);
        if actual_direction == predicted_direction {
            self.correct_direction += 1;
        }
        self.samples += 1;
        Ok(())
    }

    fn finish(&self) -> ForecastMetrics {
        ForecastMetrics {
            samples: self.samples,
            mae: self.absolute_error / self.samples as f64,
            rmse: (self.squared_error / self.samples as f64).sqrt(),
            directional_accuracy: self.correct_direction as f64 / self.samples as f64,
        }
    }
}

pub fn evaluate_walk_forward(
    candles: &[Candle],
    interval_seconds: u64,
    config: WalkForwardConfig,
) -> Result<WalkForwardReport, String> {
    evaluate_walk_forward_with_horizons(candles, interval_seconds, config, &[1])
}

pub fn evaluate_walk_forward_with_horizons(
    candles: &[Candle],
    interval_seconds: u64,
    config: WalkForwardConfig,
    horizons_bars: &[usize],
) -> Result<WalkForwardReport, String> {
    config.validate()?;
    if horizons_bars.is_empty()
        || horizons_bars.len() > 10
        || horizons_bars
            .iter()
            .any(|horizon| *horizon == 0 || *horizon > 1_000)
    {
        return Err(
            "Walk-forward horizons must contain 1-10 values between 1 and 1,000 bars".into(),
        );
    }
    let mut unique_horizons = horizons_bars.to_vec();
    unique_horizons.sort_unstable();
    unique_horizons.dedup();
    if unique_horizons.len() != horizons_bars.len() {
        return Err("Walk-forward horizons must be unique".into());
    }
    if config
        .folds
        .checked_mul(config.test_bars_per_fold)
        .and_then(|count| count.checked_mul(horizons_bars.len()))
        .is_none_or(|count| count > 100_000)
    {
        return Err("Walk-forward evaluation is limited to 100,000 horizon test bars".into());
    }
    if interval_seconds == 0 || interval_seconds > i64::MAX as u64 {
        return Err("Walk-forward interval is outside the supported timestamp range".into());
    }
    validate_dataset(candles, Some(interval_seconds))?;
    let test_bars = config
        .folds
        .checked_mul(config.test_bars_per_fold)
        .ok_or_else(|| "Walk-forward test window size overflows".to_owned())?;
    let required_bars = config
        .minimum_training_bars
        .checked_add(*horizons_bars.iter().max().unwrap() - 1)
        .and_then(|minimum| minimum.checked_add(test_bars))
        .ok_or_else(|| "Walk-forward minimum history requirement overflows".to_owned())?;
    let first_test = candles
        .len()
        .checked_sub(test_bars)
        .filter(|start| {
            *start >= config.minimum_training_bars + *horizons_bars.iter().max().unwrap() - 1
        })
        .ok_or_else(|| {
            format!(
                "Insufficient history: need at least {} bars for the requested folds",
                required_bars
            )
        })?;

    let mut aggregate_random_walk = MetricAccumulator::default();
    let mut aggregate_drift = MetricAccumulator::default();
    let mut folds = Vec::with_capacity(config.folds);
    let mut cumulative_returns = vec![0.0; candles.len()];
    for index in 1..candles.len() {
        let one_bar_return = candles[index].close / candles[index - 1].close - 1.0;
        cumulative_returns[index] = cumulative_returns[index - 1] + one_bar_return;
        if !cumulative_returns[index].is_finite() {
            return Err("Historical return accumulation overflowed".into());
        }
    }

    for fold_index in 0..config.folds {
        let fold_start = first_test + fold_index * config.test_bars_per_fold;
        let fold_end = fold_start + config.test_bars_per_fold;
        let mut random_walk = MetricAccumulator::default();
        let mut historical_drift = MetricAccumulator::default();
        for target_index in fold_start..fold_end {
            let previous = candles[target_index - 1].close;
            let actual = candles[target_index].close;
            let mean_return = cumulative_returns[target_index - 1] / (target_index - 1) as f64;
            let drift_prediction = previous * (1.0 + mean_return);
            if !drift_prediction.is_finite() || drift_prediction <= 0.0 {
                return Err("Historical-drift prediction is outside the supported range".into());
            }
            random_walk.add(actual, previous, previous)?;
            historical_drift.add(actual, drift_prediction, previous)?;
            aggregate_random_walk.add(actual, previous, previous)?;
            aggregate_drift.add(actual, drift_prediction, previous)?;
        }
        folds.push(WalkForwardFold {
            fold: fold_index + 1,
            training_bars_at_start: fold_start,
            training_end_timestamp: candles[fold_start - 1].timestamp,
            first_test_timestamp: candles[fold_start].timestamp,
            last_test_timestamp: candles[fold_end - 1].timestamp,
            random_walk: random_walk.finish(),
            historical_drift: historical_drift.finish(),
        });
    }

    let mut horizon_evaluations = Vec::with_capacity(horizons_bars.len());
    for horizon in horizons_bars {
        let mut aggregate_random_walk = MetricAccumulator::default();
        let mut aggregate_drift = MetricAccumulator::default();
        let mut horizon_folds = Vec::with_capacity(config.folds);
        for fold_index in 0..config.folds {
            let fold_start = first_test + fold_index * config.test_bars_per_fold;
            let fold_end = fold_start + config.test_bars_per_fold;
            let mut random_walk = MetricAccumulator::default();
            let mut historical_drift = MetricAccumulator::default();
            for target_index in fold_start..fold_end {
                let origin_index = target_index - horizon;
                let origin_close = candles[origin_index].close;
                let actual = candles[target_index].close;
                let mean_return = cumulative_returns[origin_index] / origin_index as f64;
                let drift_prediction = origin_close * (1.0 + mean_return).powf(*horizon as f64);
                if !drift_prediction.is_finite() || drift_prediction <= 0.0 {
                    return Err("Historical-drift prediction is outside the supported range".into());
                }
                random_walk.add(actual, origin_close, origin_close)?;
                historical_drift.add(actual, drift_prediction, origin_close)?;
                aggregate_random_walk.add(actual, origin_close, origin_close)?;
                aggregate_drift.add(actual, drift_prediction, origin_close)?;
            }
            horizon_folds.push(HorizonFold {
                fold: fold_index + 1,
                training_bars_at_start: fold_start - horizon + 1,
                first_target_timestamp: candles[fold_start].timestamp,
                last_target_timestamp: candles[fold_end - 1].timestamp,
                random_walk: random_walk.finish(),
                historical_drift: historical_drift.finish(),
            });
        }
        horizon_evaluations.push(HorizonEvaluation {
            horizon_bars: *horizon,
            folds: horizon_folds,
            random_walk: aggregate_random_walk.finish(),
            historical_drift: aggregate_drift.finish(),
        });
    }

    Ok(WalkForwardReport {
        model_id: "one_step_baseline_comparison".into(),
        model_version: "1".into(),
        folds,
        random_walk: aggregate_random_walk.finish(),
        historical_drift: aggregate_drift.finish(),
        horizon_evaluations,
        warnings: vec![
            "Expanding-window evaluation uses only bars available at each forecast origin; target bars are never used before prediction.".into(),
            "Random walk and historical drift are evaluated at each configured bar horizon.".into(),
            "This is a statistical evaluation, not a strategy simulation or profitability claim.".into(),
            "No uncertainty interval is estimated; session calendars and corporate actions are not modeled.".into(),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::{WalkForwardConfig, evaluate_walk_forward, evaluate_walk_forward_with_horizons};
    use crate::data::Candle;

    fn candles(closes: &[f64]) -> Vec<Candle> {
        closes
            .iter()
            .enumerate()
            .map(|(index, close)| Candle {
                timestamp: index as i64 * 60,
                open: *close,
                high: *close,
                low: *close,
                close: *close,
                volume: None,
            })
            .collect()
    }

    fn config() -> WalkForwardConfig {
        WalkForwardConfig {
            folds: 2,
            minimum_training_bars: 2,
            test_bars_per_fold: 2,
        }
    }

    #[test]
    fn uses_expanding_chronological_folds_and_reports_hand_calculated_metrics() {
        let report = evaluate_walk_forward(
            &candles(&[10.0, 11.0, 12.0, 13.0, 14.0, 15.0]),
            60,
            config(),
        )
        .unwrap();
        assert_eq!(report.folds.len(), 2);
        assert_eq!(report.folds[0].training_bars_at_start, 2);
        assert_eq!(report.folds[0].first_test_timestamp, 120);
        assert_eq!(report.folds[1].training_bars_at_start, 4);
        assert_eq!(report.random_walk.samples, 4);
        assert_eq!(report.random_walk.mae, 1.0);
        assert_eq!(report.random_walk.rmse, 1.0);
        assert_eq!(report.random_walk.directional_accuracy, 0.0);
    }

    #[test]
    fn rejects_insufficient_history_and_invalid_fold_configuration() {
        assert!(evaluate_walk_forward(&candles(&[10.0, 11.0, 12.0]), 60, config()).is_err());
        assert!(
            evaluate_walk_forward(
                &candles(&[10.0; 8]),
                60,
                WalkForwardConfig {
                    folds: 0,
                    ..config()
                }
            )
            .is_err()
        );
    }

    #[test]
    fn changing_future_test_bars_does_not_change_earlier_fold_metrics() {
        let original = candles(&[10.0, 11.0, 12.0, 13.0, 14.0, 15.0]);
        let before = evaluate_walk_forward(&original, 60, config()).unwrap();
        let mut changed = original;
        changed[5].close = 50.0;
        changed[5].high = 50.0;
        let after = evaluate_walk_forward(&changed, 60, config()).unwrap();
        assert_eq!(before.folds[0], after.folds[0]);
    }

    #[test]
    fn evaluates_multiple_horizons_without_using_future_bars() {
        let original = candles(&[10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0]);
        let before = evaluate_walk_forward_with_horizons(&original, 60, config(), &[1, 2]).unwrap();
        assert_eq!(
            before
                .horizon_evaluations
                .iter()
                .map(|evaluation| evaluation.horizon_bars)
                .collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(before.horizon_evaluations[1].random_walk.samples, 4);
        let mut changed = original;
        changed[7].close = 50.0;
        changed[7].high = 50.0;
        let after = evaluate_walk_forward_with_horizons(&changed, 60, config(), &[1, 2]).unwrap();
        assert_eq!(
            before.horizon_evaluations[1].folds[0],
            after.horizon_evaluations[1].folds[0]
        );
    }

    #[test]
    fn historical_drift_compounds_per_bar_for_longer_horizons() {
        let prices = [
            100.0, 110.0, 121.0, 133.1, 146.41, 161.051, 177.1561, 194.87171,
        ];
        let report =
            evaluate_walk_forward_with_horizons(&candles(&prices), 60, config(), &[2]).unwrap();
        let drift = report.horizon_evaluations[0].historical_drift;
        assert!(drift.mae < 1e-9);
        assert!(report.horizon_evaluations[0].random_walk.mae > 1.0);
    }
}
