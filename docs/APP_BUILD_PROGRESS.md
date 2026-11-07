# Application build progress

Updated: 25 October 2026

## Current stage

The Web application slice is implemented and verified. The Windows Tauri
shell and release executable compile, its main window opens, and the NSIS
setup package builds. Installer installation/uninstallation and complete
native UI/IPC workflow checks remain; Android is unimplemented.

## Completed

- Read `apptxtfinal.txt` as requirements; did not execute its Python-style
  prompt-generator contents.
- Audited the existing Rust CLI/library and recorded the starting state in
  `docs/APP_BASELINE_AUDIT.md`.
- Documented the Rust/WASM + shared TypeScript/Vite architecture.
- Added `src-wasm/`, a thin adapter to existing Rust CSV validation and
  backtesting, including 10 MiB CSV limits and feature enforcement at the
  Rust boundary.
- Added the responsive TypeScript/Vite Web interface: dashboard, local CSV
  validation, backtest configuration/results, feature manager, settings, run
  history, and JSON/CSV report exports.
- Added versioned browser-local settings and feature availability rules.
  Only Web is declared as a supported platform until native shells exist.
- Added a Tauri v2 Windows shell with bundled frontend, restricted window
  capability, Rust IPC readiness/capability status, and HTTP(S)-only external
  link opening. It deliberately does not expose a separate local HTTP server
  or bind a network port.
- Added an always-visible AI Control Center for browser and desktop builds,
  connected to the actual local Rust/WASM analyst and baseline evaluator.
  It reports provider/LLM/vision limitations, safe app state, typed local
  navigation/state tools, bounded task history, and local-only validated
  image preview without pretending vision is available.
- Separated the forecast chart's projected series from observed closes;
  forecast line begins at projected timestamps and is visually labelled as
  an unobserved estimate.
- Added user, setup, test, and feature-status documentation.
- Updated Vite to 8.3.5 after the initial audit found vulnerabilities in
  8.2.0. The current npm audit reports zero vulnerabilities.

## Verified checks

- `cargo test`: 85 core tests passed (77 unit, 8 integration).
- `cargo test --manifest-path src-wasm\Cargo.toml`: 10 passed.
- `cargo fmt --check`: passed for the existing core.
- `cargo clippy -- -D warnings`: passed for the existing core.
- `cargo clippy --manifest-path src-wasm\Cargo.toml -- -D warnings`: passed.
- `npm.cmd test`: 19 passed in the current frontend suite.
- `npm.cmd run typecheck`: passed.
- `npm.cmd run build`: WASM compilation, typecheck, and Vite production build
  passed, including the AI Control Center.
- `npm.cmd audit`: zero vulnerabilities.
- Production browser smoke test: loaded `data/sample.csv`, validated all 24
  rows through Rust/WASM, and ran the Rust strategy/backtest successfully.
  The synthetic fixture returned zero trades, which the UI reported without
  inventing results.
- Browser reload confirmed the theme persists in local storage and run
  history is retained. Browser checks also confirmed that Backtesting stays
  disabled without validated data or when the feature is switched off, and
  that the switch state persists after reload.
- Frontend/WASM source files are at or below 700 lines.
- `git diff --check`: passed.
- `npm.cmd run tauri:build -- --no-bundle`: compiled the release executable
  successfully. This does not verify native UI startup or installer packaging.
- `npm.cmd run tauri:build`: produced the NSIS setup package
  `TREDALIX AI_0.1.0_x64-setup.exe` (2,159,724 bytes; 2.06 MiB).
- The release executable opened a native window titled `TREDALIX AI` during
  an 8-second smoke check and was then closed. In-app UI interactions and IPC
  result rendering were not verified.
- Production browser preview smoke-tested the AI button, eight status cards,
  safe app snapshot, local CSV analyst, and baseline scenario graph on a
  synthetic dataset. The scenario remained visibly labelled as unobserved.
  Local image preview, task activity, and AI screen navigation were also
  exercised in the browser. Frontend source modules remain under 700 lines.
- Installer installation/uninstallation and a full native Windows workflow
  with AI interactions are still not tested.

## Not implemented or not verified

- Tauri installer installation/uninstallation and in-app IPC/UI workflow
  verification remain.
- No Android app, debug APK, device installation, or emulator test exists.
- The available Web slice has not been tested on other browsers or devices.
- No real-market provider, live feed, paper trading, broker execution,
  trained prediction model, trained-model calibration, or exchange calendar
  is implemented.
- Browser imports are session-memory only; browser-local settings and run
  summaries persist separately. Native settings/storage behavior does not
  exist yet.

## Platform blockers

- Windows release executable compilation is verified in this environment.
  The app window and NSIS build are verified; installer setup/uninstall and
  in-app AI workflow remain to be checked.
- Android SDK/NDK configuration, Android Rust targets, and an emulator or
  device are not verified. Java and `adb.exe` alone are not sufficient.
- Native settings/storage behavior and Android packaging are not implemented.

## Next actions

1. Verify in-app IPC/UI interactions and install/uninstall the NSIS package
   before claiming desktop support.
2. Configure Android toolchains only with user approval, then build and install
   a debug APK before claiming Android support.
3. Continue regression tests and preserve the existing Rust CLI and core.

No files have been staged or committed.

## Hard-parts specification progress — 25 October 2026

- `analyze.txt` is now populated with a 570-line hard-parts specification
  embedded in Python-style source text. It was read only; it was not executed.
- `MASTER SPECIFICATION.txt`, `impwork.txt`, and `imp2.text` are now
  populated; their overlapping priorities are reconciled in
  `docs/HARD_PARTS_GAP_REPORT.md`.
- Added the baseline/gap report and file plan at
  `docs/HARD_PARTS_GAP_REPORT.md`.
- Implemented Rust domain contracts for exchange-qualified instrument
  identity, normalized quotes/candles with provenance, timestamp-aligned
  indicator series, user/indicator overlays, forecast DTOs with no-look-ahead
  timestamp validation, dependency-aware feature registry types, and
  versioned service request/response/error DTOs.
- Added `serde`/`serde_json` serialization support to the shared Rust core.
  Existing CLI and WASM JSON-string interfaces remain unchanged.
- New tests exercise identity collisions, validation and serialization,
  feature dependency/platform gates, forecast horizon/origin/quantile rules,
  overlay coordinates, and API version/error validation.
- Post-change Rust tests pass (68 library unit + 8 integration tests).
  `cargo clippy --all-targets -- -D warnings` passes.
- Existing WASM adapter tests, frontend tests/typecheck/build, and CLI CSV
  inspection smoke test still pass.
- Provider search/quotes/fundamentals/news, the unified shared feature
  catalog, technical indicator overlays, trained forecast models, full
  multi-horizon walk-forward validation, screenshot analysis, and native
  shells remain incomplete or blocked. See
  `docs/HARD_PARTS_GAP_REPORT.md`.

### Additional hard-parts work

- Added `src/indicators/` with SMA, EMA, Cutler RSI, MACD, Bollinger Bands,
  and ATR; calculations are causal and return aligned values with explicit
  warm-up/missing entries.
- Refactored the existing SMA/RSI strategy to reuse the shared implementations.
- The backtest engine now validates public `Dataset` inputs before strategy
  execution and propagates indicator calculation errors through its
  fallible strategy hook. The original `Strategy::action` method remains
  available for existing strategy implementations.
- Added hand-calculated and failure-path tests plus
  `docs/INDICATORS.md`. VWAP remains intentionally absent until exchange
  sessions and volume semantics are supported.
- The final Rust suite now has 68 library unit tests and 8 integration tests.

### Forecast and evaluation additions — 25 October 2026

- Added an experimental naive random-walk forecast using the last validated
  close, explicit exchange/currency/source provenance, future timestamps, and
  no fabricated uncertainty interval.
- Added expanding chronological one-step walk-forward evaluation comparing
  random walk against historical drift, with per-fold sample boundaries,
  MAE, RMSE, directional accuracy, insufficient-history handling, and a
  future-data leakage sentinel test.
- Exposed both through the Rust/WASM boundary with feature gates enforced in
  Rust, then added a local Web research panel and a clearly separated,
  labelled forecast line over observed closes.
- Added a bounded in-memory candle cache and provider capability/error/adapter
  contracts. These do not implement a real network provider or persistent
  cache; official provider terms and storage adapters remain prerequisites.

### Priority modules 1–4 continuation

- Reconciled `MASTER SPECIFICATION.txt`, `impwork.txt`, and `imp2.text`;
  these files are populated and were read as text, not executed.
- Added sourced company profile/fundamental/news contracts and validation.
  No company content is fabricated or fetched; see
  `docs/RESEARCH_DATA.md`.
- Reviewed official Twelve Data docs, plan coverage and pricing pages as a
  candidate. Its published coverage is plan-dependent and Indian equities
  are listed as EOD for the reviewed individual tier. No account, entitlement,
  contract-level retention/redistribution terms, or secure Web backend is
  confirmed. Real-provider work remains blocked; see
  `docs/PROVIDERS_AND_CACHE.md`.
- Added an interactive TradingView Lightweight Charts™ visualization of local
  validated OHLCV, with Rust-calculated SMA/EMA/Bollinger/RSI/MACD overlays and
  a bounded chart payload. The required upstream attribution is shown in UI.
- Extended Rust chronological evaluation to selectable horizons, retaining
  the original one-step config API and adding origin-only multi-horizon
  metrics. No trained model or calibration is claimed.
- Native Windows/Android shells, sourced company data, provider access,
  annotation persistence, session calendars, and trained forecasts remain
  incomplete or blocked.

### Verification for the priority continuation

- Rust core: 72 library unit tests and 8 integration tests passed.
- Rust/WASM: 8 tests passed; core/adapter formatting and strict Clippy passed.
- Frontend: 7 tests, TypeScript check, production build and npm audit passed;
  audit reported 0 vulnerabilities.
- Browser production preview: validated the 24-row synthetic fixture,
  rendered interactive OHLCV/indicator charts, toggled SMA and ATR, ran
  1/3/5-bar expanding evaluation, and verified the clearly labelled forecast
  overlay.
- CLI CSV inspection smoke test passed. No desktop/Android or real-provider
  claims are made.

### Local deterministic analyst milestone — 25 October 2026

- Implemented a deterministic local analyst for validated CSV questions. The
  pipeline classifies a bounded user question, derives structured candle
  evidence, reuses Rust SMA/RSI/ATR calculations, computes close-to-close
  return and maximum close-based drawdown, drafts a fixed response, verifies
  claim evidence references, and creates non-cryptographic audit fingerprints.
- Wired the pipeline through Rust/WASM and an enabled-by-default, dependency-
  gated Web feature. The Data manager displays evidence, user-provided source
  label, data class, as-of timestamps, assumptions, uncertainty, gaps and a
  clear-session control.
- Answers are session-only. No LLM, external API, provider data, news, durable
  memory or trained forecast was added. Forecast, causality, company research,
  comparison and unsafe trading-advice requests are answered with explicit
  capability boundaries.
- Verification after implementation: Rust core 84 tests; WASM adapter 10
  tests; frontend 10 tests; formatting, strict Clippy, TypeScript typecheck
  and production build passed.
- Production browser smoke test: local Rust/WASM validated the 24-row
  `data\sample.csv` after it was marked `SYNTHETIC / TEST`; the analyst
  returned a 22.7723% close-to-close change with evidence and provenance.
  Clearing the result removed it from session memory. Synthetic fixture
  values do not establish market performance.
