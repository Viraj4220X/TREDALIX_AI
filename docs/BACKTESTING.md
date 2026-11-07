# Offline backtest engine

Phase 2 uses one deterministic, long-only cash-equity simulator. It is a
historical simulation, not a brokerage connection or guarantee of future
results.

## Run

```text
cargo run -- backtest SBIN data/SBIN.csv --interval-seconds 60 --source "provider export" --instrument-type equity --capital 100000 --fee-bps 5 --slippage-bps 2 --max-quantity 100 --max-exposure 100000 --test-fraction 0.3
```

Add `--synthetic` when the input is test data. This is a user-supplied label,
not a provenance check. Use `--buy-and-hold` only for an instrument explicitly
declared with `--instrument-type equity`; index data must not be treated as an
executable asset. `inspect` remains available for data-quality checks.

If omitted, a report is written to a unique directory under `reports/`. To
choose a path, pass `--report-dir PATH`. The target directory must not already
exist; reports are never silently overwritten. A successful run writes:

- `summary.txt` — assumptions and human-readable portfolio metrics.
- `metrics.csv` — machine-readable strategy and benchmark metrics; undefined
  values are `NA`.
- `orders.csv` — decision/execution timestamps, side, quantity, fill, costs,
  status, and rejection reason.
- `trades.csv` — entry/exit, gross PnL, fees, estimated slippage, net PnL.
- `equity_curve.csv` — cash, marked position value, equity, peak, and
  drawdown after each candle.
- `effective_config.json` — strategy, settings, evaluation window, instrument,
  data class, source, and a non-cryptographic FNV-1a dataset fingerprint.

Fatal input/configuration/report errors return a nonzero process status.

## Version 1 simulation rules

- SMA-9/SMA-21/RSI-14 emits long-entry, long-exit, or hold actions. It cannot
  short, use margin, add to a position, or place derivative orders.
- A decision is made using data through candle `t` only. A market order is
  simulated at the next candle's open, with configured adverse percentage
  slippage. If no next candle exists, the order is rejected.
- When a declared interval is exceeded before the next candle, an order is
  rejected. Session meaning remains unknown because no NSE session/holiday
  calendar is configured.
- Position size is capped by maximum quantity, maximum exposure, and available
  cash including estimated entry fees. Only one open position is supported.
- `--fee-bps` applies a generic estimated fee to executed notional. It does not
  claim to model provider-specific brokerage, taxes, duties, or exchange fees.
- Slippage affects fill prices and is also itemized separately. Gross trade
  PnL is based on simulated fills before fees; net portfolio PnL includes all
  fees and marked open-position value.
- Open positions at the end are marked to the final close, not force-liquidated.
- The default `--test-fraction 0.3` evaluates the last 30% chronologically.
  Earlier data is available only as strategy warm-up. No parameters are fitted
  by this version, so this is a trailing holdout, not a full repeated
  walk-forward/calibration framework.
- The cash benchmark has zero return. Optional buy-and-hold uses the first
  executable next-candle open and the same fee/slippage assumptions; it is
  disabled for instruments not explicitly declared cash equities.

## Not modelled yet

This version does not model bid/ask spread, order-book depth, queue priority,
partial fills, taxes by date/product, market holidays/sessions, stop-loss or
take-profit triggers, daily loss limits, short selling, leverage, or live
orders. A bar-based simulation cannot reconstruct actual exchange fills.
Unknown session gaps are conservatively rejected for execution but are not
certified as missing candles.

The current workspace's NIFTY50 and SBIN files are synthetic two-row samples.
No real-market performance conclusion can be drawn from them. Genuine data
provenance and a reliable exchange calendar remain separate acceptance gates.
