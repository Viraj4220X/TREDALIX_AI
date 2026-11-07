# TREDALIX AI

TREDALIX AI is an offline-first market-data inspection and backtesting
project. The existing Rust CLI/library remains the source of truth for CSV
validation and financial calculations. The Web interface calls that engine
through WebAssembly; it does not reimplement backtesting in TypeScript.

## Current status

- Web app: implemented and production-built. It can inspect a local CSV,
  run the existing Rust strategy/backtest, show actual results, persist
  settings and run summaries locally, export reports, and run an experimental
  naive forecast plus configurable multi-horizon walk-forward comparisons.
  The data page has an interactive local OHLCV candlestick chart with Rust
  indicator overlays and an offline deterministic analyst for validated CSV
  questions. The visible AI Control Center shows actual local capabilities,
  bounded session task history, evidence-backed questions, baseline scenario
  analysis, and an honest unavailable state for providers/LLM/vision. The
  analyst is not an LLM and does not use network data.
- Windows desktop: a Tauri shell is implemented, and its release executable
  launches successfully, and the NSIS setup executable builds. Installer
  installation/uninstallation and full native workflow checks remain.
- Android: not implemented. There is no APK.
- Live market data/provider search, sourced company research, LLM integration,
  saved chart annotations, paper trading, broker execution, and trained
  prediction models are not available. Multi-horizon baseline metrics are not
  trained-model calibration.

The included market samples are synthetic or unverified. Backtests are
simulations and do not guarantee future results.

## Run the Web app

See [docs/INSTALLATION.md](docs/INSTALLATION.md) for tool prerequisites.
For the Windows desktop preview, see [docs/WINDOWS_DESKTOP.md](docs/WINDOWS_DESKTOP.md).

```powershell
cd frontend
npm.cmd ci
npm.cmd run dev
```

The development command builds the Rust/WASM adapter before starting Vite.
For a production build and local preview:

```powershell
npm.cmd run build
npm.cmd run preview -- --host 127.0.0.1
```

The preview URL is printed by Vite. It serves the static Web build locally.

## Run the Rust CLI

From the repository root:

```powershell
cargo run -- inspect TEST data\sample.csv --interval-seconds 300 --source "local test fixture" --synthetic
```

For supported commands and simulation assumptions, see
[docs/BACKTESTING.md](docs/BACKTESTING.md) and
[docs/DATA_FORMAT.md](docs/DATA_FORMAT.md).

## Documentation

- [Installation](docs/INSTALLATION.md)
- [Windows desktop](docs/WINDOWS_DESKTOP.md)
- [AI Control Center](docs/WINDOWS_DESKTOP.md#ai-control-center)
- [User guide](docs/USER_GUIDE.md)
- [Testing](docs/TESTING.md)
- [Feature status](docs/FEATURE_STATUS.md)
- [Offline analyst](docs/LOCAL_ANALYST.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Shared domain contracts](docs/DOMAIN_CONTRACTS.md)
- [Rust indicators](docs/INDICATORS.md)
- [Hard-parts specification gap report](docs/HARD_PARTS_GAP_REPORT.md)
- [Build progress](docs/APP_BUILD_PROGRESS.md)
- [Baseline audit](docs/APP_BASELINE_AUDIT.md)
