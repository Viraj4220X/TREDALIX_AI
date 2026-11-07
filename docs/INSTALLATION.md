# Installation and development setup

These steps apply to the Web application and Rust CLI. Windows desktop
preview build steps are in [WINDOWS_DESKTOP.md](WINDOWS_DESKTOP.md). Android
packaging is not implemented.

## Required tools

- Rust toolchain and Cargo.
- Rust target `wasm32-unknown-unknown`.
- `wasm-pack` 0.15.0 or a compatible release.
- Node.js 20.19+ or 22.12+; the verified build used Node 26.6.0.
- npm; PowerShell on this machine blocks `npm.ps1`, so use `npm.cmd`.

The checked build used Rust/Cargo 1.99.1, Node 26.6.0, npm 11.18.0,
Vite 8.3.5, and TypeScript 6.0.3.

## First-time setup on Windows

From the project root in PowerShell:

```powershell
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --version 0.15.0
cd frontend
npm.cmd ci
```

`npm.cmd ci` installs exact versions from the committed lockfile. The browser
workflow itself uses only local assets and does not require market data
credentials or a cloud account.

## Start development server

From `frontend\`:

```powershell
npm.cmd run dev
```

The script builds the Rust/WASM package and starts Vite bound to
`127.0.0.1`. Open the URL printed by Vite.

## Build and preview the production Web app

From `frontend\`:

```powershell
npm.cmd run build
npm.cmd run preview -- --host 127.0.0.1
```

The static output is written to `frontend\dist\`. The preview URL is printed
by Vite. No public deployment is performed by these commands.

## Existing Rust CLI

From the project root:

```powershell
cargo run -- inspect TEST data\sample.csv --interval-seconds 300 --source "local test fixture" --synthetic
cargo run -- backtest TEST data\sample.csv --interval-seconds 300 --source "local test fixture" --synthetic
```

See [DATA_FORMAT.md](DATA_FORMAT.md) and [BACKTESTING.md](BACKTESTING.md)
for input and execution assumptions.

## Android

Android packaging is not implemented. It requires Android Studio/SDK, Android
NDK, configured environment variables, Rust Android targets, and a device or
emulator. Review native toolchain setup before installing.
