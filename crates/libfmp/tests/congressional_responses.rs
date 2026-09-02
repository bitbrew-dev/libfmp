use libfmp::{
    codecs::TitleCaseBoolFlag, responses::congressional::CongressionalTrade, types::Date,
};
use serde::{Serialize, de::DeserializeOwned};

const FIXTURES: [&[u8]; 8] = [
    include_bytes!("fixtures/congress_senate_latest.json"),
    include_bytes!("fixtures/congress_house_latest.json"),
    include_bytes!("fixtures/congress_senate_trades.json"),
    include_bytes!("fixtures/congress_senate_trades_by_name.json"),
    include_bytes!("fixtures/congress_senate_trades_by_id.json"),
    include_bytes!("fixtures/congress_house_trades.json"),
    include_bytes!("fixtures/congress_house_trades_by_name.json"),
    include_bytes!("fixtures/congress_house_trades_by_id.json"),
];

#[test]
fn all_eight_exact_fixtures_decode_and_round_trip() {
    for fixture in FIXTURES {
        let source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
        let rows: Vec<CongressionalTrade> = serde_json::from_value(source.clone()).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(serde_json::to_value(rows).unwrap(), source);
    }
}

#[test]
fn shared_row_preserves_identifiers_dates_empty_symbol_and_exact_boolean_flag() {
    let senate: Vec<CongressionalTrade> = serde_json::from_slice(FIXTURES[3]).unwrap();
    assert_eq!(senate[0].symbol, "");
    assert_eq!(senate[0].member_id.as_str(), "M000934");
    assert_eq!(
        senate[0].disclosure_date,
        Date::parse("2026-07-21").unwrap()
    );
    assert_eq!(
        senate[0].transaction_date,
        Date::parse("2026-06-23").unwrap()
    );
    assert_eq!(
        senate[0].capital_gains_over_200_usd,
        Some(TitleCaseBoolFlag::False)
    );

    let latest: Vec<CongressionalTrade> = serde_json::from_slice(FIXTURES[0]).unwrap();
    assert_eq!(latest[0].capital_gains_over_200_usd, None);
    assert!(
        !serde_json::to_value(&latest[0])
            .unwrap()
            .as_object()
            .unwrap()
            .contains_key("capitalGainsOver200USD")
    );
}

#[test]
fn capital_gain_is_the_only_optional_field_and_unknowns_are_accepted() {
    let required_keys = [
        "symbol",
        "senateID",
        "disclosureDate",
        "transactionDate",
        "firstName",
        "lastName",
        "office",
        "district",
        "owner",
        "assetDescription",
        "assetType",
        "type",
        "amount",
        "comment",
        "link",
    ];
    let source: serde_json::Value = serde_json::from_slice(FIXTURES[1]).unwrap();
    assert_eq!(source[0].as_object().unwrap().len(), 16);
    for key in required_keys {
        let mut missing = source.clone();
        missing[0].as_object_mut().unwrap().remove(key);
        assert!(
            serde_json::from_value::<Vec<CongressionalTrade>>(missing).is_err(),
            "{key} unexpectedly accepted when missing"
        );

        let mut null = source.clone();
        null[0][key] = serde_json::Value::Null;
        assert!(
            serde_json::from_value::<Vec<CongressionalTrade>>(null).is_err(),
            "{key} unexpectedly accepted as null"
        );
    }

    let mut missing_capital_gain = source.clone();
    missing_capital_gain[0]
        .as_object_mut()
        .unwrap()
        .remove("capitalGainsOver200USD");
    let rows: Vec<CongressionalTrade> =
        serde_json::from_value(missing_capital_gain.clone()).unwrap();
    assert_eq!(rows[0].capital_gains_over_200_usd, None);
    assert_eq!(serde_json::to_value(rows).unwrap(), missing_capital_gain);

    let mut unknown = source;
    unknown[0]["futureProviderField"] = serde_json::json!({ "accepted": true });
    assert!(serde_json::from_value::<Vec<CongressionalTrade>>(unknown).is_ok());
}

#[test]
fn explicit_provider_keys_do_not_accept_rust_or_ordinary_camel_case_aliases() {
    for (wire, alias) in [("senateID", "memberId"), ("type", "transactionType")] {
        let mut source: serde_json::Value = serde_json::from_slice(FIXTURES[1]).unwrap();
        let value = source[0].as_object_mut().unwrap().remove(wire).unwrap();
        source[0][alias] = value;
        assert!(serde_json::from_value::<Vec<CongressionalTrade>>(source).is_err());
    }

    let mut capital_gain: serde_json::Value = serde_json::from_slice(FIXTURES[1]).unwrap();
    let value = capital_gain[0]
        .as_object_mut()
        .unwrap()
        .remove("capitalGainsOver200USD")
        .unwrap();
    capital_gain[0]["capitalGainsOver200Usd"] = value;
    let rows: Vec<CongressionalTrade> = serde_json::from_value(capital_gain).unwrap();
    assert_eq!(rows[0].capital_gains_over_200_usd, None);
}

#[test]
fn contracts_are_bare_arrays_with_empty_and_multiple_rows() {
    assert!(serde_json::from_value::<Vec<CongressionalTrade>>(serde_json::json!([])).is_ok());
    let source: serde_json::Value = serde_json::from_slice(FIXTURES[1]).unwrap();
    let row = source[0].clone();
    let rows: Vec<CongressionalTrade> =
        serde_json::from_value(serde_json::json!([row.clone(), row])).unwrap();
    assert_eq!(rows.len(), 2);
    assert!(
        serde_json::from_value::<Vec<CongressionalTrade>>(serde_json::json!({ "trades": [] }))
            .is_err()
    );
}

#[allow(dead_code)]
fn assert_traits<T: DeserializeOwned + Serialize>() {}

#[test]
fn congressional_trade_supports_the_public_serde_contract() {
    assert_traits::<CongressionalTrade>();
}
