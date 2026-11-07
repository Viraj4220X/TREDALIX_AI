mod momentum;
mod series;
mod volatility;

pub use momentum::{MacdSeries, macd};
pub use series::{IndicatorError, ema, rsi, sma};
pub use volatility::{BollingerBands, atr, bollinger_bands};
