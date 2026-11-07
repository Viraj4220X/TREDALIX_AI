# Offline deterministic analyst

The Data manager includes an offline analyst for the currently validated
local CSV. It is a small, deterministic rules-and-template pipeline, not a
language model.

## Request and response flow

1. The Web form passes its question, CSV, metadata and feature flags to
   `src-wasm/src/analyst.rs`.
2. The WASM boundary applies the 10 MiB input cap and requires CSV import,
   data validation and local-analyst features to be enabled. It reparses and
   revalidates the CSV before analysis.
3. `src/analyst.rs` assembles bounded evidence and reuses Rust SMA-20,
   Cutler RSI-14 and Wilder ATR-14 calculations. It also calculates the
   close-to-close return and maximum close-based drawdown for the supplied
   range.
4. `src/analyst/intent.rs` routes a bounded English question to a supported
   intent. Deterministic templates answer summaries, return, trend, risk and
   indicator questions. Requests for forecasts, causes, comparisons, company
   facts or trading directives receive an explicit limitation rather than an
   invented answer.
5. The output validator rejects non-finite evidence, invalid response sizes
   and factual claims whose evidence IDs are absent. The UI renders escaped
   response text and presents evidence, source labels, timestamps, data gaps,
   assumptions and uncertainty.

## Data, safety and retention

- The only evidence source is the user-selected CSV. Its source label is
  user-provided, not independently verified; the app marks its class as
  `SYNTHETIC / TEST` or `UNKNOWN / UNVERIFIED`.
- Price currency, provider freshness, corporate-action adjustment basis,
  exchange calendar, company fundamentals and sourced news are unavailable
  unless separately provided by a future verified data service.
- The analyst does not use a remote LLM, send network requests, make trading
  recommendations, or generate forecasts. The separate random-walk baseline
  remains experimental and is not called by this analyst.
- The current question and answer remain in browser memory for the active
  session only. The user can clear the current result. There is no durable
  conversation, user-approved note store, or cross-instrument memory.
- Audit fingerprints use FNV-1a 64-bit. They are non-cryptographic
  correlation identifiers, not secure hashes, privacy controls or
  tamper-evident audit logs. No full dataset or question is written to
  localStorage.
- Intent recognition is keyword-based and English-only. Ambiguous and
  unsupported wording may be routed to a generic dataset summary; it is not
  natural-language understanding.

## Verification

Core tests use synthetic data explicitly labelled `SYNTHETIC / TEST` to verify
return arithmetic, evidence references, Rust indicator reuse, unsupported
forecast/causality boundaries, adversarial requests and input limits. WASM
tests verify revalidation and feature gates. Frontend tests verify safe HTML
escaping. These tests establish software behavior only, not market accuracy.
