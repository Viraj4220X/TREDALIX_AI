# Hard-parts specification gap report

Updated: current implementation milestone

## Specification inputs

- `MASTER SPECIFICATION.txt`, `impwork.txt`, and `imp2.text` are populated
  specification inputs. Their Python-style raw-string wrappers were read as
  text and were not executed.
- `maincont.txt` was also read as text without executing its Python-style
  wrapper. Its AI Mind milestone is partially implemented as a deterministic
  analyst for a validated local CSV.
- Their overlapping priority scope is real provider-backed market data and
  instrument search, sourced company research, interactive charts, and
  multi-horizon leakage-safe evaluation. This report reconciles that scope
  with the existing `analyze.txt` and the actual implementation.
- Provider terms and deployment entitlement are not fully confirmed, and no
  API key or paid account was supplied. The network provider remains blocked;
  offline work is not presented as external data access.

## Baseline

- Repository: `C:\TREDALIX_AI`; branch: `master`; existing project files are
  untracked. No files were staged or committed.
- Existing architecture: Rust CLI/library, a Rust/WASM adapter, and a
  TypeScript/Vite Web interface. The Web app uses the Rust engine for CSV
  validation and backtesting.
- `cargo test`: passed, 80 tests (72 library unit tests and 8 integration
  tests).
- Core formatting and strict all-target Clippy passed.
- `cargo clippy --all-targets -- -D warnings`: initially failed due to
  existing test-only findings (inconsistent numeric digit grouping and
  `field_reassign_with_default`); these were repaired with test-only
  initializer cleanups, with no behavior changes.
- `npm.cmd test`: passed, 7 tests.
- `npm.cmd run typecheck`: passed.
- `npm.cmd run build`: WASM compilation, TypeScript check, and Vite production
  build passed.
- Rust/WASM adapter: 8 tests; strict all-target Clippy and formatting passed.
- `npm.cmd audit` reported zero vulnerabilities. Production browser smoke test
  verified the local OHLC chart, indicator toggles, baseline overlay and
  1/3/5-bar evaluation on the explicitly synthetic 24-row fixture.
- CLI smoke test on `data\sample.csv` passed.
- `wasm32-unknown-unknown`, `wasm-pack`, Java, and `adb` are available. No
  Tauri shell, Windows C++ build tools, configured Android SDK/NDK, or
  `ANDROID_HOME`/`ANDROID_SDK_ROOT` were verified.

## Gap summary

| Specification area | Current status | Evidence / limitation |
|---|---|---|
| Rust CLI, CSV validation, deterministic backtest | PARTIAL / existing | Implemented and regression-tested; generic costs and a single trailing holdout remain. |
| Web/WASM import and backtest | COMPLETE for current Web scope | Local browser smoke test passed; not a native platform claim. |
| Instrument identity and provider search | PARTIAL | Exchange-qualified Rust `InstrumentId` and provider-ID separation exist; no search service or provider. |
| Provider-agnostic quotes/history | PARTIAL | Typed provider error/capability contracts and search/quote/history traits exist; no real external adapter or entitlement. |
| Normalization, bounded cache, freshness/offline data | PARTIAL | Normalized DTO validation and bounded opt-in in-memory TTL/FIFO candle cache exist; no persistent/platform cache or provider lifecycle. |
| Company research/fundamentals/news | PARTIAL | Sourced profile/fact/news contracts, provenance validation, secure-link checks, and safe ratio utility exist. No provider-backed company data or research UI is enabled. |
| Shared charts and indicator library | PARTIAL | The Web UI now renders interactive local OHLC candles and volume, with Rust-calculated SMA/EMA/Bollinger/RSI/MACD series. No provider feed, VWAP/session model, saved annotations, or drawing/edit history. |
| Backtest audit | PARTIAL | Existing timing/accounting regression tests remain; public dataset validation and strategy calculation error propagation were added. Wider report/audit remains. |
| Forecast interface/baselines | PARTIAL | Deterministic naive random-walk baseline is available locally; no trained model, uncertainty, or accuracy claim. |
| AI Mind local analyst | PARTIAL | Web UI routes dataset questions through a local Rust/WASM deterministic pipeline with evidence-linked claims, validation and session-only audit fingerprints. No LLM, durable memory, company/news context, causal analysis, forecast generation, or remote model gateway is implemented. |
| Repeated walk-forward validation | PARTIAL | Expanding chronological comparison now reports configurable 1–10 bar horizons and per-horizon MAE/RMSE/direction metrics with origin-only training data. No trained model calibration, uncertainty, or trading folds. |
| Forecast overlays | PARTIAL | Web retains the separately labelled naive forecast visualization; candlestick chart is implemented for observed CSV data. Forecast is not embedded in or confused with observed candles. |
| Feature gates | PARTIAL | New baseline and walk-forward gates are wired through Web and Rust/WASM; not all platform features use one shared runtime registry. |
| Versioned service API/errors | PARTIAL | Versioned DTO and error contracts exist; the active adapter still uses JSON strings and string errors. |
| Screenshot/image analysis | NOT_STARTED | No image or vision engine; must remain unavailable. |
| Provider security/network policy | BLOCKED | Official Twelve Data plan/coverage pages and API docs were reviewed as a candidate; no contractual retention/redistribution terms, user deployment class, entitlement, or credentials were confirmed. No provider adapter or exposed browser key. See `docs/PROVIDERS_AND_CACHE.md`. |
| Diagnostics/performance | PARTIAL | Explicit UI errors and input bounds exist; cache and model errors are tested, but structured diagnostics and performance measurements are absent. |
| Windows desktop / Android | PARTIAL | Tauri Windows shell, release executable, native window launch, and NSIS package build are verified; install/uninstall and full UI/IPC workflow remain. Android is not implemented. |
| AI Control Center | PARTIAL | Always-visible UI, local analyst/baseline wiring, safe app-state/navigation tools, bounded session task feed, and local-only validated image preview are implemented; no provider, LLM, vision, full background worker/cancellation, or native end-to-end interaction exists. |

## Milestone 1 implementation

Implement a small Rust `domain` module without changing CLI behavior:

- `src/domain/instrument.rs`: exchange-qualified canonical identity, provider
  identity kept separate, validation, and collision/serialization tests.
- `src/domain/market_data.rs`: normalized quote/candle, currency, source,
  retrieval/event timestamps, freshness classification, and validation.
- `src/domain/chart.rs`: timestamp-aligned indicator series and typed overlay
  boundaries; observed and forecast data remain distinct.
- `src/domain/forecast.rs`: forecast provenance/horizon/target contract and
  origin/future-time invariants only; no model or fabricated prediction.
- `src/domain/feature.rs`: typed capability status, dependency definitions,
  registry validation, and fail-closed feature lookup.
- `src/domain/service.rs`: versioned request/response envelope and stable safe
  error code contract.
- `src/domain/mod.rs` and `src/lib.rs`: public module boundary.
- Root Cargo manifest/lockfile: reuse `serde` and `serde_json` for DTO
  serialization tests.

Implemented the planned domain modules and public boundary, plus the
independent indicator milestone. Tests cover
invalid values, JSON round trips, exchange-qualified identity, dependency and
platform failures, forecast timestamp ordering, and stable API version
handling. Each new source file remains below 700 lines. The FeatureRegistry
and versioned service DTOs are not yet the runtime source of truth for the
existing TypeScript UI/WASM bridge; that integration remains PARTIAL.

This milestone does not claim provider search, a live quote, company research,
a prediction model, screen capture, or native platform support.

The indicator module implements SMA, EMA, Cutler RSI, MACD, Bollinger Bands,
and ATR; the existing strategy now reuses SMA/RSI without changing its
long-only signal thresholds. Hand-calculated, warm-up, missing-input,
parameter, and numeric-overflow cases are tested.

The existing backtest entry point now revalidates untrusted public `Dataset`
values before execution, including OHLC and timestamp order. Strategy
calculation failures propagate through a new fallible hook while the original
infallible trait method remains source-compatible.

## Offline research / provider framework additions

- `src/forecasting.rs` implements a typed, deterministic random-walk baseline;
  it repeats the last close and does not claim prediction skill.
- `src/evaluation.rs` implements expanding chronological one-step evaluation
  against a historical-drift baseline, reporting MAE/RMSE/directional accuracy.
- `src/providers.rs` adds typed provider errors, capability metadata, adapter
  traits, and an opt-in bounded memory-only candle cache.
- `src-wasm/src/lib.rs` exposes baseline analysis and enforces its feature
  gates; the Web data page renders an explicit estimate/observed series split.
- No real provider, persistent cache, trained forecast model, image analysis,
  or native shell was introduced.

## Current milestone additions

- Added `src/domain/research.rs`: sourced company profiles, period-labelled
  fundamental facts, source-linked news items, bounded validation, secure URL
  checks, snapshot validation, and a ratio helper that returns no result for
  non-finite or zero-denominator inputs. This is a data contract only; it does
  not create company data.
- Added a provider trait for company research; no fake provider is registered.
- Added a bounded candlestick chart payload in the WASM adapter, aggregating
  OHLC/volume when more than 1,200 points are supplied. SMA, EMA, Bollinger,
  RSI, MACD, and ATR series are calculated in Rust and returned only while the Technical Indicators feature
  is enabled.
- Added interactive browser charts with candlesticks, volume, SMA/EMA,
  Bollinger bands, RSI and MACD, plus touch/mouse pan/zoom, visibility toggles,
  and required TradingView attribution. Data remains the validated local CSV.
- Extended expanding-window evaluation to user-selected bar horizons while
  preserving the existing one-step `WalkForwardConfig` API. Each horizon's
  predictions use only observations at or before its forecast origin.
- Added official provider research notes and linked them from
  `docs/PROVIDERS_AND_CACHE.md`.

## Deferred work / prerequisites

1. Confirm provider/deployment choice and provider-specific retention,
   redistribution, and data entitlement before implementing real API access.
2. Implement a secure server/native provider path; never place a private API
   key in the frontend bundle.
3. Add saved chart drawings/annotations and a verified exchange session
   calendar before supporting VWAP or session-aware forecast timestamps.
4. Build/test bounded caches, offline freshness, and provider errors only
   against the chosen provider contract.
5. Implement prediction and walk-forward validation only with leakage tests,
   baseline comparisons, and explicit data sufficiency.
6. Keep vision/screen-capture unavailable until a real engine/platform adapter
   and consent workflow exist.
7. Install native toolchains only after user approval; then build and test
   Windows and Android separately.

## Source file inventory

At baseline, Rust/frontend application sources are modular and below 700
lines. The largest relevant existing files are `src/backtest/engine.rs`
(596 lines), `frontend/src/main.ts` (408 lines), and
`src-wasm/src/lib.rs` (394 lines). Build outputs under `target/`,
`node_modules/`, `frontend/dist/`, and generated WASM are excluded.

## Local AI Mind foundation — 25 October 2026

- Added `src/analyst.rs` plus keyword-intent routing in
  `src/analyst/intent.rs`. The local pipeline validates a question, extracts
  structured features from validated candles, calculates return, close-based
  drawdown and Rust SMA-20/RSI-14/ATR-14, drafts deterministic answers, links
  factual claims to evidence, validates the output and creates session-only
  audit fingerprints.
- Added `src-wasm/src/analyst.rs` and wired the analyst through the existing
  Rust/WASM boundary. CSV size, metadata, data quality, and Feature Manager
  gates are checked before analysis.
- Added the **Offline deterministic analyst** capability on the Web Data
  manager. Answers show the selected instrument, synthetic/unverified class,
  source label, as-of timestamps, assumptions, limitations and data gaps.
  The result is held in browser memory and has an explicit clear control.
- Regression coverage includes exact synthetic return results, citations,
  indicator reuse, prompt-injection/safety boundaries, unsupported forecasts
  and causality, invalid CSV and disabled feature gates. HTML escaping is
  covered in frontend tests.
- Remote LLM, durable research memory, user-preference memory, market/news
  providers, company research service, and trained forecasts remain
  NOT_STARTED or BLOCKED; this work does not imply those features exist.
- Current verification: Rust core 84 tests, Rust/WASM 10 tests, frontend 10
  tests, strict Clippy, formatting, typecheck and production build passed.
  Browser smoke test on the clearly synthetic 24-row `data\sample.csv`
  produced a locally calculated 22.7723% close-to-close return with evidence
  displayed; this is fixture-only evidence, not market performance.
- Audit fingerprints use non-cryptographic FNV-1a and are for correlation,
  not tamper detection. No conversation or dataset is durably persisted.
- Changed source-size audit: `src/analyst.rs` 657 lines,
  `src-wasm/src/analyst.rs` 131, `frontend/src/main.ts` 568, and
  `src-wasm/src/lib.rs` 603; each is below the 700-line ceiling.
