#[macro_use]
#[path = "support/assert_row.rs"]
mod assert_row;

use std::str::FromStr;

use libfmp::{
    codecs::{IsoTimestamp, NumericString, YnFlag},
    responses::funds::{
        EtfAssetExposure, EtfCountryWeighting, EtfFundHolding, EtfFundInfo, EtfSectorExposure,
        EtfSectorWeighting, FundDisclosure, FundDisclosureDate, FundDisclosureHolder,
        FundDisclosureSearchResult,
    },
    responses::institutional_ownership::Form13fFilingDate,
    types::{
        ApiDateTime, CalendarQuarter, CalendarYear, Cik, CountryCode, CurrencyCode, Cusip, Date,
        Industry, Isin, Lei, Sector, Ticker,
    },
};
use serde::{Serialize, de::DeserializeOwned};

const HOLDINGS: &[u8] = include_bytes!("fixtures/etf_fund_holdings.json");
const INFO: &[u8] = include_bytes!("fixtures/etf_fund_info.json");
const COUNTRY: &[u8] = include_bytes!("fixtures/etf_country_weightings.json");
const ASSET: &[u8] = include_bytes!("fixtures/etf_asset_exposure.json");
const SECTOR: &[u8] = include_bytes!("fixtures/etf_sector_weightings.json");
const LATEST_HOLDERS: &[u8] = include_bytes!("fixtures/latest_fund_disclosure_holders.json");
const DISCLOSURES: &[u8] = include_bytes!("fixtures/fund_disclosures.json");
const SEARCH: &[u8] = include_bytes!("fixtures/fund_disclosure_holder_search.json");
const DATES: &[u8] = include_bytes!("fixtures/fund_disclosure_dates.json");

#[test]
fn exact_holding_fixture_decodes_all_nine_fields_and_space_timestamp() {
    assert_field_count(HOLDINGS, 9);
    let rows: Vec<EtfFundHolding> = serde_json::from_slice(HOLDINGS).unwrap();
    assert_rows!(
        rows,
        [EtfFundHolding {
            symbol: Ticker::new("SPY").unwrap(),
            asset: Some(Ticker::new("AAPL").unwrap()),
            name: "APPLE INC".to_owned(),
            isin: Some(Isin::new("US0378331005").unwrap()),
            security_cusip: Some(Cusip::new("037833100").unwrap()),
            shares_number: 181_418_073.0,
            weight_percentage: 7.79997012,
            market_value: 61_679_458_958.0,
            updated_at: ApiDateTime::parse("2026-07-30 08:07:21").unwrap(),
        }]
    );
    assert_eq!(
        rows[0].security_cusip.as_ref().unwrap().as_str(),
        "037833100"
    );
}

#[test]
fn exact_info_fixture_decodes_nineteen_fields_and_nested_sectors() {
    assert_field_count(INFO, 19);
    let rows: Vec<EtfFundInfo> = serde_json::from_slice(INFO).unwrap();
    let row = &rows[0];
    assert_eq!(row.symbol, Ticker::new("SPY").unwrap());
    assert_eq!(row.name, "State Street SPDR S&P 500 ETF");
    assert!(row.description.contains("It also can`t reinvest"));
    assert_eq!(row.isin, Isin::new("US78462F1030").unwrap());
    assert_eq!(row.asset_class, "Equity");
    assert_eq!(row.security_cusip, Cusip::new("78462F103").unwrap());
    assert_eq!(row.domicile, CountryCode::new("US").unwrap());
    assert_eq!(row.etf_company, "SPDR");
    assert_eq!(row.expense_ratio, 0.09);
    assert_eq!(row.assets_under_management, 777_349_860_000.0);
    assert_eq!(row.avg_volume, 52_093_933.0);
    assert_eq!(row.inception_date, Date::from_str("1993-01-22").unwrap());
    assert_eq!(row.nav, 729.27);
    assert_eq!(row.nav_currency, CurrencyCode::new("USD").unwrap());
    assert_eq!(row.holdings_count, 504);
    assert!(row.is_actively_trading);
    assert_eq!(
        row.updated_at,
        IsoTimestamp::from_str("2026-07-30T16:00:20.049Z").unwrap()
    );
    assert_rows!(
        row.sectors_list,
        [
            EtfSectorExposure {
                industry: Industry::new("Basic Materials").unwrap(),
                exposure: 1.6916311902850854,
            },
            EtfSectorExposure {
                industry: Industry::new("Cash & Others").unwrap(),
                exposure: 0.30489782336177595,
            },
            EtfSectorExposure {
                industry: Industry::new("Communication Services").unwrap(),
                exposure: 9.23211353485037,
            },
        ]
    );
}

#[test]
fn exact_allocation_fixtures_preserve_string_and_numeric_percent_kinds() {
    assert_field_count(COUNTRY, 2);
    let countries: Vec<EtfCountryWeighting> = serde_json::from_slice(COUNTRY).unwrap();
    assert_eq!(countries[0].country, "United States");
    assert_eq!(countries[0].weight_percentage.as_str(), "97.26%");
    assert_eq!(
        serde_json::to_value(countries).unwrap()[0],
        source_row(COUNTRY)
    );

    assert_field_count(ASSET, 5);
    let assets: Vec<EtfAssetExposure> = serde_json::from_slice(ASSET).unwrap();
    assert_eq!(assets[0].symbol, Ticker::new("ZWT-T.TO").unwrap());
    assert_eq!(assets[0].asset, Ticker::new("AAPL").unwrap());
    assert_eq!(assets[0].shares_number, 42_372.0);
    assert_eq!(assets[0].weight_percentage, 10.1);
    assert_eq!(assets[0].market_value, 20_141_231.66);
    let encoded_assets = serde_json::to_value(assets).unwrap();
    assert_eq!(encoded_assets[0]["symbol"], source_row(ASSET)["symbol"]);
    assert_eq!(encoded_assets[0]["weightPercentage"].as_f64(), Some(10.1));

    assert_field_count(SECTOR, 3);
    let sectors: Vec<EtfSectorWeighting> = serde_json::from_slice(SECTOR).unwrap();
    assert_eq!(sectors[0].symbol, Ticker::new("SPY").unwrap());
    assert_eq!(sectors[0].sector, Sector::new("Basic Materials").unwrap());
    assert_eq!(sectors[0].weight_percentage, 1.6916311902850854);
    assert_eq!(
        serde_json::to_value(sectors).unwrap()[0],
        source_row(SECTOR)
    );

    let mut country: serde_json::Value = serde_json::from_slice(COUNTRY).unwrap();
    country[0]["weightPercentage"] = serde_json::json!(97.26);
    assert!(serde_json::from_value::<Vec<EtfCountryWeighting>>(country).is_err());

    let mut asset: serde_json::Value = serde_json::from_slice(ASSET).unwrap();
    asset[0]["weightPercentage"] = serde_json::json!("10.1%");
    assert!(serde_json::from_value::<Vec<EtfAssetExposure>>(asset).is_err());

    let mut sector: serde_json::Value = serde_json::from_slice(SECTOR).unwrap();
    sector[0]["weightPercentage"] = serde_json::json!("1.69%");
    assert!(serde_json::from_value::<Vec<EtfSectorWeighting>>(sector).is_err());
}

#[test]
fn exact_latest_holder_fixture_preserves_leading_zeroes_and_signed_change() {
    assert_field_count(LATEST_HOLDERS, 7);
    let rows: Vec<FundDisclosureHolder> = serde_json::from_slice(LATEST_HOLDERS).unwrap();
    assert_rows!(
        rows,
        [FundDisclosureHolder {
            cik: Cik::new("0000866256").unwrap(),
            holder: "PARNASSUS INCOME FUNDS".to_owned(),
            security_cusip: Cusip::new("037833100").unwrap(),
            shares: 3_638_451.0,
            date_reported: Date::from_str("2026-06-30").unwrap(),
            change: -316_881.0,
            weight_percent: 4.06607721,
        }]
    );
    assert_eq!(
        serde_json::to_value(rows).unwrap()[0],
        source_row(LATEST_HOLDERS)
    );
}

#[test]
fn exact_disclosure_fixture_decodes_all_twenty_three_wire_fields() {
    assert_field_count(DISCLOSURES, 23);
    let rows: Vec<FundDisclosure> = serde_json::from_slice(DISCLOSURES).unwrap();
    assert_rows!(
        rows,
        [FundDisclosure {
            cik: Cik::new("0000857489").unwrap(),
            date: Date::from_str("2023-10-31").unwrap(),
            accepted_date: ApiDateTime::parse("2023-12-28 09:26:13").unwrap(),
            symbol: Some(Ticker::new("000089.SZ").unwrap()),
            name: "Shenzhen Airport Co Ltd".to_owned(),
            lei: Lei::new("3003009W045RIKRBZI44").unwrap(),
            title: "SHENZ AIRPORT-A".to_owned(),
            cusip: Cusip::new("N/A").unwrap(),
            isin: Some(Isin::new("CNE000000VK1").unwrap()),
            balance: 2_438_784.0,
            units: "NS".to_owned(),
            currency_code: CurrencyCode::new("CNY").unwrap(),
            val_usd: 2_255_873.6,
            pct_val: 0.0023838966190458206,
            payoff_profile: "Long".to_owned(),
            asset_cat: "EC".to_owned(),
            issuer_cat: "CORP".to_owned(),
            inv_country: CountryCode::new("CN").unwrap(),
            is_restricted_sec: YnFlag::False,
            fair_val_level: NumericString::new("2").unwrap(),
            is_cash_collateral: YnFlag::False,
            is_non_cash_collateral: YnFlag::False,
            is_loan_by_fund: YnFlag::False,
        }]
    );
    assert_eq!(rows[0].symbol.as_ref().unwrap().as_str(), "000089.SZ");
    assert_eq!(rows[0].cusip.as_str(), "N/A");
    assert_eq!(rows[0].fair_val_level.as_str(), "2");
    assert_eq!(
        serde_json::to_value(rows).unwrap()[0],
        source_row(DISCLOSURES)
    );
}

#[test]
fn exact_search_fixture_preserves_all_thirteen_string_fields() {
    assert_field_count(SEARCH, 13);
    let rows: Vec<FundDisclosureSearchResult> = serde_json::from_slice(SEARCH).unwrap();
    let row = &rows[0];
    assert_eq!(row.symbol, Ticker::new("FGOAX").unwrap());
    assert_eq!(row.cik.as_str(), "0000355691");
    assert_eq!(row.class_id, "C000024574");
    assert_eq!(row.series_id, "S000009042");
    assert_eq!(
        row.entity_name,
        "Federated Hermes Government Income Securities, Inc."
    );
    assert_eq!(
        row.entity_org_type.as_ref().map(NumericString::as_str),
        Some("30")
    );
    assert_eq!(
        row.series_name,
        "Federated Hermes Government Income Securities, Inc."
    );
    assert_eq!(row.class_name, "Class A Shares");
    assert_eq!(row.reporting_file_number, "811-03266");
    assert_eq!(row.address.as_deref(), Some("4000 ERICSSON DRIVE"));
    assert_eq!(row.city, "WARRENDALE");
    assert_eq!(row.zip_code, "15086-7561");
    assert_eq!(row.state, "PA");
    assert_eq!(serde_json::to_value(rows).unwrap()[0], source_row(SEARCH));
}

#[test]
fn exact_dates_fixture_uses_numeric_calendar_period_units() {
    assert_field_count(DATES, 3);
    let rows: Vec<FundDisclosureDate> = serde_json::from_slice(DATES).unwrap();
    assert_rows!(
        rows,
        [FundDisclosureDate {
            date: Date::from_str("2026-04-30").unwrap(),
            year: CalendarYear(2026),
            quarter: CalendarQuarter::new(2).unwrap(),
        }]
    );
    assert_eq!(serde_json::to_value(rows).unwrap()[0], source_row(DATES));

    let reused: Vec<Form13fFilingDate> = serde_json::from_slice(DATES).unwrap();
    assert_eq!(reused[0].year, CalendarYear(2026));
}

#[test]
fn integer_widths_signed_change_and_decimal_market_values_are_preserved() {
    for field in [
        "sharesNumber",
        "assetsUnderManagement",
        "avgVolume",
        "holdingsCount",
    ] {
        let fixture = if field == "sharesNumber" {
            HOLDINGS
        } else {
            INFO
        };
        let mut value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
        let width = if field == "avgVolume" || field == "assetsUnderManagement" {
            u64::from(u32::MAX) + 1
        } else {
            u64::MAX
        };
        value[0][field] = serde_json::json!(width);
        if field == "sharesNumber" {
            let rows: Vec<EtfFundHolding> = serde_json::from_value(value).unwrap();
            assert_eq!(rows[0].shares_number, u64::MAX as f64);
        } else {
            let rows: Vec<EtfFundInfo> = serde_json::from_value(value).unwrap();
            assert_eq!(serde_json::to_value(rows).unwrap()[0][field], width);
        }
    }

    let mut holder: serde_json::Value = serde_json::from_slice(LATEST_HOLDERS).unwrap();
    holder[0]["shares"] = serde_json::json!(u64::MAX);
    holder[0]["change"] = serde_json::json!(i64::MIN);
    let rows: Vec<FundDisclosureHolder> = serde_json::from_value(holder).unwrap();
    assert_eq!(rows[0].shares, u64::MAX as f64);
    assert_eq!(rows[0].change, i64::MIN as f64);

    let mut disclosure: serde_json::Value = serde_json::from_slice(DISCLOSURES).unwrap();
    disclosure[0]["balance"] = serde_json::json!(u64::MAX);
    disclosure[0]["valUsd"] = serde_json::json!(0.125);
    let rows: Vec<FundDisclosure> = serde_json::from_value(disclosure).unwrap();
    assert_eq!(rows[0].balance, u64::MAX as f64);
    assert_eq!(rows[0].val_usd, 0.125);
}

#[test]
fn identifiers_flags_and_temporal_wire_kinds_are_not_coerced() {
    let mut disclosure: serde_json::Value = serde_json::from_slice(DISCLOSURES).unwrap();
    disclosure[0]["cik"] = serde_json::json!("0000000001");
    disclosure[0]["symbol"] = serde_json::json!("000001.SZ");
    disclosure[0]["cusip"] = serde_json::json!("001234567");
    let rows: Vec<FundDisclosure> = serde_json::from_value(disclosure).unwrap();
    assert_eq!(rows[0].cik.as_str(), "0000000001");
    assert_eq!(rows[0].symbol.as_ref().unwrap().as_str(), "000001.SZ");
    assert_eq!(rows[0].cusip.as_str(), "001234567");

    let mut info: serde_json::Value = serde_json::from_slice(INFO).unwrap();
    info[0]["isActivelyTrading"] = serde_json::json!("Y");
    assert!(serde_json::from_value::<Vec<EtfFundInfo>>(info).is_err());

    for field in [
        "isRestrictedSec",
        "isCashCollateral",
        "isNonCashCollateral",
        "isLoanByFund",
    ] {
        let mut value: serde_json::Value = serde_json::from_slice(DISCLOSURES).unwrap();
        value[0][field] = serde_json::json!(false);
        assert!(serde_json::from_value::<Vec<FundDisclosure>>(value).is_err());
    }

    let mut holding: serde_json::Value = serde_json::from_slice(HOLDINGS).unwrap();
    holding[0]["updatedAt"] = serde_json::json!("2026-07-30T08:07:21.000Z");
    assert!(serde_json::from_value::<Vec<EtfFundHolding>>(holding).is_err());

    let mut info: serde_json::Value = serde_json::from_slice(INFO).unwrap();
    info[0]["updatedAt"] = serde_json::json!("2026-07-30 16:00:20");
    assert!(serde_json::from_value::<Vec<EtfFundInfo>>(info).is_err());

    let mut disclosure: serde_json::Value = serde_json::from_slice(DISCLOSURES).unwrap();
    disclosure[0]["date"] = serde_json::json!("2023-10-31 00:00:00");
    assert!(serde_json::from_value::<Vec<FundDisclosure>>(disclosure).is_err());

    let mut disclosure: serde_json::Value = serde_json::from_slice(DISCLOSURES).unwrap();
    disclosure[0]["acceptedDate"] = serde_json::json!("2023-12-28");
    assert!(serde_json::from_value::<Vec<FundDisclosure>>(disclosure).is_err());
}

#[test]
fn all_fields_and_nested_sector_fields_are_required_non_null_and_forward_tolerant() {
    assert_contract::<EtfFundHolding>(HOLDINGS, &["asset", "isin", "securityCusip"]);
    assert_contract::<EtfFundInfo>(INFO, &[]);
    assert_contract::<EtfCountryWeighting>(COUNTRY, &[]);
    assert_contract::<EtfAssetExposure>(ASSET, &[]);
    assert_contract::<EtfSectorWeighting>(SECTOR, &[]);
    assert_contract::<FundDisclosureHolder>(LATEST_HOLDERS, &[]);
    assert_contract::<FundDisclosure>(DISCLOSURES, &["symbol", "isin"]);
    assert_contract::<FundDisclosureSearchResult>(SEARCH, &["entityOrgType", "address"]);
    assert_contract::<FundDisclosureDate>(DATES, &[]);

    for field in ["industry", "exposure"] {
        let mut missing: serde_json::Value = serde_json::from_slice(INFO).unwrap();
        missing[0]["sectorsList"][0]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(serde_json::from_value::<Vec<EtfFundInfo>>(missing).is_err());

        let mut null: serde_json::Value = serde_json::from_slice(INFO).unwrap();
        null[0]["sectorsList"][0][field] = serde_json::Value::Null;
        assert!(serde_json::from_value::<Vec<EtfFundInfo>>(null).is_err());
    }
    let mut forward: serde_json::Value = serde_json::from_slice(INFO).unwrap();
    forward[0]["sectorsList"][0]["futureProviderField"] = serde_json::json!([1, true]);
    assert_eq!(
        serde_json::from_value::<Vec<EtfFundInfo>>(forward)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn all_nine_contracts_are_bare_arrays_preserving_empty_and_multiple_rows() {
    assert_bare_array::<EtfFundHolding>(HOLDINGS);
    assert_bare_array::<EtfFundInfo>(INFO);
    assert_bare_array::<EtfCountryWeighting>(COUNTRY);
    assert_bare_array::<EtfAssetExposure>(ASSET);
    assert_bare_array::<EtfSectorWeighting>(SECTOR);
    assert_bare_array::<FundDisclosureHolder>(LATEST_HOLDERS);
    assert_bare_array::<FundDisclosure>(DISCLOSURES);
    assert_bare_array::<FundDisclosureSearchResult>(SEARCH);
    assert_bare_array::<FundDisclosureDate>(DATES);
    assert!(
        serde_json::from_value::<Vec<FundDisclosure>>(serde_json::json!({ "data": [] })).is_err()
    );
}

#[test]
fn omittable_members_decode_null_and_empty_as_none() {
    for (field, wire) in [
        ("asset", serde_json::Value::Null),
        ("asset", serde_json::json!("")),
        ("isin", serde_json::Value::Null),
        ("isin", serde_json::json!("")),
        ("securityCusip", serde_json::Value::Null),
        ("securityCusip", serde_json::json!("")),
    ] {
        let mut value: serde_json::Value = serde_json::from_slice(HOLDINGS).unwrap();
        value[0][field] = wire;
        let rows: Vec<EtfFundHolding> = serde_json::from_value(value).unwrap();
        let decoded = match field {
            "asset" => rows[0].asset.is_none(),
            "isin" => rows[0].isin.is_none(),
            _ => rows[0].security_cusip.is_none(),
        };
        assert!(decoded, "{field}");
        assert!(serde_json::to_value(&rows).unwrap()[0][field].is_null());
    }

    for wire in [serde_json::Value::Null, serde_json::json!("")] {
        let mut value: serde_json::Value = serde_json::from_slice(DISCLOSURES).unwrap();
        value[0]["isin"] = wire;
        let rows: Vec<FundDisclosure> = serde_json::from_value(value).unwrap();
        assert_eq!(rows[0].isin, None);
    }
    let mut disclosure: serde_json::Value = serde_json::from_slice(DISCLOSURES).unwrap();
    disclosure[0]["symbol"] = serde_json::Value::Null;
    let rows: Vec<FundDisclosure> = serde_json::from_value(disclosure).unwrap();
    assert_eq!(rows[0].symbol, None);
    assert!(serde_json::to_value(&rows).unwrap()[0]["symbol"].is_null());
    let mut disclosure: serde_json::Value = serde_json::from_slice(DISCLOSURES).unwrap();
    disclosure[0]["symbol"] = serde_json::json!("");
    assert!(serde_json::from_value::<Vec<FundDisclosure>>(disclosure).is_err());

    let mut search: serde_json::Value = serde_json::from_slice(SEARCH).unwrap();
    search[0]["address"] = serde_json::Value::Null;
    let rows: Vec<FundDisclosureSearchResult> = serde_json::from_value(search).unwrap();
    assert_eq!(rows[0].address, None);
    let mut search: serde_json::Value = serde_json::from_slice(SEARCH).unwrap();
    search[0]["address"] = serde_json::json!("");
    let rows: Vec<FundDisclosureSearchResult> = serde_json::from_value(search).unwrap();
    assert_eq!(rows[0].address.as_deref(), Some(""));

    let mut search: serde_json::Value = serde_json::from_slice(SEARCH).unwrap();
    search[0]["entityOrgType"] = serde_json::json!("NULL");
    let rows: Vec<FundDisclosureSearchResult> = serde_json::from_value(search).unwrap();
    assert_eq!(rows[0].entity_org_type, None);
    assert!(serde_json::to_value(&rows).unwrap()[0]["entityOrgType"].is_null());
    for wire in ["SECRET", "null", ""] {
        let mut search: serde_json::Value = serde_json::from_slice(SEARCH).unwrap();
        search[0]["entityOrgType"] = serde_json::json!(wire);
        let error = serde_json::from_value::<Vec<FundDisclosureSearchResult>>(search)
            .unwrap_err()
            .to_string();
        assert!(!error.contains("SECRET"), "{error}");
    }
}

fn source_row(fixture: &[u8]) -> serde_json::Value {
    serde_json::from_slice::<serde_json::Value>(fixture).unwrap()[0].clone()
}

fn assert_contract<T: DeserializeOwned>(fixture: &[u8], nullable: &[&str]) {
    let source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    let keys = source[0]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<_>>();

    for key in keys {
        let mut missing = source.clone();
        missing[0].as_object_mut().unwrap().remove(&key);
        assert!(
            serde_json::from_value::<Vec<T>>(missing).is_err(),
            "accepted missing {key}"
        );

        let mut null = source.clone();
        null[0][&key] = serde_json::Value::Null;
        assert_eq!(
            serde_json::from_value::<Vec<T>>(null).is_ok(),
            nullable.contains(&key.as_str()),
            "null {key}"
        );
    }

    let mut forward = source;
    forward[0]["futureProviderField"] = serde_json::json!({ "nested": [1, true, null] });
    assert_eq!(serde_json::from_value::<Vec<T>>(forward).unwrap().len(), 1);
}

fn assert_bare_array<T: DeserializeOwned + Serialize>(fixture: &[u8]) {
    assert!(serde_json::from_slice::<Vec<T>>(b"[]").unwrap().is_empty());
    let mut source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    let duplicate = source[0].clone();
    source.as_array_mut().unwrap().push(duplicate);
    assert_eq!(serde_json::from_value::<Vec<T>>(source).unwrap().len(), 2);
}

fn assert_field_count(fixture: &[u8], expected: usize) {
    let value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), expected);
}
