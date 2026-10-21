use crate::internal::utils;

use super::config::*;

/// SQL dialect: quoting, literals, and which syntax is legal.
///
/// Use a builtin such as `PostgreSql`, or build one from `OPT_*` flags.
#[rapidquery_macros::pyclass("rapidquery.dialect.Dialect", eq, hash)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct PyDialect(pub DialectConfig);

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for PyDialect {
    type Error = pyo3::PyErr;

    fn extract(obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>) -> Result<Self, Self::Error> {
        unsafe {
            // SAFETY: `obj` is a borrowed `&Bound<PyAny>`, so `as_ptr()` is a
            // live `PyObject*`.
            if crate::typeref::check_exact(obj.as_ptr(), crate::typeref::DIALECT_TYPE) {
                let cast = obj.cast_unchecked::<Self>();
                return Ok(Self(cast.get().0));
            }

            rapidquery_macros::pyerr!(
                Err(PyTypeError),
                "expected Dialect, got {}",
                utils::get_type_name(obj.py(), obj.as_ptr())
            )
        }
    }
}

// FIXME: Set dialect default values as disabled/unsupported when possible,
//        otherwise set a traditional one. So developer can predict the values
//        without needing to specify all arguments.
rapidquery_macros::pydialect_constructor!(
    placeholder_character: char = '?',

    #[enum_(Simple, Numbered)]
    placeholder_syntax: PlaceholderSyntax = PlaceholderSyntax::default(),

    #[bit(SCALAR, ANY)]
    like_escape: LikeEscape = LikeEscape::SCALAR.bits(),

    #[enum_(OrExpansion, ArrayConstructor, PatternList)]
    like_any: LikeAny = LikeAny::default(),

    #[enum_(Standard, UseLikeLower, UseLike)]
    ilike_expression: ILikeExpression = ILikeExpression::default(),

    #[enum_(Unsupported, Standard, Tilde, RegexpLike, RegexpContains, RegexpMatches)]
    rlike_expression: RLikeExpression = RLikeExpression::default(),

    supports_similar_to: bool = false,

    #[enum_(Unsupported, InsideParens, Postfix)]
    null_treatment: NullTreatment = NullTreatment::default(),

    #[enum_(Unsupported, Filter, FilterWhere, InsideWhere)]
    function_filter: FunctionFilter = FunctionFilter::default(),

    #[bit(AND, OR, NOT, SHL, SHR)]
    bitwise_operations: BitwiseOperations = BitwiseOperations::AND
        .union(BitwiseOperations::OR)
        .union(BitwiseOperations::NOT)
        .union(BitwiseOperations::SHL)
        .union(BitwiseOperations::SHR)
        .bits(),

    #[enum_(Infix, BitFunction, ShiftFunction)]
    bitwise_call_syntax: BitwiseCallSyntax = BitwiseCallSyntax::default(),

    #[enum_(Unsupported, Caret, Hash, Function, BitFunction)]
    bitwise_xor_spelling: BitwiseXorSpelling = BitwiseXorSpelling::default(),

    #[enum_(DoubleQuote, Backtick, SquareBracket)]
    identifier_format: IdentifierFormat = IdentifierFormat::default(),

    #[enum_(Double, Backslash)]
    identifier_escape: IdentifierEscape = IdentifierEscape::default(),

    #[enum_(SingleQuote, PrefixBigE, PrefixBigN)]
    string_format: StringFormat = StringFormat::default(),

    #[enum_(Quote, BackslashHex, BackslashMysql, BackslashUnicode, BackslashHexZero, BackslashUnicodeQuote)]
    string_escape: StringEscape = StringEscape::default(),

    #[enum_(Pipes, Plus, ConcatFunction)]
    string_concat: StringConcat = StringConcat::default(),

    #[enum_(Unsupported, TryCast, TryCastWord, SafeCast)]
    cast_try_cast: TryCast = TryCast::default(),

    cast_double_colon: bool = false,

    #[enum_(Unsupported, Row, Struct, Parentheses)]
    tuple_syntax: TupleSyntax = TupleSyntax::default(),

    #[bit(EMPTY, ONE)]
    tuple_arity: TupleArity = TupleArity::EMPTY.union(TupleArity::ONE).bits(),

    #[enum_(Quoted, Keyword)]
    date_format: DateFormat = DateFormat::default(),

    #[enum_(Quoted, Keyword)]
    time_format: TimeFormat = TimeFormat::default(),

    #[enum_(Quoted, Timestamptz, WithTimezone)]
    timestamp_format: TimestampFormat = TimestampFormat::default(),

    #[enum_(Keyword, Integer)]
    bool_format: BoolFormat = BoolFormat::default(),

    #[enum_(Unsupported, PrefixBigB, PrefixSmallB, PostfixBigB)]
    bit_format: BitFormat = BitFormat::default(),

    #[enum_(PrefixBigX, PrefixZeroX, SlashXBytea, PrefixBigBSlashX, HexToRaw, Unhex, ToVarbyte, PostfixXbv, ToBinary, SlashXBlob)]
    bytes_format: BytesFormat = BytesFormat::default(),

    #[enum_(Unsupported, Quoted, Cast)]
    inet_format: InetFormat = InetFormat::default(),

    #[enum_(Unsupported, Quoted, Cast)]
    macaddr_format: MacAddrFormat = MacAddrFormat::default(),

    #[enum_(Quoted, UuidKeyword, CastUuid)]
    uuid_format: UuidFormat = UuidFormat::default(),

    #[enum_(Quoted, JsonKeyword, ParseJson)]
    json_format: JsonFormat = JsonFormat::default(),

    #[enum_(Unsupported, Constructor, List, Function)]
    array_format: ArrayFormat = ArrayFormat::default(),

    #[enum_(Percent, Function, Infix)]
    modulo_spelling: ModuloSpelling = ModuloSpelling::default(),

    #[enum_(Unsupported, Standard, NullSafeEqual)]
    is_distinct_from_spelling: IsDistinctFromSpelling = IsDistinctFromSpelling::default(),

    #[enum_(SqlFrom, Function, SpacesOnly)]
    trim_spelling: TrimSpelling = TrimSpelling::default(),

    #[enum_(Unsupported, InfixIdent, FunctionString)]
    collate_spelling: CollateSpelling = CollateSpelling::default(),

    #[bit(UNSUPPORTED, IF_NOT_EXISTS, EMBEDDED_DDL)]
    schema_features: SchemaFeatures = SchemaFeatures::empty().bits(),

    #[enum_(Unsupported, Forced, Optional)]
    schema_authorization_style: SchemaAuthorizationStyle = SchemaAuthorizationStyle::default(),

    #[enum_(Unsupported, LocationKeyword, ManagedLocationKeyword, ExternalVolume)]
    schema_location_style: SchemaLocationStyle = SchemaLocationStyle::default(),
);

#[pyo3::pymethods]
impl PyDialect {
    /// Write a boolean literal (`TRUE`/`FALSE` or `1`/`0`).
    fn write_boolean(&self, value: bool) -> pyo3::PyResult<String> {
        let mut buf = String::new();
        self.0.write_boolean(&mut buf, value)?;
        Ok(buf)
    }

    /// Escape `ident` and wrap it in this dialect’s identifier delimiters.
    fn write_identifier(&self, value: &str) -> pyo3::PyResult<String> {
        let mut buf = String::new();
        let _ = self.0.write_identifier(&mut buf, value);
        Ok(buf)
    }

    /// Escape `value` and wrap it as a complete string literal.
    fn write_string_literal(&self, value: &str) -> pyo3::PyResult<String> {
        let mut buf = String::new();
        self.0.write_string_literal(&mut buf, value)?;
        Ok(buf)
    }

    /// Write a bit-string ('0'/'1' characters) as a native literal.
    fn write_bits(&self, value: &str) -> pyo3::PyResult<String> {
        if value.chars().any(|c| c != '0' && c != '1') {
            return rapidquery_macros::pyerr!(Err(PyValueError), "invalid input: {}", value);
        }

        let mut buf = String::new();
        self.0.write_bits(&mut buf, value)?;
        Ok(buf)
    }

    /// Write bytes as a native blob/bytea/varbyte literal.
    fn write_bytes(&self, value: &[u8]) -> pyo3::PyResult<String> {
        let mut buf = String::new();
        self.0.write_bytes(&mut buf, value)?;
        Ok(buf)
    }
}
