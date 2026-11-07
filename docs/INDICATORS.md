# Rust indicator library

Indicators live in `src/indicators/` and are deterministic, causal
calculations over input values. Each output vector is aligned to its input
index. `None` means warm-up or a missing observation; no future candle is used.
Inputs are expected to use one consistent price adjustment mode and interval.

## Implemented formulas

- **SMA(period):** arithmetic mean of the current trailing `period`
  observations. Emits `None` until a complete contiguous window exists.
- **EMA(period):** first value is seeded by the SMA of the first complete
  window; subsequent values use `alpha = 2 / (period + 1)`. A missing value
  resets the seed and warm-up.
- **RSI(period):** Cutler RSI, using simple trailing averages of gains and
  losses over `period` changes. This is deliberately identified as Cutler
  RSI, not Wilder-smoothed RSI. A flat window returns 50; only losses returns
  0; only gains returns 100. Missing data resets the change window.
- **MACD(fast, slow, signal):** difference of fast and slow EMAs, signal EMA
  of that line, and histogram difference. Requires `fast < slow`.
- **Bollinger bands:** trailing SMA with population standard deviation and a
  caller-supplied nonnegative multiplier.
- **ATR(period):** true range (high-low, or the maximum of high-low and gaps
  from prior close), seeded with the first-period average and then Wilder
  smoothing.

Invalid periods, non-finite inputs/results, invalid candles, and incompatible
parameters return errors. No silent numeric fallback is intended. CPU
complexity is proportional to the input length times period for the windowed
standard-deviation calculation; large-input performance has not yet been
benchmarked.

VWAP is not implemented because the project does not yet have verified
session boundaries and volume semantics. Support/resistance is also not
implemented. These functions produce numeric series only; chart rendering,
exchange calendars, provider provenance, and a prediction overlay are
separate contracts and are not implied by indicator availability.

The existing SMA-9/SMA-21/Cutler-RSI-14 strategy now calls the shared Rust
SMA/RSI functions. Existing backtest regression tests verify its accounting
and deterministic outputs.
