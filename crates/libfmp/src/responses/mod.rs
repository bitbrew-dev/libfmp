//! Typed models for FMP API responses.
//!
//! Every public response struct is `#[non_exhaustive]` so a field the provider
//! starts sending is a minor release: build rows by deserializing JSON, not with
//! struct literals (ADR 0031, `docs/adr/0031-pre-1.0-contract-decisions.md`).

pub mod analyst;
pub mod bulk;
pub mod calendar;
pub mod chart;
pub mod commitment_of_traders;
pub mod commodities;
pub mod company;
pub mod congressional;
pub mod crypto;
pub mod dcf;
pub mod directory;
pub mod economics;
pub mod esg;
pub mod forex;
pub mod fundraising;
pub mod funds;
pub mod indexes;
pub mod insider_trading;
pub mod institutional_ownership;
pub mod market;
pub mod market_hours;
pub mod news;
pub mod quote;
pub mod screener;
pub mod search;
pub mod sec_filings;
pub mod statements;
pub mod technical_indicators;
pub mod tipranks;
pub mod transcripts;
