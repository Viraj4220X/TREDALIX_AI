# Testing and verified results

## Commands

Run from the repository root:

```powershell
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --manifest-path src-wasm\Cargo.toml
cargo fmt --manifest-path src-wasm\Cargo.toml --check
cargo clippy --manifest-path src-wasm\Cargo.toml --all-targets -- -D warnings
```

Run from `frontend\`:

```powershell
npm.cmd test
npm.cmd run typecheck
npm.cmd audit
npm.cmd run build
```

`npm.cmd run build` compiles the WebAssembly adapter, type-checks TypeScript,
and builds the Vite production bundle. For an interactive browser preview,
run `npm.cmd run preview -- --host 127.0.0.1`.

## Verified results

The preceding implementation passed:

- Rust core: 85 tests (77 library unit tests and 8 integration tests),
  0 failures.
- Rust/WASM adapter: 10 tests, 0 failures.
- Core and adapter all-target Clippy with warnings denied.
- Core and adapter formatting checks.
- Frontend: 21 tests, 0 failures.
- AI workspace tests cover the always-visible entry point, privacy-limited app
  snapshot, typed tool allow-list/input bounds, bounded task history, image
  magic-byte validation, and a forecast series that begins only after observed
  data.
- TypeScript typecheck and Vite production Web build.
- Offline analyst regression tests verify deterministic return calculations,
  evidence references, source/data-class provenance, indicator reuse, input
  limits, unsupported forecast/causality boundaries, and adversarial prompt
  requests. Frontend tests verify feature gating and HTML escaping.
- Experimental baseline forecast and multi-horizon walk-forward UI is gated
  by settings and calls the Rust/WASM implementation.
- Production browser smoke test loaded the 24-row synthetic CSV, rendered the
  interactive candlestick/volume view and Rust-computed indicators, verified
  SMA and ATR visibility toggles, then ran three chronological folds across
  1-, 3-, and 5-bar horizons (9 test samples per horizon). The 124.0000 naive estimate
  stayed labelled as a model estimate, with `SYNTHETIC / TEST` provenance
  visible. No predictive or market-performance claim was made.
- npm dependency audit: 0 known vulnerabilities after upgrading Vite to
  8.3.5.
- Browser smoke test using `data/sample.csv`: 24 rows validated by Rust/WASM,
  followed by a successful actual Rust backtest. The synthetic fixture
  produced no trades; this was reported as zero rather than fabricated data.
- Browser reload check confirmed theme persistence and retained run summary.
- Browser gating check confirmed the backtest action remains disabled without
  validated data and when Backtesting is switched off; the feature preference
  persisted after reload.
- CLI smoke command:
  `cargo run -- inspect SBIN data\sample.csv --synthetic --interval-seconds 60`
  passed and reported 24 rows as `SYNTHETIC / TEST`.
- Production browser smoke test: imported the 24-row
  `data\sample.csv` fixture, explicitly marked it synthetic, validated it,
  asked “What was the return?”, and received a local Rust-backed result
  (23.0000 price units / 22.7723%) with 11 evidence items, synthetic label,
  source/as-of details, and “No remote model was used.” Clearing the response
  removed it from session memory. These fixture values are not market evidence.
- Windows Tauri release executable: `npm.cmd run tauri:build -- --no-bundle`
  completed and produced `frontend\src-tauri\target\release\tredalix-ai-desktop.exe`.
  Native WebView launch, installer packaging, and signed distribution were
  not verified by this compile-only build.
- Windows Tauri native launch: the release executable stayed running for an
  8-second smoke check and exposed the `TREDALIX AI` main window title; the
  process was then closed. This does not verify UI interactions or IPC results.
- Windows NSIS package: `npm.cmd run tauri:build` completed and generated
  `frontend\src-tauri\target\release\bundle\nsis\TREDALIX AI_0.1.0_x64-setup.exe`
  (2,159,724 bytes; 2.06 MiB). Installation, uninstallation, and signing were not tested.
- Browser production-preview smoke test verified the AI button opens the
  workspace, eight capability cards and idle state render, app-state inspection
  returns only the bounded snapshot, and the existing local analyst answered a
  question on the 24-row `SYNTHETIC / TEST` CSV. The baseline scenario ran
  from the AI workspace and remained there; the estimate label, separate
  observed/estimate legend, and walk-forward section rendered. No provider,
  LLM, or vision call was made. A local PNG file also passed content and
  dimension checks and displayed through a `blob:` preview; no interpretation
  or upload occurred.
## Native Gemma milestone

The current native-model changes passed:

- Root Rust: 90 tests (82 library plus 8 integration), 0 failures; root format
  check passed.
- Rust/WASM adapter: 10 tests, 0 failures; adapter format check passed.
- Tauri backend: 15 unit tests, 0 failures; Tauri format check passed.
- Frontend: 25 tests, 0 failures; TypeScript typecheck passed.
- Production Tauri/Windows build completed with the CPU runtime resources
  included. The NSIS installer is
  `frontend\src-tauri\target-verify\release\bundle\nsis\TREDALIX AI_0.1.0_x64-setup.exe`;
  final byte size is 18,249,007 bytes (17.40 MiB).
- The final built Tauri executable launched and kept the `TREDALIX AI` window
  open for a 15-second startup smoke test. In the native UI/IPC smoke test,
  Tauri returned the pinned model status and four registered calculation tools;
  the UI rendered Gemma manager, chat, and tool registry.
- This Windows host reported 16,215 MiB physical RAM and 4,855 MiB free RAM.
  The updated model manager requires at least 6 GiB measurable free RAM and
  reports model-process working set when available, so Gemma loading was
  correctly blocked/not attempted in this session.
- The final un-loaded Windows Tauri executable stayed open for 15 seconds and
  displayed the `TREDALIX AI` window. Its measured process working set was
  27.9 MiB. This is startup memory only, not loaded-model or inference memory.
- Downloaded assets were verified locally: Gemma Q4_K_M weights
  2,489,757,856 bytes, SHA-256
  `882e8d2db44dc554fb0ea5077cb7e4bc49e7342a1f0da57901c0802ea21a0863`;
  matching projector 851,251,104 bytes, SHA-256
  `8c0fb064b019a6972856aaae2c7e4792858af3ca4561be2dbf649123ba6c40cb`.
  The GGUF header parsed as `gemma3`, version 3, 444 tensors, 34 blocks,
  embedding length 2560, context metadata 131,072. llama.cpp CPU release
  b12077 loaded the model and multimodal projector with GPU layers set to 0.
- Direct local CPU inference returned a Hinglish response. A synthetic PNG
  smoke test transcribed `TEST CHART`, `PRICE 123.45`, and `Trend line:
  rising`, but incorrectly described the white background as black/fading.
  This confirms image input works while demonstrating that visual output is
  fallible. These direct runtime checks did not exercise the Tauri chat IPC.
- Direct model output for a financial prompt followed the registered JSON
  tool-call protocol (in a fenced JSON block); Rust parser and tool validation
  tests pass. The complete model-to-tool Tauri call was not run because the
  memory guard blocked model loading.
- The current SSE chunk parser, response-size limits, memory guard, and OCR
  input bounds have Rust unit coverage. The Tauri stream channel, cancellation
  through model-process unload, Windows OCR on a real screenshot, and Gemma
  inference after the streaming change have not been tested end to end.

The Gemma source is `ggml-org/gemma-3-4b-it-GGUF` at revision
`d0976223747697cb51e056d85c532013931fe52e`; the CPU runtime archive is the
official llama.cpp `b12077` Windows x64 release. The archive SHA-256 was
`12ddff27f017f22601ffc5e9e5415a181dc4c99ea57a10dd9a5afee5fac7fae6`.
Gemma Terms of Use apply; this report does not accept terms on the user's
behalf.

The browser smoke test was performed against the local Vite production
preview, not a Windows native shell or Android device.

## Not tested

- Gemma chat and image understanding through Tauri IPC, loaded-model/inference
  RAM and speed, 16 GB target performance, installer install/uninstall, and
  signed distribution.
- A robust Devanagari Hindi test; the first PowerShell request was corrupted
  by terminal encoding. Hinglish was tested successfully.
- Android APK build/install: SDK/NDK configuration and a target device or
  emulator are not verified.
- Other browser engines, mobile browsers, and large CSV performance.
- Public Web deployment or signed installers.

Check [APP_BUILD_PROGRESS.md](APP_BUILD_PROGRESS.md) for current platform
blockers and [FEATURE_STATUS.md](FEATURE_STATUS.md) for availability.
