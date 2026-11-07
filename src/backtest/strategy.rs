use crate::data::Candle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Hold,
    EnterLong,
    ExitLong,
}

pub trait Strategy {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn action(&self, history: &[Candle]) -> Action;

    fn try_action(&self, history: &[Candle]) -> Result<Action, String> {
        Ok(self.action(history))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SmaRsiStrategy;

impl Strategy for SmaRsiStrategy {
    fn name(&self) -> &'static str {
        "SMA9-SMA21-RSI14"
    }

    fn version(&self) -> &'static str {
        "1"
    }

    fn action(&self, history: &[Candle]) -> Action {
        self.try_action(history).unwrap_or(Action::Hold)
    }

    fn try_action(&self, history: &[Candle]) -> Result<Action, String> {
        match (sma(history, 9)?, sma(history, 21)?, rsi(history, 14)?) {
            (Some(fast), Some(slow), Some(momentum))
                if fast > slow && (50.0..70.0).contains(&momentum) =>
            {
                Ok(Action::EnterLong)
            }
            (Some(fast), Some(slow), Some(momentum))
                if fast < slow && (30.0..50.0).contains(&momentum) =>
            {
                Ok(Action::ExitLong)
            }
            _ => Ok(Action::Hold),
        }
    }
}

fn sma(history: &[Candle], period: usize) -> Result<Option<f64>, String> {
    if history.len() < period {
        return Ok(None);
    }
    let values = history[history.len() - period..]
        .iter()
        .map(|candle| Some(candle.close))
        .collect::<Vec<_>>();
    let output = crate::indicators::sma(&values, period)
        .map_err(|error| error.to_string())?
        .last()
        .copied()
        .ok_or_else(|| "SMA output alignment failure".to_owned())?;
    Ok(output)
}

fn rsi(history: &[Candle], period: usize) -> Result<Option<f64>, String> {
    if history.len() <= period {
        return Ok(None);
    }
    let values = history[history.len() - period - 1..]
        .iter()
        .map(|candle| Some(candle.close))
        .collect::<Vec<_>>();
    let output = crate::indicators::rsi(&values, period)
        .map_err(|error| error.to_string())?
        .last()
        .copied()
        .ok_or_else(|| "RSI output alignment failure".to_owned())?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::{Action, SmaRsiStrategy, Strategy};
    use crate::data::Candle;

    #[test]
    fn strategy_warmup_does_not_invent_a_signal() {
        let strategy = SmaRsiStrategy;
        assert_eq!(strategy.action(&[]), Action::Hold);
        let candle = Candle {
            timestamp: 1,
            open: 100.0,
            high: 100.0,
            low: 100.0,
            close: 100.0,
            volume: None,
        };
        assert_eq!(strategy.action(&[candle]), Action::Hold);
    }
}
