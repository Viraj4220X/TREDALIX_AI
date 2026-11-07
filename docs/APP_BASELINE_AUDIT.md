# Application baseline audit

Audit date: 25 October 2026
Repository root: `C:\TREDALIX_AI`

## Repository state

- The working tree is on `master`; the repository has no commits yet.
- Existing project files are untracked. They were left unchanged during this audit.
- `target/` is present as Cargo build output. `.gitignore` excludes `/target/`,
  environment files, logs, and common OS metadata.
- The application requirements are in `apptxtfinal.txt`. It is a Python-style
  prompt generator containing a cross-platform application specification; it
  was read as text and was not executed.

## Existing project inventory

- `src/main.rs`: Rust CLI with `inspect` and `backtest` commands.
- `src/lib.rs`: reusable Rust library exporting data validation and backtesting.
- `src/data/`: candle model, CSV parser, timestamp conversion, validation,
  session-gap reporting, and quality output.
- `src/backtest/`: configuration, strategy, execution, portfolio, risk,
  metrics, engine, and report generation.
- `tests/`: Rust unit and integration coverage for CSV and backtesting.
- `data/`: sample CSV fixtures. These are test data, not verified market history.
- `docs/`: data format/source documentation and backtesting assumptions.
- No README, frontend, Tauri integration, WASM adapter, application settings
  store, or GUI test suite existed at audit time.

## Verified functionality

- CLI commands: `cargo run -- inspect SYMBOL PATH [options]` and
  `cargo run -- backtest SYMBOL PATH [options]`.
- CSV input requires timestamp/open/high/low/close; volume is optional.
- Data validation checks finite positive OHLC values, OHLC consistency,
  optional volume, and strictly ascending timestamps.
- The simulator is long-only, uses next-candle-open fills, applies configured
  generic fee/slippage assumptions, and writes reports without overwriting an
  existing report directory.
- Session calendars, live data, broker orders, repeated walk-forward
  evaluation, and genuine-data provenance are not implemented.
- Existing sample datasets are synthetic or unverified and must not be used to
  claim real-market performance.

## Baseline commands and results

Commands were run from the repository root on 25 October 2026:

| Command | Result |
|---|---|
| `cargo test` | Passed: 41 tests (33 library unit tests, 8 integration tests); no failures. |
| `cargo fmt --check` | Passed. |
| `cargo clippy -- -D warnings` | Passed. |
| `cargo run -- inspect SBIN data\sample.csv --interval-seconds 300 --synthetic` | Passed; 24 rows reported as synthetic/test, with provenance and calendar limitations shown. |

## Toolchain and platform availability

- Windows 11; Rust/Cargo `1.99.1`, target `x86_64-pc-windows-msvc`.
- Node `v26.6.0`; `npm.cmd` `11.18.0`. PowerShell blocks the `npm.ps1`
  shim, so use `npm.cmd`.
- Java `17.0.1` is installed.
- `adb.exe` was found at `C:\Program Files\Sensitivity\adb.exe`, but Android
  SDK/NDK, Android Studio, configured SDK environment variables, and an
  emulator were not verified. This executable alone does not establish a
  usable Android build environment.
- `cl.exe`, `link.exe`, `msbuild.exe`, Tauri CLI, `wasm-pack`, and
  `wasm-bindgen` were not found on PATH. Windows Tauri packaging is therefore
  blocked until the supported Microsoft C++ Build Tools/WebView2 requirements
  are available.
- The installed Rust target list contains only `x86_64-pc-windows-msvc`;
  `wasm32-unknown-unknown` and Android targets are not installed.

## Initial implementation boundary

Keep the existing Rust CLI and library in place. Build a small TypeScript/Vite
frontend and a narrow Rust/WASM adapter so the browser can invoke the existing
CSV validation and backtest engine locally. Add a Tauri shell only after the
portable browser slice works; reuse the same frontend and Rust logic. Android
packaging remains contingent on the missing SDK/NDK toolchain.
