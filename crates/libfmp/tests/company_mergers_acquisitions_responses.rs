use libfmp::responses::company::MergerAcquisition;

const LATEST: &[u8] = include_bytes!("fixtures/company_mergers_acquisitions_latest.json");
const SEARCH: &[u8] = include_bytes!("fixtures/company_mergers_acquisitions_search.json");
const MULTIPLE: &[u8] = include_bytes!("fixtures/company_mergers_acquisitions_multiple.json");
const UNKNOWN: &[u8] = include_bytes!("fixtures/company_mergers_acquisitions_unknown.json");
const EMPTY: &[u8] = include_bytes!("fixtures/company_empty.json");

#[test]
fn documented_latest_merger_acquisition_decodes_every_exact_field_and_wire_type() {
    let rows: Vec<MergerAcquisition> = serde_json::from_slice(LATEST).unwrap();
    let row = &rows[0];

    assert_eq!(row.symbol.as_str(), "AGH");
    assert_eq!(row.company_name, "Aureus Greenway Holdings Inc");
    assert_eq!(row.cik.as_str(), "0002009312");
    assert_eq!(row.targeted_company_name, "Aureus Greenway Holdings, Inc.");
    assert_eq!(row.targeted_cik.as_str(), "0002009312");
    assert_eq!(row.targeted_symbol.as_str(), "PUSA");
    assert_eq!(row.transaction_date.to_string(), "2026-07-29");
    assert_eq!(row.accepted_date.to_string(), "2026-07-29 16:00:46");
    assert_eq!(
        row.link,
        "https://www.sec.gov/Archives/edgar/data/2009312/000149315226035181/forms-4.htm"
    );
}

#[test]
fn documented_search_merger_acquisition_preserves_leading_zero_ciks_names_and_url() {
    let rows: Vec<MergerAcquisition> = serde_json::from_slice(SEARCH).unwrap();
    let row = &rows[0];

    assert_eq!(row.symbol.as_str(), "PEGY");
    assert_eq!(row.company_name, "Pineapple Energy Inc.");
    assert_eq!(row.cik.as_str(), "0000022701");
    assert_eq!(row.targeted_company_name, "Communications Systems, Inc.");
    assert_eq!(row.targeted_cik.as_str(), "0000022701");
    assert_eq!(row.targeted_symbol.as_str(), "JCS");
    assert_eq!(row.transaction_date.to_string(), "2021-11-12");
    assert_eq!(row.accepted_date.to_string(), "2021-11-12 09:54:22");
    assert_eq!(
        row.link,
        "https://www.sec.gov/Archives/edgar/data/22701/000089710121000932/a211292_s-4.htm"
    );
}

#[test]
fn merger_acquisition_arrays_preserve_empty_multiple_and_unknown_field_shapes() {
    let empty: Vec<MergerAcquisition> = serde_json::from_slice(EMPTY).unwrap();
    let multiple: Vec<MergerAcquisition> = serde_json::from_slice(MULTIPLE).unwrap();
    let unknown: Vec<MergerAcquisition> = serde_json::from_slice(UNKNOWN).unwrap();

    assert!(empty.is_empty());
    assert_eq!(multiple.len(), 2);
    assert_eq!(multiple[0].targeted_symbol.as_str(), "PUSA");
    assert_eq!(multiple[1].targeted_symbol.as_str(), "JCS");
    assert_eq!(unknown.len(), 1);
    assert_eq!(unknown[0].cik.as_str(), "0002009312");
}
