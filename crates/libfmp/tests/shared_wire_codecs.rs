use std::str::FromStr;

use libfmp::{
    codecs::{
        DateOrDateTime, DateOrYear, DynamicJson, DynamicObject, FiscalYear, IsoTimestamp,
        NumberOrNumericString, OpaqueDateText, PercentageValue, TitleCaseBoolFlag, TrueFalseFlag,
        UsDate, WireBool, YesNoFlag, YnFlag, empty_date, empty_or_null_date,
        empty_or_null_date_or_datetime,
    },
    types::Date,
};
use serde::{Deserialize, Serialize};

#[test]
fn mixed_numeric_forms_round_trip_without_f64_coercion() {
    let year_string: FiscalYear = serde_json::from_str(r#""2025""#).unwrap();
    let year_integer: FiscalYear = serde_json::from_str("2025").unwrap();
    assert_eq!(serde_json::to_string(&year_string).unwrap(), r#""2025""#);
    assert_eq!(serde_json::to_string(&year_integer).unwrap(), "2025");
    assert!(serde_json::from_str::<FiscalYear>("-2025").is_err());
    assert!(serde_json::from_str::<FiscalYear>(r#""20.25""#).is_err());

    let documented_string: NumberOrNumericString =
        serde_json::from_str(r#""33644000000""#).unwrap();
    let documented_number: NumberOrNumericString = serde_json::from_str("416161000000").unwrap();
    assert_eq!(
        serde_json::to_string(&documented_string).unwrap(),
        r#""33644000000""#
    );
    assert_eq!(
        serde_json::to_string(&documented_number).unwrap(),
        "416161000000"
    );

    let beyond_javascript_safe_integer = "9007199254740993";
    let exact: NumberOrNumericString =
        serde_json::from_str(beyond_javascript_safe_integer).unwrap();
    assert_eq!(
        serde_json::to_string(&exact).unwrap(),
        beyond_javascript_safe_integer
    );

    // This cannot be held by u64. The exact bare-number round trip depends on
    // serde_json's arbitrary-precision representation remaining enabled.
    let beyond_u64 = "18446744073709551616";
    let exact: NumberOrNumericString = serde_json::from_str(beyond_u64).unwrap();
    assert_eq!(serde_json::to_string(&exact).unwrap(), beyond_u64);
}

#[test]
fn percentages_and_boolean_families_preserve_wire_kinds_and_spelling() {
    for wire in ["0.4690516410716045", r#""0.10335""#, r#""97.26%""#] {
        let value: PercentageValue = serde_json::from_str(wire).unwrap();
        assert_eq!(serde_json::to_string(&value).unwrap(), wire);
    }

    assert_eq!(
        serde_json::from_str::<WireBool>("true").unwrap(),
        WireBool(true)
    );
    assert_eq!(
        serde_json::from_str::<YnFlag>(r#""N""#).unwrap(),
        YnFlag::False
    );
    assert_eq!(
        serde_json::from_str::<YesNoFlag>(r#""Yes""#).unwrap(),
        YesNoFlag::True
    );
    assert_eq!(
        serde_json::from_str::<TrueFalseFlag>(r#""false""#).unwrap(),
        TrueFalseFlag::False
    );
    assert_eq!(
        serde_json::from_str::<TitleCaseBoolFlag>(r#""False""#).unwrap(),
        TitleCaseBoolFlag::False
    );
    assert!(serde_json::from_str::<YnFlag>(r#""No""#).is_err());
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct OptionalTemporalFixture {
    #[serde(default, with = "empty_or_null_date")]
    declaration_date: Option<Date>,
    #[serde(default, with = "empty_or_null_date_or_datetime")]
    filing_date: Option<DateOrDateTime>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct RequiredEmptyDateFixture {
    #[serde(with = "empty_date")]
    date_of_first_sale: Option<Date>,
}

#[test]
fn temporal_contracts_are_strict_and_empty_or_null_aware() {
    assert_eq!(
        IsoTimestamp::from_str("2026-07-30T16:00:20.049Z")
            .unwrap()
            .to_string(),
        "2026-07-30T16:00:20.049Z"
    );
    assert!(IsoTimestamp::from_str("2026-07-30 16:00:20").is_err());
    assert_eq!(
        UsDate::from_str("10-31-2026").unwrap().to_string(),
        "10-31-2026"
    );
    assert!(UsDate::from_str("2026-10-31").is_err());

    assert!(matches!(
        DateOrDateTime::from_str("2023-11-13").unwrap(),
        DateOrDateTime::Date(_)
    ));
    assert!(matches!(
        DateOrDateTime::from_str("2026-07-30 00:00:00").unwrap(),
        DateOrDateTime::DateTime(_)
    ));
    assert!(DateOrDateTime::from_str("2026-07-30T00:00:00Z").is_err());

    let year: DateOrYear = serde_json::from_str(r#""2020""#).unwrap();
    assert_eq!(year, DateOrYear::Year(2020));
    assert_eq!((year.as_year(), year.as_date()), (Some(2020), None));
    assert_eq!(serde_json::to_string(&year).unwrap(), r#""2020""#);
    let date: DateOrYear = serde_json::from_str(r#""2026-07-28""#).unwrap();
    assert_eq!(date, DateOrYear::Date(Date::parse("2026-07-28").unwrap()));
    assert_eq!(date.as_year(), None);
    assert_eq!(serde_json::to_string(&date).unwrap(), r#""2026-07-28""#);
    for garbage in [
        r#""20x0""#,
        r#""20201""#,
        r#""202""#,
        r#""2026-13-01""#,
        r#""2026-07-28 00:00:00""#,
        r#"" 2020""#,
        r#""""#,
    ] {
        let message = serde_json::from_str::<DateOrYear>(garbage)
            .unwrap_err()
            .to_string();
        let inner = garbage.trim_matches('"');
        assert!(
            inner.is_empty() || !message.contains(inner),
            "{garbage}: error echoes the value"
        );
    }
    assert!(serde_json::from_str::<DateOrYear>("2020").is_err());
    assert!(serde_json::from_str::<DateOrYear>("null").is_err());

    for wire in [
        r#"{"declaration_date":"","filing_date":null}"#,
        r#"{"declaration_date":null,"filing_date":""}"#,
        r#"{}"#,
    ] {
        let fixture: OptionalTemporalFixture = serde_json::from_str(wire).unwrap();
        assert_eq!(fixture.declaration_date, None);
        assert_eq!(fixture.filing_date, None);
    }

    let partial = OpaqueDateText("--09-27".to_owned());
    assert_eq!(serde_json::to_string(&partial).unwrap(), r#""--09-27""#);
}

#[test]
fn required_empty_date_preserves_its_sentinel_and_rejects_null_missing_or_malformed_values() {
    let empty: RequiredEmptyDateFixture =
        serde_json::from_str(r#"{"date_of_first_sale":""}"#).unwrap();
    assert_eq!(empty.date_of_first_sale, None);
    assert_eq!(
        serde_json::to_string(&empty).unwrap(),
        r#"{"date_of_first_sale":""}"#
    );

    let actual: RequiredEmptyDateFixture =
        serde_json::from_str(r#"{"date_of_first_sale":"2014-02-14"}"#).unwrap();
    assert_eq!(actual.date_of_first_sale.unwrap().to_string(), "2014-02-14");
    assert_eq!(
        serde_json::to_string(&actual).unwrap(),
        r#"{"date_of_first_sale":"2014-02-14"}"#
    );

    for wire in [
        r#"{"date_of_first_sale":null}"#,
        r#"{}"#,
        r#"{"date_of_first_sale":"02-14-2014"}"#,
        r#"{"date_of_first_sale":"2014-02-30"}"#,
    ] {
        assert!(serde_json::from_str::<RequiredEmptyDateFixture>(wire).is_err());
    }
}

#[test]
fn dynamic_json_preserves_recursive_native_shape() {
    let value: DynamicJson = serde_json::from_str(
        r#"{"documenttype":"10-K","documentannualreport":"true","documentfiscalyearfocus":2025,"nested":[null,false,{"amount":"33644000000","exact":9007199254740993}]}"#,
    )
    .unwrap();

    assert_eq!(value["documenttype"], "10-K");
    assert_eq!(value["documentfiscalyearfocus"], 2025);
    assert_eq!(value["nested"][2]["amount"], "33644000000");
    assert_eq!(value["nested"][2]["exact"].to_string(), "9007199254740993");
}

#[test]
fn dynamic_object_preserves_semantics_without_treating_member_order_as_data() {
    let wire = r#"{
        "Issuer Defined Section": {
            "heterogeneousCells": [null, false, "text", 184467440737095516160, -42, 0.125],
            "nested": {"arbitrary key": [1, {"flag": true}]}
        },
        "another section": []
    }"#;
    let object: DynamicObject = serde_json::from_str(wire).unwrap();

    assert_eq!(
        object["Issuer Defined Section"]["heterogeneousCells"][3].to_string(),
        "184467440737095516160"
    );
    assert_eq!(
        object["Issuer Defined Section"]["nested"]["arbitrary key"][1]["flag"],
        true
    );

    // JSON object member order is intentionally not part of this contract;
    // semantic equality, arbitrary keys, and exact value tokens are preserved.
    let encoded = serde_json::to_string(&object).unwrap();
    let round_trip: DynamicObject = serde_json::from_str(&encoded).unwrap();
    assert_eq!(round_trip, object);
    assert_eq!(
        round_trip["Issuer Defined Section"]["heterogeneousCells"][3].to_string(),
        "184467440737095516160"
    );

    assert!(serde_json::from_str::<DynamicObject>("[]").is_err());
    assert!(serde_json::from_str::<DynamicObject>("null").is_err());
}
