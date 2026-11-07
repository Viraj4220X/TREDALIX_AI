# Web application user guide

## Start the app

Follow [INSTALLATION.md](INSTALLATION.md), then run
`npm.cmd run dev` from `frontend\`. The app works with local files and the
bundled Rust/WASM engine; no account or broker connection is used.

## Import and validate a CSV

1. Open **Data manager** and select a `.csv` file (maximum 10 MiB).
2. Enter the instrument, instrument type, interval in seconds, and a source
   description. These are user-provided labels, not verified provenance.
3. Mark the synthetic/test-data checkbox for generated or sample data.
4. Select **Validate selected CSV**.

The file is read into memory for the current browser session. The Rust core
validates timestamp order, OHLC values, and optional volume. Required columns
are `timestamp,open,high,low,close`; `volume` is optional. Invalid rows are
reported rather than silently skipped.

The validated data page includes an interactive candlestick/volume view.
Rust computes the SMA-20, EMA-20, Bollinger bands, RSI-14, MACD, and ATR-14 series;
warm-up portions are omitted rather than filled with guessed values. Use the
checkboxes to toggle overlays and mouse/touch controls to pan or zoom. For
large files, the chart bounds its payload by aggregating OHLC buckets while
preserving each bucket's open, high, low, close, and available volume. This is
only a view of the selected CSV, not an exchange feed.

## Ask the offline analyst

After validating a CSV, use **Ask about this dataset** on the Data manager
page. It can summarize the imported rows, calculate the close-to-close return
and maximum close-based drawdown, compare the last close with SMA-20, and
explain RSI-14/ATR-14 when enough bars are available. Numeric analysis reuses
the Rust engine. Each result displays its evidence, user-provided source label,
as-of timestamp, assumptions, limitations, and an audit fingerprint.

This deterministic feature does not use an LLM, internet, external provider,
company fundamentals, or news. It cannot establish why a price moved, compare
two instruments, or generate a trained forecast. A request for future targets
is answered with that limitation; forecasts remain separate and experimental.
The question and current answer stay in browser memory only and can be cleared
with **Clear this analysis**; they are not saved to local storage. The analyst
is English-only, and its FNV-1a fingerprints are for correlation, not security
or tamper detection.

## Run a backtest

1. Validate a dataset first.
2. Open **Backtesting** and review capital, generic fee/slippage estimates,
   quantity/exposure limits, and the trailing test fraction.
3. Choose whether to include the cash-equity benchmark only when the input is
   explicitly classified as a cash equity.
4. Select **Run offline backtest**.

The existing Rust SMA-9/SMA-21/RSI-14 strategy is used. Decisions use data
through a candle close and simulated market orders execute at the next candle
open. Results, fees, slippage, drawdown, trades, and equity points are actual
engine output. The app sends no live orders.

The backtest evaluation is a single chronological trailing holdout. On the
validated data page, the optional **Baseline forecast & evaluation** panel
can separately run an experimental last-close random-walk estimate and
expanding evaluation against historical drift. Enter and verify the exchange
and currency rather than inferring them from the ticker; choose comma-separated
evaluation horizons such as `1,3,5` bars. Each horizon uses only data available
at its forecast origin. This is not a trained model, uncertainty calibration,
or backtest. Generic fees are not a complete India-specific statutory charge
model. Exchange sessions/holidays are not configured. A synthetic test result
is not market evidence and no result guarantees future performance.

## Reports and settings

After a run, open **Reports** to download the current run as JSON or CSV
reports. Only actual engine results are exported. The browser stores settings
and compact run summaries in local storage. Imported CSV contents are held in
memory and are not retained after the browser page is reloaded.

Feature switches can disable available operations. Planned or unavailable
features cannot be enabled. Theme and settings can be changed in **Settings**.

Open **AI** beside the theme control for the local AI Control Center. In the
Windows desktop app, verify and load the separately downloaded Gemma 3 4B
model to enable local chat and image understanding; the model is not bundled
in the installer. The app uses CPU inference, one request at a time, and
requires at least 6 GiB of currently available RAM to load. Browser mode
does not start the native model.

The workspace also retains the deterministic Rust analyst for a validated
CSV and the educational baseline scenario/walk-forward calculation. It lists
bounded local calculation tools and shows their schema, method, and result.
Neither Gemma answers nor tool results are independently cross-verified or
confirmed financial results. Image and screen observations can be inaccurate
and are not structured market data. Chat is session-only and is cleared by
closing the app or using **Clear chat**; it is not saved to local storage.

## Not available

The current Web build has no live feed, provider-backed search, company
research source, paper-trading lifecycle, broker execution, trained prediction
model, calibrated uncertainty, saved chart drawings, cloud sync, or exchange
calendar. A Windows desktop preview shell is implemented; its installer and
native UI/IPC workflows still need install and interaction testing. Android is
not implemented.
