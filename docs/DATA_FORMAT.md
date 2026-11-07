# Market data CSV format

The offline data loader accepts UTF-8 CSV with one header row and one candle per
subsequent line. Quoted commas and escaped double quotes (`""`) are supported.
Quoted fields cannot span multiple physical lines.

Required, case-insensitive columns:

```text
timestamp,open,high,low,close
```

`volume` is optional. An empty volume is represented as unavailable, not as
zero. A numeric zero remains a real reported zero. Other extra columns are
allowed, but every row must have the same number of fields as the header.

Rules:

- Timestamps use `YYYY-MM-DD HH:MM` or `YYYY-MM-DD HH:MM:SS`.
- Timestamps are interpreted as India exchange-local time (`Asia/Kolkata`,
  UTC+05:30), independent of the computer's timezone.
- A timestamp represents the **candle open time**.
- Prices must be finite and greater than zero.
- `high` must be at least open, close, and low; `low` must not exceed open
  or close.
- Supplied volume must be finite and nonnegative.
- Timestamps must be strictly ascending and unique.
- Invalid rows are rejected with a row-numbered error; they are never silently
  skipped.
- Interval and provenance are not inferred from a filename. Declare them in
  the CLI when known. The system does not verify a provider's authenticity.
- Without an interval declaration, gap counts and horizon resolution remain
  unknown. A declared interval is user-provided metadata, not independently
  verified evidence.
- No NSE session or holiday calendar is configured. Any detected gap remains
  unclassified rather than being assumed to be a missing candle or a market
  closure.

Inspect a file without running the strategy:

```text
cargo run -- inspect TEST data/sample.csv --interval-seconds 300 --source "local test fixture" --synthetic
```

Run a backtest with the same metadata options:

```text
cargo run -- backtest TEST data/sample.csv --interval-seconds 300 --source "local test fixture" --synthetic
```

Example below is synthetic and must not be used as market-performance evidence:

```csv
timestamp,open,high,low,close,volume
2026-10-20 09:15:00,100,102,99,101,1500
2026-10-20 09:16:00,101,103,100,102,
```

The current loader holds decoded candles in memory. For exceptionally large
files, resource use should be measured before production use.
