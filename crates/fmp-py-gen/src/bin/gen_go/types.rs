//! The Rust-to-Go type table of ADR 0030 ("Codec policy") and the registry
//! arg-kind table. Every Rust type the table does not know is an error that
//! names the struct and the field; nothing is ever mapped to `any`.

use std::collections::{BTreeMap, BTreeSet};

use fmp_py_gen::registry::ArgKind;
use fmp_py_gen::responses::{FieldDef, StructDef, Wrap, peel};

use crate::emit::{exported, go_name};

/// How a member is decoded through the shadow struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Codec {
    /// A pointer shadow member: nil means missing or null.
    Plain,
    /// `deserialize_with = "required_option"`: the key must be present, null
    /// is allowed. The shadow keeps the raw value to tell the two apart.
    RequiredOption,
    /// `with = "empty_date"`: `""` is absent, null is rejected.
    EmptyDate,
    /// `with = "empty_or_null_date"`: `""` and null are both absent.
    EmptyOrNullDate,
    /// A required `DynamicJson`: null is a value, only a missing key fails.
    DynamicJson,
    /// `DynamicObject`: the value must be a JSON object.
    DynamicObject,
    /// `serde_json::Number` or `Option<Number>`: the raw digits are kept
    /// (integer and decimal spellings survive) and the value must be a JSON
    /// number. A pointer shadow member: nil means missing or null.
    Number,
    /// `deserialize_with = "required_option"` on `Option<Number>`: the key
    /// must be present, null is allowed, a present value must be a number.
    RequiredNumber,
    /// `deserialize_with = "crate::codecs::count::deserialize"` on a bare
    /// `Count`: the key is required, null is rejected, and an integral float
    /// such as `3.0` decodes as the integer. The shadow keeps the raw value.
    Count,
    /// `deserialize_with = "crate::codecs::empty_or_null::deserialize"` on an
    /// `Option` of a string-backed type: the key is required, and `""` and
    /// null are both absent (ADR 0033). The shadow keeps the raw value.
    EmptyOrNullString,
    /// A `DynamicObject` that holds every member the named fields do not
    /// claim, as serde `flatten` on a map does: `#[serde(flatten)]`, or the
    /// single `DynamicObject` field of a struct with a hand-written
    /// `Deserialize`. The member has no wire name; json/v2 `embed` collects
    /// and re-emits the remaining members.
    Embedded,
}

/// The `deserialize_with` path of the Rust count codec.
const COUNT_CODEC: &str = "crate::codecs::count::deserialize";

/// The `deserialize_with` path of the Rust empty-or-null typed-code codec.
const EMPTY_OR_NULL_CODEC: &str = "crate::codecs::empty_or_null::deserialize";

/// One Go struct member derived from a Rust field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GoField {
    /// The exported Go field name.
    pub(crate) name: String,
    /// The JSON member name (`json:"..."` tag).
    pub(crate) wire: String,
    /// The type on the public struct.
    pub(crate) public_ty: String,
    /// The type on the shadow struct.
    pub(crate) shadow_ty: String,
    /// Whether the Rust type is `Option<_>` at the outermost level.
    pub(crate) optional: bool,
    pub(crate) codec: Codec,
    /// `skip_serializing_if = "Option::is_none"`: the public struct tag
    /// carries `omitzero` so a nil member is omitted when re-encoded, as
    /// serde omits `None`.
    pub(crate) omit_none: bool,
    /// The wire name cannot be spelled in a `json` struct tag, so the member
    /// travels through the shadow's embedded fallback on decode and the
    /// model's own `MarshalJSONTo` on encode. See [`tag_spellable`].
    pub(crate) raw_key: bool,
}

impl GoField {
    /// Whether the JSON key must be present, as serde enforces it.
    pub(crate) fn required_key(&self) -> bool {
        match self.codec {
            Codec::Plain | Codec::Number => !self.optional,
            Codec::Embedded => false,
            Codec::RequiredOption
            | Codec::RequiredNumber
            | Codec::Count
            | Codec::EmptyOrNullString
            | Codec::EmptyDate
            | Codec::EmptyOrNullDate
            | Codec::DynamicJson
            | Codec::DynamicObject => true,
        }
    }
}

/// The names an emitter resolves against: every discovered response struct
/// and the public type aliases of the crate.
pub(crate) struct TypeTable<'a> {
    structs: BTreeSet<&'a str>,
    aliases: &'a BTreeMap<String, String>,
}

impl<'a> TypeTable<'a> {
    pub(crate) fn new(structs: &'a [StructDef], aliases: &'a BTreeMap<String, String>) -> Self {
        Self {
            structs: structs.iter().map(|def| def.name.as_str()).collect(),
            aliases,
        }
    }

    /// Maps one field of `def`, or explains why it cannot be mapped.
    pub(crate) fn go_field(&self, def: &StructDef, field: &FieldDef) -> Result<GoField, String> {
        let label = format!("{}.{}", def.name, field.name);
        let fail = |message: String| format!("{label}: {message}");
        if field.attrs.skip {
            return Err(fail("serde skip is not supported by gen_go".to_string()));
        }
        let (wraps, ident) = peel(&field.ty);
        let ident = ident.ok_or_else(|| fail("unsupported type shape".to_string()))?;
        let embedded = field.attrs.flatten || def.custom_deserialize;
        if embedded && ident == "DynamicObject" && wraps.is_empty() {
            return Ok(GoField {
                name: exported(&field.name),
                wire: String::new(),
                public_ty: "jsontext.Value".to_string(),
                shadow_ty: "jsontext.Value".to_string(),
                optional: false,
                codec: Codec::Embedded,
                omit_none: false,
                raw_key: false,
            });
        }
        if field.attrs.flatten {
            return Err(fail(
                "serde flatten is supported by gen_go only on a bare DynamicObject".to_string(),
            ));
        }
        if wraps
            .windows(2)
            .any(|pair| pair == [Wrap::Option, Wrap::Option])
        {
            return Err(fail("nested Option<Option<_>> has no Go shape".to_string()));
        }
        let optional = wraps.first() == Some(&Wrap::Option);
        let codec_attr = field
            .attrs
            .with
            .as_deref()
            .or(field.attrs.deserialize_with.as_deref());
        if field.attrs.default && (!optional || codec_attr.is_some()) {
            return Err(fail(
                "serde default is only supported on a plain Option<_> field, where a missing \
                 key is None with or without the attribute; any other default has no Go shape"
                    .to_string(),
            ));
        }
        let omit_none = match field.attrs.skip_serializing_if.as_deref() {
            None => false,
            Some("Option::is_none") if optional => true,
            Some("Option::is_none") => {
                return Err(fail(
                    "skip_serializing_if = \"Option::is_none\" needs an Option<_> field"
                        .to_string(),
                ));
            }
            Some(predicate) => {
                return Err(fail(format!(
                    "skip_serializing_if `{predicate}` is not in the gen_go table"
                )));
            }
        };
        let base = self.base(&ident, 0).map_err(fail)?;
        let name = exported(&field.name);
        let wire = field.wire_name(def.rename_all.as_deref());
        let raw_key = !tag_spellable(&wire);
        let public_ty = wrap(&wraps, &base.go);

        let (codec, shadow_ty) = match (codec_attr, base.kind, wraps.as_slice()) {
            (
                Some("required_option"),
                BaseKind::Scalar | BaseKind::DynamicJson,
                [Wrap::Option, ..],
            ) => (Codec::RequiredOption, "jsontext.Value".to_string()),
            (Some("required_option"), BaseKind::Number, [Wrap::Option]) => {
                (Codec::RequiredNumber, "jsontext.Value".to_string())
            }
            (Some(COUNT_CODEC), BaseKind::Scalar, []) if base.go == "uint64" => {
                (Codec::Count, "jsontext.Value".to_string())
            }
            (Some(COUNT_CODEC), _, _) => {
                return Err(fail(
                    "the count codec needs a bare Count (u64) field; Option<Count> has no Go \
                     shape yet"
                        .to_string(),
                ));
            }
            (Some(EMPTY_OR_NULL_CODEC), BaseKind::Scalar, [Wrap::Option])
                if base.go == "string" =>
            {
                (Codec::EmptyOrNullString, "jsontext.Value".to_string())
            }
            (Some(EMPTY_OR_NULL_CODEC), _, _) => {
                return Err(fail(
                    "the empty_or_null codec needs an Option of a string-backed type".to_string(),
                ));
            }
            (Some("required_option"), _, _) => {
                return Err(fail(
                    "required_option needs an Option<scalar, Number, or DynamicJson> field"
                        .to_string(),
                ));
            }
            (
                Some(codec @ ("empty_date" | "empty_or_null_date")),
                BaseKind::Scalar,
                [Wrap::Option],
            ) if base.go == "Date" => {
                let codec = if codec == "empty_date" {
                    Codec::EmptyDate
                } else {
                    Codec::EmptyOrNullDate
                };
                (codec, "jsontext.Value".to_string())
            }
            (Some(codec), _, _) => {
                return Err(fail(format!(
                    "serde codec `{codec}` is not in the gen_go table"
                )));
            }
            (None, BaseKind::DynamicJson, []) => (Codec::DynamicJson, public_ty.clone()),
            (None, BaseKind::DynamicObject, []) => (Codec::DynamicObject, format!("*{public_ty}")),
            (None, BaseKind::DynamicObject, [Wrap::Option]) => {
                (Codec::DynamicObject, public_ty.clone())
            }
            (None, BaseKind::DynamicObject, _) => {
                return Err(fail(
                    "DynamicObject inside Vec has no per-element object check".to_string(),
                ));
            }
            (None, BaseKind::Number, []) => (Codec::Number, format!("*{public_ty}")),
            (None, BaseKind::Number, [Wrap::Option]) => (Codec::Number, public_ty.clone()),
            (None, BaseKind::Number, _) => {
                return Err(fail(
                    "Number inside Vec has no per-element number check".to_string(),
                ));
            }
            (None, _, _) if optional => (Codec::Plain, public_ty.clone()),
            (None, _, _) => (Codec::Plain, format!("*{public_ty}")),
        };
        if raw_key && codec != Codec::Plain {
            return Err(fail(format!(
                "wire name `{wire}` cannot be spelled in a json struct tag and only a plain \
                 field is bridged through the embedded fallback; codec {codec:?} has no Go shape"
            )));
        }
        Ok(GoField {
            name,
            wire,
            public_ty,
            shadow_ty,
            optional,
            codec,
            omit_none,
            raw_key,
        })
    }

    fn base(&self, ident: &str, depth: usize) -> Result<Base, String> {
        if depth > 8 {
            return Err(format!("alias cycle at `{ident}`"));
        }
        if ident == "FiscalYear" {
            return Err(
                "FiscalYear (integer or digit-only string) has no Go codec; \
                 ADR 0002 says no public model uses it, so this needs a decision"
                    .to_string(),
            );
        }
        if let Some(go) = scalar(ident) {
            return Ok(Base::scalar(go));
        }
        match ident {
            "DynamicJson" => {
                return Ok(Base {
                    go: "jsontext.Value".to_string(),
                    kind: BaseKind::DynamicJson,
                });
            }
            "DynamicObject" => {
                return Ok(Base {
                    go: "jsontext.Value".to_string(),
                    kind: BaseKind::DynamicObject,
                });
            }
            "Number" => {
                return Ok(Base {
                    go: "jsontext.Value".to_string(),
                    kind: BaseKind::Number,
                });
            }
            _ => {}
        }
        if self.structs.contains(ident) {
            return Ok(Base::scalar(&go_name(ident)));
        }
        if let Some(target) = self.aliases.get(ident) {
            return self.base(target, depth + 1);
        }
        Err(format!(
            "unmapped Rust type `{ident}`; add it to gen_go/types.rs"
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BaseKind {
    Scalar,
    DynamicJson,
    DynamicObject,
    Number,
}

struct Base {
    go: String,
    kind: BaseKind,
}

impl Base {
    fn scalar(go: &str) -> Self {
        Self {
            go: go.to_string(),
            kind: BaseKind::Scalar,
        }
    }
}

/// The ADR 0030 table for types with a direct Go spelling.
fn scalar(ident: &str) -> Option<&'static str> {
    Some(match ident {
        "f64" => "float64",
        "f32" => "float32",
        "u64" => "uint64",
        "i64" => "int64",
        "u32" => "uint32",
        "i32" => "int32",
        "u16" => "uint16",
        "i16" => "int16",
        "u8" => "uint8",
        "i8" => "int8",
        "bool" | "WireBool" => "bool",
        "String" | "NumericString" | "PercentString" | "FiscalYearString" | "IsoTimestamp"
        | "DateOrDateTime" | "OpaqueDateText" => "string",
        "SecretUrl" => "string",
        "Ticker"
        | "TipRanksExpertUid"
        | "BulkPart"
        | "SearchTerm"
        | "Cik"
        | "CongressionalMemberId"
        | "FormType"
        | "TransactionTypeCode"
        | "Cusip"
        | "Isin"
        | "Lei"
        | "ExchangeCode"
        | "MarketHoursTimestamp"
        | "CurrencyCode"
        | "CountryCode"
        | "Sector"
        | "Industry"
        | "BenchmarkYear" => "string",
        "YnFlag" | "YesNoFlag" | "TrueFalseFlag" | "TitleCaseBoolFlag" => "string",
        "FiscalPeriod" | "RetrievalFrequency" | "StatementPeriod" | "Quarter"
        | "ChartTimeframe" | "EconomicIndicator" => "string",
        "NumberOrNumericString" | "PercentageValue" => "NumberOrString",
        "Date" => "Date",
        "UsDate" => "USDate",
        "ApiDateTime" => "DateTime",
        "UnixSeconds" => "UnixSeconds",
        "UnixMilliseconds" => "UnixMilliseconds",
        "CalendarYear" | "PeriodLength" | "Page" | "Limit" | "Year" => "uint32",
        "CalendarQuarter" => "uint8",
        "FiniteDecimal" => "float64",
        _ => return None,
    })
}

/// Whether `encoding/json/v2` accepts `wire` as a struct tag name. Its tag
/// parser (`fields.go`, Go 1.27) reserves the comma, the backslash, and the
/// three quote characters, and rejects the single-quoted spelling at the
/// name position, so such a name has no tag form at all and the member is
/// bridged through an embedded fallback instead.
pub(crate) fn tag_spellable(wire: &str) -> bool {
    !wire.is_empty()
        && !wire
            .chars()
            .any(|c| matches!(c, ',' | '\\' | '\'' | '"' | '`'))
}

fn wrap(wraps: &[Wrap], base: &str) -> String {
    let mut out = String::new();
    for wrap in wraps {
        out.push_str(match wrap {
            Wrap::Option => "*",
            Wrap::Vec => "[]",
        });
    }
    out.push_str(base);
    out
}

/// The Go parameter type and the `query.go` helper for a registry arg kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ArgGo {
    pub(crate) go_type: &'static str,
    /// The `sdk/go/query.go` function `(name string, value T) (queryParam, error)`.
    pub(crate) helper: &'static str,
}

/// Maps a registry kind to its Go spelling, or explains which helper the
/// hand-written `query.go` still lacks.
pub(crate) fn arg_kind_go(kind: ArgKind) -> Result<ArgGo, String> {
    let (go_type, helper) = match kind {
        ArgKind::Ticker => ("string", "tickerParam"),
        ArgKind::TickerList => ("[]string", "tickerListParam"),
        ArgKind::SearchTerm
        | ArgKind::Cik
        | ArgKind::ExchangeCode
        | ArgKind::CongressionalMemberId
        | ArgKind::TipranksExpertUid
        | ArgKind::MarketHoursTimestamp
        | ArgKind::BenchmarkYear
        | ArgKind::TransactionTypeCode
        | ArgKind::Sector
        | ArgKind::Industry
        | ArgKind::CountryCode
        | ArgKind::CurrencyCode
        | ArgKind::BulkPart
        | ArgKind::Cusip
        | ArgKind::Isin
        | ArgKind::Lei
        | ArgKind::FormType => ("string", "stringParam"),
        ArgKind::Text => ("string", "textParam"),
        ArgKind::Limit | ArgKind::Page | ArgKind::Year | ArgKind::CalendarYear => {
            ("uint32", "uint32Param")
        }
        ArgKind::PeriodLength => ("uint32", "periodLengthParam"),
        ArgKind::CalendarQuarter => ("uint8", "calendarQuarterParam"),
        ArgKind::MarketCapitalization | ArgKind::Volume => ("uint64", "uint64Param"),
        ArgKind::FiniteDecimal => ("float64", "finiteDecimalParam"),
        ArgKind::Boolean | ArgKind::TrueFalseFlag => ("bool", "boolParam"),
        ArgKind::Date => ("Date", "dateParam"),
        ArgKind::ApiDatetime => ("DateTime", "dateTimeParam"),
        ArgKind::RetrievalFrequency => ("RetrievalFrequency", "retrievalFrequencyParam"),
        ArgKind::FiscalPeriod => ("FiscalPeriod", "fiscalPeriodParam"),
        ArgKind::StatementPeriod => ("StatementPeriod", "statementPeriodParam"),
        ArgKind::SegmentationStructure => ("SegmentationStructure", "segmentationStructureParam"),
        ArgKind::EconomicIndicator => ("EconomicIndicator", "economicIndicatorParam"),
        ArgKind::Quarter => ("Quarter", "quarterParam"),
        ArgKind::ChartTimeframe => ("ChartTimeframe", "chartTimeframeParam"),
        ArgKind::DateRange | ArgKind::OpenEconomicIndicator => {
            return Err(format!(
                "arg kind `{kind}` has no Go helper yet: add `{}Param` to sdk/go/query.go \
                 (mirroring the libfmp `{}` validation) and its row to gen_go/types.rs",
                crate::emit::lower_first(&exported(kind.name())),
                kind.libfmp_type()
            ));
        }
    };
    Ok(ArgGo { go_type, helper })
}
