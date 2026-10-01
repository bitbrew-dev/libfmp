//! Shared CSV contract checks for the bulk test suites (ADR 0035).
//!
//! Bulk routes answer `text/csv`, so these helpers decode a fixture through
//! the real client and CSV contract instead of `serde_json`.

use std::sync::Arc;

use http::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use libfmp::{
    Client,
    client::{EndpointSpec, QueryParameters},
    error::DecodeErrorKind,
    transport::TransportResponse,
};
use serde::Serialize;
use serde_json::Value;

use crate::support::FixtureExecutor;

/// A `200 text/csv` fixture response.
fn csv_fixture(body: impl Into<Vec<u8>>) -> TransportResponse {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/csv"));
    TransportResponse::new(200, headers, body)
}

/// Decodes `body` through `endpoint` on a fixture client, as the bulk method does.
pub fn decode<Q, R>(
    endpoint: &EndpointSpec<Q, Vec<R>>,
    body: impl Into<Vec<u8>>,
) -> libfmp::Result<Vec<R>>
where
    Q: QueryParameters,
{
    let client = Client::builder()
        .base_url("https://fixture.test")
        .executor(Arc::new(FixtureExecutor::new([csv_fixture(body)])))
        .build()
        .unwrap();
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(client.execute(endpoint))
}

/// Asserts every fixture row decodes and re-encodes its first row with
/// `fields` members, each equal to its cell: verbatim text for strings and
/// numeric strings, the parsed value for numbers and booleans, and null or
/// `""` for an empty cell.
pub fn assert_round_trip<Q, R>(endpoint: &EndpointSpec<Q, Vec<R>>, fixture: &[u8], fields: usize)
where
    Q: QueryParameters,
    R: Serialize,
{
    let (header, records) = read(fixture);
    let rows = decode(endpoint, fixture).unwrap();
    assert_eq!(rows.len(), records.len());
    let Value::Object(encoded) = serde_json::to_value(&rows[0]).unwrap() else {
        panic!("a row re-encodes as a JSON object");
    };
    assert_eq!(encoded.len(), fields);
    for (member, value) in &encoded {
        let column = header.iter().position(|name| name == member);
        let cell = &records[0][column.unwrap_or_else(|| panic!("{member} is a header column"))];
        let matches = match value {
            Value::Null => cell.is_empty(),
            Value::String(text) => text == cell,
            Value::Bool(flag) => flag.to_string() == *cell,
            Value::Number(number) => cell.parse::<f64>().ok() == number.as_f64(),
            _ => false,
        };
        assert!(matches, "{member} does not re-encode its cell");
    }
}

/// Asserts the CSV member contract on the fixture's first record: a blanked
/// model member's cell decodes only for a member in `blankable`, and is
/// otherwise a null at `[0].member`; dropping a model member's column is a missing member even
/// for an `Option`, while any other column, and an unknown one, is ignored.
pub fn assert_members<Q, R>(endpoint: &EndpointSpec<Q, Vec<R>>, fixture: &[u8], blankable: &[&str])
where
    Q: QueryParameters,
    R: Serialize,
{
    let (header, records) = read(fixture);
    let rows = decode(endpoint, fixture).unwrap();
    let Value::Object(members) = serde_json::to_value(&rows[0]).unwrap() else {
        panic!("a row re-encodes as a JSON object");
    };
    for (column, member) in header.iter().enumerate() {
        let mut blank = records[0].clone();
        blank[column].clear();
        let result = decode(endpoint, write(&header, &blank));
        if blankable.contains(&member.as_str()) || !members.contains_key(member) {
            assert!(result.is_ok(), "blank {member} did not decode");
        } else {
            assert_failure(result, member, DecodeErrorKind::Null);
        }

        let keep = |cells: &[String]| -> Vec<String> {
            cells
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != column)
                .map(|(_, cell)| cell.clone())
                .collect()
        };
        let dropped = decode(endpoint, write(&keep(&header), &keep(&records[0])));
        if members.contains_key(member) {
            assert_failure(dropped, member, DecodeErrorKind::MissingMember);
        } else {
            assert!(dropped.is_ok(), "dropping {member} failed");
        }
    }
    let mut future_header = header.clone();
    future_header.push("futureColumn".to_owned());
    let mut future = records[0].clone();
    future.push("future value".to_owned());
    assert!(decode(endpoint, write(&future_header, &future)).is_ok());
}

/// Asserts the empty and header-only bodies of an endpoint decode to no rows.
pub fn assert_empty_bodies<Q, R>(endpoint: &EndpointSpec<Q, Vec<R>>, fixture: &[u8])
where
    Q: QueryParameters,
{
    let (header, _) = read(fixture);
    assert!(decode(endpoint, Vec::new()).unwrap().is_empty());
    assert!(decode(endpoint, write(&header, &[])).unwrap().is_empty());
}

/// Returns the fixture's header and first record with the named cells replaced.
#[allow(dead_code)] // Only suites that probe individual cells use it.
pub fn with_cells(fixture: &[u8], cells: &[(&str, &str)]) -> Vec<u8> {
    let (header, records) = read(fixture);
    let mut record = records[0].clone();
    for (name, value) in cells {
        let column = header.iter().position(|column| column == name);
        record[column.unwrap_or_else(|| panic!("{name} is a header column"))] = (*value).to_owned();
    }
    write(&header, &record)
}

fn assert_failure<R>(result: libfmp::Result<R>, member: &str, kind: DecodeErrorKind) {
    let Err(error) = result else {
        panic!("{member}: expected a {kind:?} failure");
    };
    assert_eq!(error.decode_path(), Some(format!("[0].{member}").as_str()));
    assert_eq!(error.decode_kind(), Some(kind), "{member}");
}

fn read(fixture: &[u8]) -> (Vec<String>, Vec<Vec<String>>) {
    let mut reader = csv::Reader::from_reader(fixture);
    let header = reader
        .headers()
        .unwrap()
        .iter()
        .map(str::to_owned)
        .collect();
    let records = reader
        .records()
        .map(|record| record.unwrap().iter().map(str::to_owned).collect())
        .collect();
    (header, records)
}

fn write(header: &[String], record: &[String]) -> Vec<u8> {
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer.write_record(header).unwrap();
    if !record.is_empty() {
        writer.write_record(record).unwrap();
    }
    writer.into_inner().unwrap()
}
