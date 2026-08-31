use libfmp::{
    codecs::{DynamicObject, NumericString},
    responses::sec_filings::{
        SecCompanyProfile, SecCompanySearchResult, SecFiling, SicClassification,
    },
    types::{
        ApiDateTime, Cik, CountryCode, CurrencyCode, Date, ExchangeCode, FormType, Isin, Sector,
        Ticker,
    },
};
use serde::{Serialize, de::DeserializeOwned};

const LATEST_8K: &[u8] = include_bytes!("fixtures/latest_8k_sec_filings.json");
const LATEST: &[u8] = include_bytes!("fixtures/latest_sec_filings.json");
const BY_FORM: &[u8] = include_bytes!("fixtures/sec_filings_by_form_type.json");
const BY_SYMBOL: &[u8] = include_bytes!("fixtures/sec_filings_by_symbol.json");
const BY_CIK: &[u8] = include_bytes!("fixtures/sec_filings_by_cik.json");
const COMPANIES_NAME: &[u8] = include_bytes!("fixtures/sec_companies_by_name.json");
const COMPANIES_SYMBOL: &[u8] = include_bytes!("fixtures/sec_companies_by_symbol.json");
const COMPANIES_CIK: &[u8] = include_bytes!("fixtures/sec_companies_by_cik.json");
const PROFILE: &[u8] = include_bytes!("fixtures/sec_company_profile.json");
const CLASSIFICATIONS: &[u8] = include_bytes!("fixtures/industry_classifications.json");
const CLASSIFICATION_SEARCH: &[u8] = include_bytes!("fixtures/industry_classification_search.json");
const ALL_CLASSIFICATIONS: &[u8] = include_bytes!("fixtures/all_industry_classifications.json");

#[test]
fn all_five_filing_routes_share_the_exact_eight_field_model() {
    assert_field_count(LATEST_8K, 8);
    assert_field_count(LATEST, 8);
    for fixture in [BY_FORM, BY_SYMBOL, BY_CIK] {
        assert_field_count(fixture, 7);
    }

    let latest_8k: Vec<SecFiling> = serde_json::from_slice(LATEST_8K).unwrap();
    assert_eq!(latest_8k[0].symbol, Ticker::new("SUNE").unwrap());
    assert_eq!(latest_8k[0].cik, Cik::new("0000022701").unwrap());
    assert_eq!(
        latest_8k[0].filing_date,
        ApiDateTime::parse("2024-03-04 00:00:00").unwrap()
    );
    assert_eq!(
        latest_8k[0].accepted_date,
        ApiDateTime::parse("2024-03-01 22:47:48").unwrap()
    );
    assert_eq!(latest_8k[0].form_type, FormType::new("8-K").unwrap());
    assert_eq!(latest_8k[0].has_financials, None);
    assert!(latest_8k[0].link.ends_with("-index.htm"));
    assert!(latest_8k[0].final_link.ends_with("_8k.htm"));

    let latest: Vec<SecFiling> = serde_json::from_slice(LATEST).unwrap();
    assert_eq!(latest[0].has_financials, Some(true));

    for fixture in [BY_FORM, BY_SYMBOL, BY_CIK] {
        let rows: Vec<SecFiling> = serde_json::from_slice(fixture).unwrap();
        assert_eq!(rows[0].has_financials, None);
    }
}

#[test]
fn company_search_fixtures_preserve_none_empty_strings_addresses_and_leading_zeroes() {
    for fixture in [
        COMPANIES_NAME,
        COMPANIES_SYMBOL,
        COMPANIES_CIK,
        ALL_CLASSIFICATIONS,
    ] {
        assert_field_count(fixture, 7);
        let rows: Vec<SecCompanySearchResult> = serde_json::from_slice(fixture).unwrap();
        assert_eq!(serde_json::to_value(&rows).unwrap()[0], source_row(fixture));
    }

    let names: Vec<SecCompanySearchResult> = serde_json::from_slice(COMPANIES_NAME).unwrap();
    assert_eq!(names[0].symbol.as_str(), "None");
    assert_eq!(names[0].cik.as_str(), "0001418405");
    assert_eq!(names[0].sic_code, "");
    assert_eq!(names[0].industry_title, "");
    assert_eq!(
        names[0].business_address,
        "c/o Berkshire Property Advisors LLC, Boston MA 02108"
    );

    let all: Vec<SecCompanySearchResult> = serde_json::from_slice(ALL_CLASSIFICATIONS).unwrap();
    assert_eq!(all[0].cik.as_str(), "0000070858");
    assert_eq!(
        all[0].business_address,
        "['BANK OF AMERICA CORPORATE CENTER', 'CHARLOTTE NC 28255']"
    );
}

#[test]
fn full_profile_decodes_exactly_thirty_five_fields_and_documented_types() {
    assert_field_count(PROFILE, 35);
    let rows: Vec<SecCompanyProfile> = serde_json::from_slice(PROFILE).unwrap();
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.symbol, Ticker::new("AAPL").unwrap());
    assert_eq!(row.cik, Cik::new("0000320193").unwrap());
    assert_eq!(row.isin, Isin::new("US0378331005").unwrap());
    assert_eq!(row.country, CountryCode::new("US").unwrap());
    assert_eq!(row.exchange, ExchangeCode::new("NASDAQ").unwrap());
    assert_eq!(row.ipo_date, Date::parse("1980-12-12").unwrap());
    assert_eq!(row.employees, NumericString::new("166000").unwrap());
    assert_eq!(row.price_currency, CurrencyCode::new("USD").unwrap());
    assert_eq!(row.market_sector, Sector::new("Technology").unwrap());
    assert_eq!(row.security_type, None);
    assert!(row.is_active);
    assert!(!row.is_etf && !row.is_adr && !row.is_fund);
    assert_eq!(serde_json::to_value(rows).unwrap()[0], source_row(PROFILE));
}

#[test]
fn sic_list_is_typed_but_documented_empty_search_stays_raw() {
    assert_field_count(CLASSIFICATIONS, 3);
    let rows: Vec<SicClassification> = serde_json::from_slice(CLASSIFICATIONS).unwrap();
    assert_eq!(
        rows,
        [SicClassification {
            office: "Office of Life Sciences".to_owned(),
            sic_code: "100".to_owned(),
            industry_title: "AGRICULTURAL PRODUCTION-CROPS".to_owned(),
        }]
    );
    assert_eq!(
        serde_json::to_value(rows).unwrap()[0],
        source_row(CLASSIFICATIONS)
    );

    let raw: Vec<DynamicObject> = serde_json::from_slice(CLASSIFICATION_SEARCH).unwrap();
    assert_eq!(raw, [DynamicObject::new()]);
    let arbitrary: Vec<DynamicObject> = serde_json::from_value(serde_json::json!([
        {"future": [1, true, null], "nested": {"sicCode": "07371"}}
    ]))
    .unwrap();
    assert_eq!(arbitrary[0]["future"][0], 1);
}

#[test]
fn typed_contracts_require_documented_non_null_fields_and_accept_unknown_fields() {
    for fixture in [LATEST_8K, LATEST, BY_FORM, BY_SYMBOL, BY_CIK] {
        assert_contract_except::<SecFiling>(fixture, &["hasFinancials"]);
    }
    for fixture in [
        COMPANIES_NAME,
        COMPANIES_SYMBOL,
        COMPANIES_CIK,
        ALL_CLASSIFICATIONS,
    ] {
        assert_contract_except::<SecCompanySearchResult>(fixture, &[]);
    }
    assert_contract_except::<SecCompanyProfile>(PROFILE, &["securityType"]);
    assert_contract_except::<SicClassification>(CLASSIFICATIONS, &[]);

    let mut profile: serde_json::Value = serde_json::from_slice(PROFILE).unwrap();
    profile[0].as_object_mut().unwrap().remove("securityType");
    assert!(serde_json::from_value::<Vec<SecCompanyProfile>>(profile).is_err());
}

#[test]
fn every_contract_is_a_bare_array_accepting_empty_and_multiple_rows() {
    for fixture in [LATEST_8K, LATEST, BY_FORM, BY_SYMBOL, BY_CIK] {
        assert_bare_array::<SecFiling>(fixture);
    }
    for fixture in [
        COMPANIES_NAME,
        COMPANIES_SYMBOL,
        COMPANIES_CIK,
        ALL_CLASSIFICATIONS,
    ] {
        assert_bare_array::<SecCompanySearchResult>(fixture);
    }
    assert_bare_array::<SecCompanyProfile>(PROFILE);
    assert_bare_array::<SicClassification>(CLASSIFICATIONS);
    assert_bare_array::<DynamicObject>(CLASSIFICATION_SEARCH);
    assert!(serde_json::from_value::<Vec<SecFiling>>(serde_json::json!({"data": []})).is_err());
}

fn source_row(fixture: &[u8]) -> serde_json::Value {
    serde_json::from_slice::<serde_json::Value>(fixture).unwrap()[0].clone()
}

fn assert_field_count(fixture: &[u8], expected: usize) {
    assert_eq!(source_row(fixture).as_object().unwrap().len(), expected);
}

fn assert_contract_except<T: DeserializeOwned>(fixture: &[u8], optional: &[&str]) {
    let source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    for key in source[0].as_object().unwrap().keys() {
        if optional.contains(&key.as_str()) {
            continue;
        }
        let mut missing = source.clone();
        missing[0].as_object_mut().unwrap().remove(key);
        assert!(
            serde_json::from_value::<Vec<T>>(missing).is_err(),
            "accepted missing {key}"
        );

        let mut null = source.clone();
        null[0][key] = serde_json::Value::Null;
        assert!(
            serde_json::from_value::<Vec<T>>(null).is_err(),
            "accepted null {key}"
        );
    }

    let mut forward = source;
    forward[0]["futureProviderField"] = serde_json::json!({"nested": [1, true, null]});
    assert_eq!(serde_json::from_value::<Vec<T>>(forward).unwrap().len(), 1);
}

fn assert_bare_array<T: DeserializeOwned + Serialize>(fixture: &[u8]) {
    assert!(serde_json::from_slice::<Vec<T>>(b"[]").unwrap().is_empty());
    let mut source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    let duplicate = source[0].clone();
    source.as_array_mut().unwrap().push(duplicate);
    assert_eq!(serde_json::from_value::<Vec<T>>(source).unwrap().len(), 2);
}
