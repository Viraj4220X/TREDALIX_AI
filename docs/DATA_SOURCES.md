# Dataset sources and provenance

This file records what is currently known. Do not infer authenticity from a
filename, symbol, or metadata entered at the command line.

## Current workspace datasets

| File | Instrument | Classification | Provider / provenance | Interval |
|---|---|---|---|---|
| `data/NIFTY50.csv` | NIFTY 50 | Synthetic test data | User-provided sample values; provider and licence unknown | Unknown |
| `data/SBIN.csv` | SBIN | Synthetic test data | User-provided sample values; provider and licence unknown | Unknown |
| `data/sample.csv` | Test fixture | Synthetic | Created for parser/backtest testing; not exchange data | 5 minutes |

The NIFTY50 and SBIN files currently contain only two rows each. They are not
suitable for accuracy or profitability claims. No genuine historical dataset
has been provenance-checked for this project yet.

The CLI defaults to `UNKNOWN / UNVERIFIED`. `--synthetic` is an explicit user
declaration for test data; it does not verify historical data or source
authenticity. A supplied `--source` and interval are recorded as declarations,
not independently checked facts.

## Recording a future source

For each sourced dataset, record:

- Instrument and provider/source name.
- How it was exported or obtained and retrieval date.
- Date range, candle interval, timezone, and whether timestamps mean candle
  open or close.
- Whether volume is present and what its units mean.
- Licence/terms that apply to storage and use.
- Any known adjustments, missing periods, or limitations.

If any item is unknown, write `UNKNOWN`. Never put API keys, passwords, or
other secrets in this document.
