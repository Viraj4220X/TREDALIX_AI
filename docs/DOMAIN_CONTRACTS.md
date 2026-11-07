# Shared Rust domain contracts

The `src/domain/` module contains platform-neutral request/data contracts.
Provider adapters are not configured. Baseline forecast and evaluation
services live separately in `src/forecasting.rs` and `src/evaluation.rs`.

## Instrument identity

`InstrumentId` is the canonical pair of exchange code and symbol, serialized
separately so the same ticker on different exchanges remains distinct.
Provider-specific IDs are stored separately on `InstrumentIdentity`. Current
normalization accepts a restricted ASCII symbol alphabet, trims surrounding
whitespace, and uppercases ASCII letters; it does not guess exchange mappings
or provide Unicode normalization.

## Market data and provenance

`NormalizedCandle` and `Quote` carry a currency and `SourceMetadata`. Event
time and retrieval time are distinct fields. Candle timestamps use UTC epoch
seconds; a candle's end must equal its start plus the declared interval.
OHLCV and quote spread/size invariants are validated explicitly.

`ProviderReportedLive` means the upstream contract reports live data; it is
not evidence of user entitlement. No provider is configured, so this status
does not occur in a production data path. Synthetic data has a distinct
`SyntheticTest` classification. Missing values remain absent, not zero.

## Charts and forecasts

Indicator points carry their own timestamps, so series do not align by array
index. Missing indicator values use `None`. User/indicator overlays have
typed geometry and are not permitted to contain forecast paths.

`ForecastSeries` requires explicit origin, input-data
end time, generation time, horizon, model/version, source, and limitations.
Its timestamps must follow the origin; quantile levels must be ordered and
within `[0, 1]`. The only generated forecast is the explicitly experimental
naive random-walk baseline; no trained model or uncertainty interval is
available. The required display label is
`MODEL ESTIMATE — NOT OBSERVED MARKET DATA`.

Reusable calculations are documented in [INDICATORS.md](INDICATORS.md). The
SMA/RSI calculations used by the existing strategy call the shared Rust
module. Other indicators are library functions but are not yet rendered in
the Web UI.

## Feature and service contracts

`FeatureRegistry` validates IDs, dependencies, cycles, platform availability,
and enabled state. The existing Web UI still has a TypeScript catalog; wiring
the registry as the single runtime catalog across CLI/WASM/UI is deferred to
the service-integration milestone. Current import/backtest operations remain
protected by their existing Rust/WASM checks.

`ApiRequest`/`ApiResponse` use API version 1. `ServiceError` defines stable
codes, retryability, and safe-message validation. These are foundational DTOs,
not yet the active CSV/backtest bridge; that existing bridge still uses the
current JSON-string contract.

All serialized DTOs must be validated after deserialization before use.
