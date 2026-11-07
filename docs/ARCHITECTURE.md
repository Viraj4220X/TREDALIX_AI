# Application architecture

Status: initial implementation decision, 25 October 2026.

## Goals

- Preserve the existing Rust CLI and its public library interfaces.
- Reuse the Rust CSV validation and backtest engine rather than reimplementing
  financial logic in TypeScript.
- Share one responsive interface across the browser and Tauri desktop shell.
- Keep basic local workflows offline; add no accounts, telemetry, live feed, or
  broker connectivity.

## Chosen shape

```text
frontend/                 TypeScript + Vite UI, browser file input/downloads
src-wasm/                 thin WASM boundary calling the root Rust library
src/domain/               shared identity, data, feature, chart, forecast, and service contracts
frontend/src-tauri/       thin Tauri v2 Windows desktop shell
src/                      existing Rust core and CLI (source of truth)
docs/                     architecture, setup, user, and test guidance
```

The first UI uses plain TypeScript modules rather than a component framework.
The current application is small, and this avoids adding a runtime framework
before the screen and state requirements justify one. Vite provides the
development server and static Web build.

The browser imports a user-selected CSV with the File API and passes its
contents to the Rust/WASM adapter. The adapter calls the existing
`parse_csv`, `validate_dataset`, and `run_backtest` functions, returning
structured data for the interface. Browser downloads remain browser-managed;
the Web build will not access arbitrary local paths.

The same frontend runs inside the Tauri v2 WebView. Tauri-specific IPC and
external-link opening stay behind a separate adapter. Android is a later
packaging target, not a claim of current support.

## Compatibility and verification risks

- Node `v26.6.0` exceeds Vite's documented minimum (Node `20.19+` or `22.12+`).
- Existing Rust modules use `std::fs` in report/CLI code. The WASM target must
  be tested; only the CSV and simulation call paths should be included if the
  full library does not compile for `wasm32-unknown-unknown`.
- Rust/WASM bindings require a compatible target and binding generator.
- Tauri v2 Windows builds need compatible Windows native build tools and
  WebView2.
- Android builds need Android Studio/SDK, NDK, build-tools, environment
  configuration, and Rust Android targets. These were not verified.
- WebView file pickers, local settings, and report download behavior need
  platform-specific testing.

## State, feature gates, and safety

Settings are device-local and versioned. Theme and feature preferences may be
persisted locally. Feature implementation status is separate from a user's
enabled preference. An unavailable feature cannot be activated; disabling
backtesting must prevent a backtest request, not only hide its button.

The initial available features are limited to CSV selection/validation and the
existing offline backtest. Features without verified implementations—live
market data, paper trading, broker access, prediction models, cloud sync, and
notifications—must remain unavailable. Existing costs and limitations are
shown as the Rust engine reports them.

## Planned frontend/backend contract

Use narrow typed requests for CSV inspection and backtests. Validate metadata
and numeric configuration at the Rust boundary. Return structured result data
and explicit errors; never send arbitrary filesystem paths or shell commands
from the browser. The backend remains responsible for all financial results.

## Current status

The browser vertical slice is implemented and production-built: the same Rust
data and simulation library is called through `src-wasm/`; the TypeScript UI
does not implement a second financial engine. Browser CSV input remains in
memory, settings and run summaries use browser-local storage, and report
exports are browser downloads. Vite is pinned to 8.3.5 after the initial
dependency audit identified advisories in 8.2.0.

Only Web is currently listed as a supported platform in the feature registry.
The Windows Tauri shell and release executable compile and open the app
window; NSIS packaging succeeds. Installer installation/uninstallation and
full IPC/UI workflow checks are pending. Android requires a configured
SDK/NDK and target device or emulator and has not been built.

The AI Control Center reuses the existing Rust/WASM analyst and forecast
contracts. Its tool registry only exposes local app-state inspection and
screen navigation. Session tasks are bounded and in-memory; there is no
remote model, provider client, vision endpoint, arbitrary command capability,
or cancellable background worker. Image selection produces a validated local
preview only.

See [APP_BASELINE_AUDIT.md](./APP_BASELINE_AUDIT.md) for the original verified
baseline and [APP_BUILD_PROGRESS.md](./APP_BUILD_PROGRESS.md) for current stage
status.

The Rust `src/domain/` contracts are documented in
[DOMAIN_CONTRACTS.md](./DOMAIN_CONTRACTS.md). They provide typed boundaries,
not external provider integrations or a prediction implementation.
Reusable causal indicator calculations live in `src/indicators/` and are
documented in [INDICATORS.md](./INDICATORS.md). The existing backtest entry
point revalidates public dataset input before strategy execution.
