# Offline forecast baseline and walk-forward evaluation

## Naive random-walk forecast

`src/forecasting.rs` exposes `naive_random_walk`. It validates the provided
candles, requires at least two bars, and repeats the latest close for a
bounded horizon of 1–20 bars. The caller supplies the exchange-qualified
instrument, currency, source provenance, and generation timestamp; none are
inferred from a ticker or invented.

The forecast uses the typed `ForecastSeries` contract. Forecast timestamps
start after the last candle's close boundary, estimates remain separate from
observations, and the result contains no confidence interval or scenario
probability. Its display label is
“MODEL ESTIMATE — NOT OBSERVED MARKET DATA”. The Web feature is experimental
and can be disabled in Feature Manager.

This is a persistence baseline, not a trained AI model or investment signal.
It does not model trading sessions, corporate actions, costs, or forecast
uncertainty. Timestamp extrapolation by bar interval can cross a session
boundary incorrectly; verify timeframe, exchange and currency before use.
Uploaded CSVs stay local. Synthetic input remains marked synthetic.

## Walk-forward baseline comparison

`src/evaluation.rs` evaluates chronological, non-overlapping test folds with
an expanding training window. Each test bar is predicted one bar ahead using
only earlier closes. It compares:

- Random walk: the previous close.
- Historical drift: previous close multiplied by one plus the mean of the
  prior simple close-to-close returns.

The report includes fold boundaries, training/test counts, MAE, RMSE, and
directional accuracy. There is no shuffled split, learned transformation,
future target in a training window, or strategy-performance claim. Insufficient
history and non-finite/overflowed calculations return explicit errors.

This is not repeated model calibration, a trading backtest, or evidence of
future profitability. Current evaluation is one-step only; it does not
estimate uncertainty, regime coverage, fees, market-session calendars, or
corporate-action adjustments.

The Web/WASM operation is guarded by the `naive_forecast` and `walk_forward`
feature settings at both the UI and Rust adapter boundaries. Results are
calculated from the validated local CSV and are not saved to run history.
