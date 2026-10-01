//! CSV row decoding for the bulk routes (ADR 0035).
//!
//! The `csv` crate only splits the body into records. Each record is then
//! handed to the row's `Deserialize` impl as a map from header name to cell,
//! so serde names, renames, and field codecs apply unchanged and a failure
//! reports `[row].member` like the JSON decoder. A cell is always text: an
//! empty cell is the only absence marker, so it becomes `None` for an
//! `Option` member, stays `""` for a string member, and is reported as
//! [`DecodeErrorKind::Null`] anywhere a value is required.

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
        let cells = headers.iter().zip(record.iter().map(Cell));
        let row = serde_path_to_error::deserialize(MapDeserializer::new(cells))
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
