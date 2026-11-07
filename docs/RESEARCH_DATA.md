# Company research data contracts

`src/domain/research.rs` defines validation-only contracts for future sourced
company research:

- Company profile with currency and source metadata.
- Fundamental facts with a named metric, explicit unit, annual/quarterly/TTM
  period, period-end, optional report timestamp, and source metadata.
- News articles with publisher, publication time, HTTPS source link, and
  provenance.
- A bounded company snapshot keyed by exchange-qualified `InstrumentId`.
- `safe_ratio`, which returns `None` for non-finite inputs, zero denominators,
  or a non-finite result.

No company data is embedded, inferred, or fetched. The contracts do not yet
normalize any selected provider's fiscal periods, accounting units, restated
statements, corporate actions, or news licensing. A real provider and its
terms must be verified before exposing company research in the UI. See
[provider and cache status](./PROVIDERS_AND_CACHE.md).
