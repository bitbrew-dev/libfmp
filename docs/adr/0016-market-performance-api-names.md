# ADR 0016: Market performance API names and contract boundaries

## Status

Accepted for issue #25's Rust core implementation and reserved future Python
facade.

## Decision

The eleven Market Performance endpoints use the following final Rust
descriptor, client, query, and response-row names and reserve the listed future
Python names.

| FMP path | Rust descriptor/client method | Rust query | Rust response row | Future Python API |
| --- | --- | --- | --- | --- |
| `sector-performance-snapshot` | `sector_performance_snapshot` | `SectorPerformanceSnapshotQuery` | `SectorPerformance` | `FmpClient.sector_performance_snapshot`; `fmp.market.SectorPerformance` |
| `industry-performance-snapshot` | `industry_performance_snapshot` | `IndustryPerformanceSnapshotQuery` | `IndustryPerformance` | `FmpClient.industry_performance_snapshot`; `fmp.market.IndustryPerformance` |
| `historical-sector-performance` | `historical_sector_performance` | `HistoricalSectorPerformanceQuery` | `SectorPerformance` | `FmpClient.historical_sector_performance`; `fmp.market.SectorPerformance` |
| `historical-industry-performance` | `historical_industry_performance` | `HistoricalIndustryPerformanceQuery` | `IndustryPerformance` | `FmpClient.historical_industry_performance`; `fmp.market.IndustryPerformance` |
| `sector-pe-snapshot` | `sector_pe_snapshot` | `SectorPeSnapshotQuery` | `SectorPe` | `FmpClient.sector_pe_snapshot`; `fmp.market.SectorPe` |
| `industry-pe-snapshot` | `industry_pe_snapshot` | `IndustryPeSnapshotQuery` | `IndustryPe` | `FmpClient.industry_pe_snapshot`; `fmp.market.IndustryPe` |
| `historical-sector-pe` | `historical_sector_pe` | `HistoricalSectorPeQuery` | `SectorPe` | `FmpClient.historical_sector_pe`; `fmp.market.SectorPe` |
| `historical-industry-pe` | `historical_industry_pe` | `HistoricalIndustryPeQuery` | `IndustryPe` | `FmpClient.historical_industry_pe`; `fmp.market.IndustryPe` |
| `biggest-gainers` | `biggest_gainers` | unit (`()`) | `MarketMover` | `FmpClient.biggest_gainers`; `fmp.market.MarketMover` |
| `biggest-losers` | `biggest_losers` | unit (`()`) | `MarketMover` | `FmpClient.biggest_losers`; `fmp.market.MarketMover` |
| `most-actives` | `most_actives` | unit (`()`) | `MarketMover` | `FmpClient.most_actives`; `fmp.market.MarketMover` |

All eleven endpoints are `GET` requests returning bare arrays. Documented
fields are required and non-null, while unknown fields remain accepted for
forward compatibility. The snapshot and historical variants intentionally
share response rows because their documented wire shapes are identical.

Snapshot queries require `date` and independently accept optional `exchange`
and sector or industry filters. Their exact wire order is `date`, `exchange`,
then `sector` or `industry`. Sector history queries require `sector` and emit
`from`, `exchange`, `sector`, then `to`. Industry history queries require
`industry` and emit `industry`, `exchange`, `from`, then `to`. Optional `from`
and `to` values remain independent; the SDK does not invent a range or impose
undocumented validation. These endpoints document no query or response caps.

Dates reuse the strict `Date` fundamental. Sector, industry, and exchange
values reuse their open representation-preserving fundamentals. Average
changes and mover percentage changes use `Percentage`; mover absolute changes
use `Change`; prices use `Price`; and P/E ratios remain raw `f64`. These aliases
accept signed, zero, integer, and fractional JSON numbers where Serde permits a
finite `f64` value.

The three US-only mover endpoints use the same six-field `MarketMover` row.
The row contains `symbol`, `price`, `name`, `change`, `changesPercentage`, and
`exchange`; it does not add the volume discussed in descriptive prose because
volume is absent from all three documented response examples. Movers use the
unit query because no query parameters are documented.

Python runtime bindings are deferred. The future facade reserves ordinary
`FmpClient` methods and the five response classes under `fmp.market`; Rust
query structs will not become Python public classes.

This foundation defines the eight endpoint-owned queries and five shared
response rows. Rust descriptors and client methods are implemented by the
following stacked changes.
