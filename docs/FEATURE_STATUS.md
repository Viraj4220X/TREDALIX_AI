# Feature status

Feature implementation status is maintained in
`frontend/src/settings/features.ts`. Web is the supported platform.
The Windows desktop shell and installer build are implemented and the
executable window has launched, but the app is not declared generally
supported until installer install/uninstall and native workflows are
verified. Android is not implemented.

## Native Gemma upgrade and requested capabilities

The legacy external model-server integration has been removed from the
application; unrelated local model files were not changed. The desktop path
uses the pinned
`ggml-org/gemma-3-4b-it-GGUF` Gemma 3 4B IT Q4_K_M model, its matching
multimodal projector, and the llama.cpp CPU runtime. The public model files
were downloaded to the ignored `models/gemma-3-4b/` directory and their
expected sizes, GGUF headers, and pinned SHA-256 values were checked. Direct
CPU text, Hinglish, and image smoke requests succeeded; image descriptions
remain approximate and are not market evidence. The new Tauri UI/backend
integration compiles and its unit tests pass; model status/tool-registry IPC
and AI UI rendering were smoke-tested. Streamed inference, screen OCR against
a real desktop capture, model loading/chat through Tauri IPC, installer
installation, and memory stress behavior remain unverified. In the latest
build run, this 16 GB host had 4,855 MiB free, below the 6 GiB safety
threshold, so loading was blocked and no in-app model inference was attempted.

| Requested capability | Status | Implemented behavior and limits |
|---|---|---|
| Central model manager | PARTIAL | Discovers the one pinned Gemma profile in a chosen local folder; downloads, hashes, checks GGUF headers, loads/unloads, reports status, and removes only its two model files. It does not update model revisions, manage arbitrary model families, or preserve multiple active profiles. |
| Memory and inference monitoring | PARTIAL | Targets a 16 GB host, reports physical RAM and the model process working set, blocks loading unless at least 6 GiB free RAM can be measured, serializes inference, uses a 4096-token context and CPU-only execution, and reports throughput when returned. CPU utilization, VRAM, context occupancy, and sustained performance are not monitored; GPU acceleration is not implemented. |
| Streamed chat, cancellation, and timeout | PARTIAL | Tauri streams local llama.cpp SSE deltas to the UI; a 180-second request timeout is applied. Stop unloads the inference process to interrupt generation and release memory. Parser/IPC code compiles and parser tests pass; live streaming, timeout, and stop behavior have not been exercised with the model. |
| Screen understanding and OCR | PARTIAL | A user-triggered system chooser selects a screen/window; exactly one frame is sent to Gemma and local Windows OCR, then the stream is closed. Image and OCR size limits are enforced. No continuous monitoring or screen recording is performed. Real screenshot OCR/vision and application/window identification are unverified. |
| Desktop application control | NOT IMPLEMENTED | No process/window inventory, UI Automation, application launching/switching, mouse/keyboard input, or scroll control. |
| Coding Agent | NOT IMPLEMENTED | No project-scoped source explorer/editor, model-generated patch review/apply workflow, compiler diagnostics integration, or controlled test runner is present. |
| Central sensitive-action permission manager | NOT IMPLEMENTED | No centralized permission service. Existing screen capture uses the system chooser and model-file removal requires confirmation; no broad desktop-control capability is exposed. |
| Persistent conversation/knowledge/correction memory | NOT IMPLEMENTED | Chat is session-only. No persistent user memory, document retrieval index, correction memory, or user-configurable retention controls exist. |
| Automatically discovered tool registry | PARTIAL | Rust exposes stable unique IDs, descriptions, JSON schemas, permissions, and bounded local calculation executors; the registry is supplied to Gemma's strict tool-call protocol. Adding an executor still requires Rust code changes; dynamic plugin discovery is not implemented. |
| Tool validation, timeout, retries, independent verification | PARTIAL | Unknown IDs, unexpected fields, wrong types, oversized inputs, and out-of-range values fail closed. Tools expose a time budget and errors are surfaced. There are no automatic retries or cancellation; the synchronous execution limit is checked after execution, and results are explicitly marked as not independently verified. |
| Answer evaluation and mathematical verification | PARTIAL | Rust Black-Scholes/Greeks, implied-volatility, position-size, and RSI tools have bounded inputs and unit/reference tests. No independent second calculation engine or general answer evaluator exists. Model and tool outputs are not presented as confirmed financial results. |
| Complete offline operation | PARTIAL | Local Gemma, CSV analysis, forecasting, charts, and Rust calculations can operate without cloud AI. Internet research is separate, reports “Web Search Not Configured” when no local provider is configured, and makes no request in that state. There is no persistent offline document knowledge base. |
| Database backup/export/recovery/migrations | NOT IMPLEMENTED | The app has no AI memory database or migration/recovery workflow. Existing localStorage run summaries and report exports are not a database-backup system. |
| Modular plugin architecture | NOT IMPLEMENTED | The bounded tool registry is modular at the source-code level but there is no loadable plugin contract, sandbox, installer, or permissioned plugin lifecycle. |
| Future model switching | PARTIAL | The UI/model process can unload and reload the pinned profile from a selected directory. A provider-neutral multi-model runtime adapter and compatibility discovery are not implemented. |
| Diagnostics, crash recovery, performance, health | PARTIAL | Model status checks process liveness, verifies pinned core runtime files, reports available RAM and approximate last-turn generation speed, and writes a local llama-server log. Automatic crash recovery, structured diagnostics export, and sustained performance monitoring are absent. |
| Preserve existing application features | IMPLEMENTED for this change | The AI workspace presents native model management, local chat, image input, one-shot screen analysis, and registered tools while retaining the CSV analyst, provider search state, forecast/evaluation, market data/chart and backtest areas. Frontend and backend regression tests pass. Installer installation and end-to-end Gemma/desktop capture interactions remain unverified. |

## Available in the Web app

- Local CSV import and Rust-backed data validation.
- Existing SMA-9/SMA-21/RSI-14 offline long-only backtest.
- Rust library calculations for SMA, EMA, Cutler RSI, MACD, Bollinger Bands,
  and ATR, displayed in an interactive local OHLCV candlestick chart with
  bounded Rust-calculated SMA/EMA/Bollinger/RSI/MACD/ATR overlays.
- Experimental naive random-walk baseline forecast and expanding,
  configurable multi-horizon walk-forward comparison, using explicit
  exchange/currency input and local CSV data only.
- Offline deterministic analyst for a validated local CSV: keyword intent
  routing, Rust-derived return/drawdown/indicators, evidence-linked claims,
  provenance and data gaps. Responses and their audit fingerprints exist only
  in session memory; no LLM or network call is made.
- Visible AI Control Center beside the theme button in browser and Tauri UI.
  It runs the existing local analyst and baseline/walk-forward engine, reports
  real task states and safe app capabilities, and offers bounded local tools.
  The desktop AI workspace now includes Gemma model management, local chat,
  user-selected image input, and schema-validated calculations. These native
  features are unavailable in browser mode. Model answers and image readings
  are not market evidence or independently verified financial results.
- Web search is provider-based but currently unconfigured unless local provider
  settings are detected. The UI reports “Web Search Not Configured” when they
  are absent; no search request is made in that state. The optional legacy
  Google CSE adapter reads its API key and Search Engine ID only from the
  desktop process environment; neither value is stored in frontend settings or
  bundled in frontend assets. Status detection does not validate credentials
  or contact Google; requests happen only after the user initiates a search.
  Local Gemma chat remains separate from web search.
- Portfolio performance and risk metrics from the Rust engine.
- Browser-local settings, theme, and compact run history.
- Current-run JSON and CSV report downloads.

Technical indicators and strategy remain experimental analysis tools, not a
newly validated predictive model. The chart displays local file data only and
does not imply a provider connection.

## Partial, planned, or unavailable

- Walk-forward: multi-horizon random-walk vs historical-drift metrics are
  implemented; trained-model evaluation, uncertainty calibration, regime
  coverage, and trading strategy fold evaluation are not.
- Company research: sourced profile/fundamental/news data contracts and
  validation exist; there is no provider-backed research service or UI.
- Trained prediction models and cloud language models are not implemented.
  The deterministic CSV analyst remains separate from the local Gemma
  conversation and is not an LLM or a forecast generator.
- AI task execution is limited to synchronous local Rust/WASM operations and
  four local calculation tools (option pricing/Greeks, implied volatility,
  position sizing, and RSI). There is no remote tool-calling model, general
  background queue, or cancellation of an in-progress WASM call.
- Real provider quotes, candles, fundamentals, and news: provider contracts
  exist, but no provider adapter or entitlement is configured. Web search has
  a provider interface and an optional legacy Google CSE adapter; it remains
  unavailable until valid local credentials and a Search Engine ID are
  confirmed and configured. Google Custom Search JSON API is scheduled to
  discontinue on 2027-01-01. No search provider configuration is assumed.
- Offline candle cache: bounded in-memory cache exists; persistent storage,
  migrations, and provider-specific freshness policy are not implemented.
- Saved chart drawings, forecast persistence, verified exchange-session
  calendar, and provider-sourced candles are not implemented. The local
  forecast overlay remains separately labelled from observed candles.
- Live market data, paper trading, broker execution, and notifications:
  unavailable or planned; no provider or trading adapter exists.
- Cloud sync: planned; no cloud service is implemented.
- Windows desktop: Tauri v2 shell and release executable are implemented;
  the app window launches and NSIS packaging succeeds. Installer
  installation/uninstallation and full native workflow remain unverified. See
  [WINDOWS_DESKTOP.md](WINDOWS_DESKTOP.md).
- Android: no Android shell, APK, or device validation.

No feature status is a profitability or prediction-accuracy claim. Sample
data remains synthetic/unverified.
