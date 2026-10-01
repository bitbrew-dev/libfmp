//! CSV row decoding for the bulk routes (ADR 0035).
//!
//! The `csv` crate only splits the body into records. Each record is then
//! handed to the row's `Deserialize` impl as a map from header name to cell,
//! so serde names, renames, and field codecs apply unchanged and a failure
//! reports `[row].member` like the JSON decoder. Every member of the row,
//! `Option` or not, must be a header column; the first record checks it. A
//! cell is always text: an empty cell is the only absence marker, so it
//! becomes `None` for an `Option` member, stays `""` for a string member,
//! and is reported as [`DecodeErrorKind::Null`] anywhere a value is
//! required.

use bytes::Bytes;
use serde::de::{
    self, DeserializeOwned, IntoDeserializer, Unexpected, Visitor,
    value::{Error as CellError, MapDeserializer},
};

use super::{DecodeFailure, ResponseMetadata, missing_member_name};
use crate::error::DecodeErrorKind;

pub(super) fn decode_csv<Row>(
    body: Bytes,
    _metadata: ResponseMetadata<'_>,
) -> std::result::Result<Vec<Row>, DecodeFailure>
where
    Row: DeserializeOwned,
{
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(false)
        .from_reader(body.as_ref());
    let headers = reader.headers().map_err(|_| syntax(None))?.clone();
    let mut rows = Vec::new();
    let mut record = csv::StringRecord::new();
    loop {
        match reader.read_record(&mut record) {
            Ok(true) => {}
            Ok(false) => return Ok(rows),
            Err(_) => return Err(syntax(Some(rows.len()))),
        }
        let cells = RowCells {
            headers: &headers,
            record: &record,
            require_columns: rows.is_empty(),
        };
        let row = serde_path_to_error::deserialize(cells)
            .map_err(|error| row_failure(rows.len(), &error, &headers, &record))?;
        rows.push(row);
    }
}

fn syntax(row: Option<usize>) -> DecodeFailure {
    DecodeFailure::Shape {
        path: row.map(|row| format!("[{row}]")),
        kind: DecodeErrorKind::Syntax,
    }
}

fn row_failure(
    row: usize,
    error: &serde_path_to_error::Error<CellError>,
    headers: &csv::StringRecord,
    record: &csv::StringRecord,
) -> DecodeFailure {
    let message = error.inner().to_string();
    let member = if error.path().iter().next().is_some() {
        Some(error.path().to_string())
    } else {
        missing_member_name(&message).map(str::to_owned)
    };
    let kind = if message.starts_with("missing field ") {
        DecodeErrorKind::MissingMember
    } else if member
        .as_deref()
        .and_then(|member| headers.iter().position(|header| header == member))
        .and_then(|column| record.get(column))
        .is_some_and(str::is_empty)
    {
        DecodeErrorKind::Null
    } else if message.starts_with("invalid type: ") {
        DecodeErrorKind::WrongType
    } else {
        DecodeErrorKind::InvalidValue
    };
    let path = match member {
        Some(member) => format!("[{row}].{member}"),
        None => format!("[{row}]"),
    };
    DecodeFailure::Shape {
        path: Some(path),
        kind,
    }
}

struct RowCells<'a> {
    headers: &'a csv::StringRecord,
    record: &'a csv::StringRecord,
    require_columns: bool,
}

impl<'de> de::Deserializer<'de> for RowCells<'_> {
    type Error = CellError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, CellError> {
        visitor.visit_map(MapDeserializer::new(
            self.headers.iter().zip(self.record.iter().map(Cell)),
        ))
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, CellError> {
        if self.require_columns
            && let Some(field) = fields
                .iter()
                .find(|field| !self.headers.iter().any(|header| header == **field))
        {
            return Err(de::Error::missing_field(field));
        }
        self.deserialize_any(visitor)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf
        option unit unit_struct newtype_struct seq tuple tuple_struct map enum identifier
        ignored_any
    }
}

struct Cell<'a>(&'a str);

impl<'de> IntoDeserializer<'de, CellError> for Cell<'_> {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self {
        self
    }
}

macro_rules! parse_cell {
    ($($method:ident => $visit:ident),* $(,)?) => {
        $(
            fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, CellError> {
                match self.0.parse() {
                    Ok(value) => visitor.$visit(value),
                    Err(_) => Err(de::Error::invalid_type(Unexpected::Str(self.0), &visitor)),
                }
            }
        )*
    };
}

impl<'de> de::Deserializer<'de> for Cell<'_> {
    type Error = CellError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, CellError> {
        visitor.visit_str(self.0)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, CellError> {
        if self.0.is_empty() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, CellError> {
        match self.0 {
            "true" => visitor.visit_bool(true),
            "false" => visitor.visit_bool(false),
            other => Err(de::Error::invalid_type(Unexpected::Str(other), &visitor)),
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, CellError> {
        if self.0.is_empty() {
            visitor.visit_unit()
        } else {
            Err(de::Error::invalid_type(Unexpected::Str(self.0), &visitor))
        }
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, CellError> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, CellError> {
        visitor.visit_enum(self.0.into_deserializer())
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, CellError> {
        visitor.visit_unit()
    }

    parse_cell! {
        deserialize_i8 => visit_i8,
        deserialize_i16 => visit_i16,
        deserialize_i32 => visit_i32,
        deserialize_i64 => visit_i64,
        deserialize_i128 => visit_i128,
        deserialize_u8 => visit_u8,
        deserialize_u16 => visit_u16,
        deserialize_u32 => visit_u32,
        deserialize_u64 => visit_u64,
        deserialize_u128 => visit_u128,
        deserialize_f32 => visit_f32,
        deserialize_f64 => visit_f64,
    }

    serde::forward_to_deserialize_any! {
        char str string bytes byte_buf unit_struct seq tuple tuple_struct map struct identifier
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;
    use crate::{codecs::NumericString, endpoints::ResponseContract, types::Date};

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Row {
        symbol: String,
        date: Date,
        price: f64,
        listed: bool,
        volume: Option<NumericString>,
        #[serde(rename = "Stock Price")]
        stock_price: Option<f64>,
        peers: String,
    }

    const HEADER: &str =
        "\"symbol\",\"date\",\"price\",\"listed\",\"volume\",\"Stock Price\",\"peers\"\n";

    fn decode(body: &str) -> std::result::Result<Vec<Row>, DecodeFailure> {
        ResponseContract::<Vec<Row>>::csv().decode(
            Bytes::from(body.to_owned()),
            ResponseMetadata::new("text/csv", None),
        )
    }

    fn failure(body: &str) -> (Option<String>, DecodeErrorKind) {
        match decode(body).unwrap_err() {
            DecodeFailure::Shape { path, kind } => (path, kind),
            DecodeFailure::ProviderMessage => panic!("CSV never reports a provider message"),
        }
    }

    #[test]
    fn empty_and_header_only_bodies_decode_to_no_rows() {
        assert_eq!(decode("").unwrap(), []);
        assert_eq!(decode(HEADER).unwrap(), []);
        assert_eq!(decode("symbol,price\n").unwrap(), []);
    }

    #[test]
    fn cells_decode_by_header_name_with_quoted_commas_and_verbatim_numbers() {
        let body = format!(
            "{HEADER}\"AAPL\",\"2025-06-02\",201.7,true,6.9148336e-9,12.5,\"MSFT,GOOG\"\n\
             \"\",\"2025-06-02\",-0.5,false,,,\"say \"\"hi\"\"\"\n"
        );
        let rows = decode(&body).unwrap();

        assert_eq!(rows[0].symbol, "AAPL");
        assert_eq!(rows[0].date, Date::parse("2025-06-02").unwrap());
        assert_eq!(rows[0].price, 201.7);
        assert!(rows[0].listed);
        assert_eq!(rows[0].volume.as_ref().unwrap().as_str(), "6.9148336e-9");
        assert_eq!(rows[0].stock_price, Some(12.5));
        assert_eq!(rows[0].peers, "MSFT,GOOG");
        assert_eq!(rows[1].symbol, "");
        assert_eq!(rows[1].volume, None);
        assert_eq!(rows[1].stock_price, None);
        assert_eq!(rows[1].peers, "say \"hi\"");
    }

    #[test]
    fn a_leading_byte_order_mark_is_not_part_of_the_first_header() {
        let rows = decode(&format!("\u{feff}{HEADER}A,2025-06-02,1,true,1,1,x\r\n")).unwrap();

        assert_eq!(rows[0].symbol, "A");
        assert_eq!(rows[0].peers, "x");
    }

    #[test]
    fn an_empty_cell_on_a_required_member_is_a_null_at_its_row_and_member() {
        let body = format!("{HEADER}A,2025-06-02,1,true,1,1,x\nB,2025-06-02,,true,1,1,x\n");
        assert_eq!(
            failure(&body),
            (Some("[1].price".into()), DecodeErrorKind::Null)
        );
        let body = format!("{HEADER}A,,1,true,1,1,x\n");
        assert_eq!(
            failure(&body),
            (Some("[0].date".into()), DecodeErrorKind::Null)
        );
    }

    #[test]
    fn malformed_cells_are_classified_without_their_values() {
        let body = format!("{HEADER}A,2025-06-02,abc,true,1,1,x\n");
        assert_eq!(
            failure(&body),
            (Some("[0].price".into()), DecodeErrorKind::WrongType)
        );
        let body = format!("{HEADER}A,2025-06-02,1,yes,1,1,x\n");
        assert_eq!(
            failure(&body),
            (Some("[0].listed".into()), DecodeErrorKind::WrongType)
        );
        let body = format!("{HEADER}A,2025-13-40,1,true,1,1,x\n");
        assert_eq!(
            failure(&body),
            (Some("[0].date".into()), DecodeErrorKind::InvalidValue)
        );
    }

    #[test]
    fn a_missing_column_is_a_missing_member_even_for_an_option() {
        let body = "symbol,date,price,listed,volume,Stock Price\nA,2025-06-02,1,true,1,1\n";
        assert_eq!(
            failure(body),
            (Some("[0].peers".into()), DecodeErrorKind::MissingMember)
        );
        let body = "symbol,date,price,listed,Stock Price,peers\nA,2025-06-02,1,true,1,x\n";
        assert_eq!(
            failure(body),
            (Some("[0].volume".into()), DecodeErrorKind::MissingMember)
        );
        assert_eq!(decode("symbol,date\n").unwrap(), []);
    }

    #[test]
    fn a_record_with_the_wrong_cell_count_is_a_syntax_error_at_its_row() {
        let body = format!("{HEADER}A,2025-06-02,1,true,1,1,x\nB,2025-06-02\n");
        assert_eq!(
            failure(&body),
            (Some("[1]".into()), DecodeErrorKind::Syntax)
        );
    }

    #[test]
    fn a_one_line_message_is_a_header_not_a_provider_message() {
        assert_eq!(decode("Invalid name\n").unwrap(), []);
    }

    #[test]
    fn the_csv_contract_accepts_only_text_csv() {
        let expected = ResponseContract::<Vec<Row>>::csv().expected_content_type();

        assert!(expected.matches("text/csv"));
        assert!(expected.matches("Text/CSV; charset=utf-8"));
        assert!(!expected.matches("application/json"));
        assert!(!expected.matches("text/plain"));
    }
}
