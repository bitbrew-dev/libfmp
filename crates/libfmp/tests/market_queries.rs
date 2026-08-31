use std::str::FromStr;

use libfmp::{
    endpoints::market::{
        HistoricalIndustryPeQuery, HistoricalIndustryPerformanceQuery, HistoricalSectorPeQuery,
        HistoricalSectorPerformanceQuery, IndustryPeSnapshotQuery,
        IndustryPerformanceSnapshotQuery, SectorPeSnapshotQuery, SectorPerformanceSnapshotQuery,
    },
    types::{Date, ExchangeCode, Industry, Sector},
};

#[test]
fn all_four_snapshot_queries_preserve_required_date_and_optional_open_filters() {
    let date = Date::from_str("2024-02-01").unwrap();
    let exchange = ExchangeCode::new("NEW / EXCHANGE").unwrap();
    let sector = Sector::new("Future & Energy").unwrap();
    let industry = Industry::new("Quantum / Services").unwrap();

    let sector_performance = SectorPerformanceSnapshotQuery::new(date)
        .with_exchange(exchange.clone())
        .with_sector(sector.clone());
    assert_eq!(sector_performance.date(), date);
    assert_eq!(sector_performance.exchange(), Some(&exchange));
    assert_eq!(sector_performance.sector(), Some(&sector));

    let industry_performance = IndustryPerformanceSnapshotQuery::new(date)
        .with_exchange(exchange.clone())
        .with_industry(industry.clone());
    assert_eq!(industry_performance.industry(), Some(&industry));

    let sector_pe = SectorPeSnapshotQuery::new(date).with_sector(sector.clone());
    assert_eq!(sector_pe.exchange(), None);
    assert_eq!(sector_pe.sector(), Some(&sector));

    let industry_pe = IndustryPeSnapshotQuery::new(date).with_industry(industry.clone());
    assert_eq!(industry_pe.exchange(), None);
    assert_eq!(industry_pe.industry(), Some(&industry));
}

#[test]
fn all_four_historical_queries_preserve_independent_optional_dates() {
    let from = Date::from_str("2024-02-01").unwrap();
    let to = Date::from_str("2024-03-01").unwrap();
    let exchange = ExchangeCode::new("NASDAQ").unwrap();
    let sector = Sector::new("Energy").unwrap();
    let industry = Industry::new("Biotechnology").unwrap();

    let sector_performance = HistoricalSectorPerformanceQuery::new(sector.clone())
        .with_from(from)
        .with_exchange(exchange.clone());
    assert_eq!(sector_performance.from(), Some(from));
    assert_eq!(sector_performance.to(), None);
    assert_eq!(sector_performance.exchange(), Some(&exchange));
    assert_eq!(sector_performance.sector(), &sector);

    let sector_pe = HistoricalSectorPeQuery::new(sector.clone()).with_to(to);
    assert_eq!(sector_pe.from(), None);
    assert_eq!(sector_pe.to(), Some(to));
    assert_eq!(sector_pe.sector(), &sector);

    let industry_performance = HistoricalIndustryPerformanceQuery::new(industry.clone())
        .with_exchange(exchange.clone())
        .with_to(to);
    assert_eq!(industry_performance.from(), None);
    assert_eq!(industry_performance.to(), Some(to));
    assert_eq!(industry_performance.exchange(), Some(&exchange));
    assert_eq!(industry_performance.industry(), &industry);

    let industry_pe = HistoricalIndustryPeQuery::new(industry.clone()).with_from(from);
    assert_eq!(industry_pe.from(), Some(from));
    assert_eq!(industry_pe.to(), None);
    assert_eq!(industry_pe.industry(), &industry);
}

#[test]
fn owned_and_borrowed_required_filter_conversions_preserve_representation() {
    let sector = Sector::new("  Future / Sector  ").unwrap();
    let owned: HistoricalSectorPerformanceQuery = sector.clone().into();
    let borrowed: HistoricalSectorPeQuery = (&sector).into();
    assert_eq!(owned.sector().as_str(), "  Future / Sector  ");
    assert_eq!(borrowed.sector().as_str(), "  Future / Sector  ");

    let industry = Industry::new("  Future & Industry  ").unwrap();
    let owned: HistoricalIndustryPerformanceQuery = industry.clone().into();
    let borrowed: HistoricalIndustryPeQuery = (&industry).into();
    assert_eq!(owned.industry().as_str(), "  Future & Industry  ");
    assert_eq!(borrowed.industry().as_str(), "  Future & Industry  ");
}
