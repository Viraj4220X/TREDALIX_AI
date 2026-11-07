use super::model::Candle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GapClassification {
    SessionCalendarNotConfigured,
}

#[derive(Debug, Default)]
pub struct GapSummary {
    pub observed_gaps: usize,
    pub unobserved_intervals: u64,
    pub irregular_gaps: usize,
    pub classification: Option<GapClassification>,
}

pub fn inspect_gaps(candles: &[Candle], interval_seconds: Option<u64>) -> GapSummary {
    let Some(interval) = interval_seconds.filter(|interval| *interval > 0) else {
        return GapSummary::default();
    };
    let mut summary = GapSummary::default();
    for pair in candles.windows(2) {
        let elapsed = (pair[1].timestamp - pair[0].timestamp) as u64;
        if elapsed <= interval {
            continue;
        }
        summary.observed_gaps += 1;
        if elapsed.is_multiple_of(interval) {
            summary.unobserved_intervals += elapsed / interval - 1;
        } else {
            summary.irregular_gaps += 1;
        }
    }
    if summary.observed_gaps > 0 {
        summary.classification = Some(GapClassification::SessionCalendarNotConfigured);
    }
    summary
}

#[cfg(test)]
mod tests {
    use super::{GapClassification, inspect_gaps};
    use crate::data::Candle;

    fn candle(timestamp: i64) -> Candle {
        Candle {
            timestamp,
            open: 1.0,
            high: 1.0,
            low: 1.0,
            close: 1.0,
            volume: None,
        }
    }

    #[test]
    fn gaps_are_reported_without_claiming_market_session_meaning() {
        let gaps = inspect_gaps(&[candle(0), candle(300)], Some(60));
        assert_eq!(gaps.observed_gaps, 1);
        assert_eq!(gaps.unobserved_intervals, 4);
        assert_eq!(
            gaps.classification,
            Some(GapClassification::SessionCalendarNotConfigured)
        );
    }

    #[test]
    fn unknown_interval_does_not_guess_missing_candles() {
        assert_eq!(
            inspect_gaps(&[candle(0), candle(300)], None).observed_gaps,
            0
        );
    }
}
