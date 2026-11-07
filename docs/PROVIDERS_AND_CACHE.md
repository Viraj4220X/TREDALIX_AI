# Provider contracts and offline candle cache

## Provider contracts

`src/providers.rs` defines typed provider errors, provider capability
metadata, and interfaces for instrument search, quotes, historical candles,
and company research. The contracts distinguish retryable
rate-limit/timeout/unavailable errors from authentication, invalid-response,
unsupported-instrument, unsupported-interval, and terms-restriction failures.

No real provider is registered or called. Official Twelve Data pages were
reviewed on 2026-10-25 as a candidate:

- [API documentation](https://twelvedata.com/docs) lists symbol search,
  historical series, profile, financial statements, statistics, and press
  releases.
- [Individual pricing](https://twelvedata.com/pricing) describes individual
  plans as personal/internal/non-commercial use, lists plan-based quotas and
  market access, and identifies fundamentals and display access as plan
  dependent. The reviewed page displayed Grow at $79/month or $790/year;
  prices can change and must be confirmed by the account holder.
- [Exchange coverage by Basic plan](https://twelvedata.com/exchanges?level=basic)
  lists three exchanges; [Grow](https://twelvedata.com/exchanges?level=grow)
  lists Indian exchanges including XNSE/XBOM as EOD market data.
- [Symbol Search docs](https://twelvedata.com/docs/discovery/symbol-search),
  [time-series docs](https://twelvedata.com/docs/market-data/time-series),
  [profile docs](https://twelvedata.com/docs/fundamentals/profile),
  [income statement docs](https://twelvedata.com/docs/fundamentals/income-statement),
  and [press release docs](https://twelvedata.com/docs/fundamentals/press-releases)
  describe endpoints but do not, by themselves, grant data redistribution
  rights.

This is a candidate assessment, not a provider selection or legal review.
Plan pricing and market lists are dynamic. The pages do not establish the
user's subscription entitlement, contractual retention/redistribution rights,
or a suitable public-app license. The Basic directory lists three exchanges;
the reviewed Grow tier adds India exchanges as EOD access. The exchange
directory identifies a separate higher business-plan minimum. Indian equity
EOD access is not live streaming. Provider-specific retention/cache,
redistribution, and end-user display terms still need contract-level review.
A secure server/native path is also needed because a private API key must
never enter the Web bundle. No key, subscription, API request, or real market
response has been supplied or tested, so provider integration remains blocked.

### Chart dependency and attribution

The Web chart uses `lightweight-charts` 5.2.1, whose package metadata and
upstream [LICENSE](https://raw.githubusercontent.com/tradingview/lightweight-charts/master/LICENSE)
identify Apache-2.0. Its upstream [documentation](https://tradingview.github.io/lightweight-charts/docs)
requires TradingView attribution and a TradingView link. The data page
displays that notice and link. The library renders only validated local CSV
data; it is not a source of market data.

## Bounded in-memory cache

`CandleCache` is a platform-neutral, bounded FIFO cache for normalized candle
responses. Keys include instrument, interval, UTC range, adjustment mode, and
source policy. Construction requires positive entry, candle, and TTL limits.
Insertion validates candle provenance/shape/range and requires explicit
permission to cache under provider terms. Reads return cached or stale
freshness labels; callers must opt in to stale reads.

This component is memory-only. It does not persist data, handle interrupted
disk writes, schema migrations, or source-specific licensing automatically.
Those require a selected provider and platform storage adapter. It must not
be treated as a live-data provider or company-research service.
