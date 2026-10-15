use crate::internal::utils;

use super::flags;

/// SQL dialect: quoting, literals, and which syntax is legal.
///
/// Use a builtin such as `PostgreSql`, or build one from `OPT_*` flags.
#[rapidquery_macros::pyclass("rapidquery.dialect.Dialect", skip_from_py_object)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyDialect(pub flags::DialectFlags);

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

// One item is one constructor argument and one getter.
// Enum items also export `OPT_*` constants.
// Adding a flag is a new item here, plus the enum and the bitfield field.
rapidquery_macros::dialect_options! {
    char placeholder_character = '?';

    enum Placeholder placeholder {
        Simple => OPT_PLACEHOLDER_SIMPLE,
        Numbered => OPT_PLACEHOLDER_NUMBERED,
    }

    enum NullTreatment null_treatment {
        Unsupported => OPT_NULL_TREATMENT_UNSUPPORTED,
        InsideParens => OPT_NULL_TREATMENT_INSIDE_PARENS,
        Postfix => OPT_NULL_TREATMENT_POSTFIX,
    }

    enum LikeEscape like_escape {
        Unsupported => OPT_LIKE_ESCAPE_UNSUPPORTED,
        Scalar => OPT_LIKE_ESCAPE_SCALAR,
        IncludingAny => OPT_LIKE_ESCAPE_INCLUDING_ANY,
    }

    enum LikeAny like_any {
        OrExpansion => OPT_LIKE_ANY_OR_EXPANSION,
        ArrayConstructor => OPT_LIKE_ANY_ARRAY_CONSTRUCTOR,
        PatternList => OPT_LIKE_ANY_PATTERN_LIST,
    }

    enum ILikeExpression ilike_expression {
        Standard => OPT_ILIKE_EXPRESSION_STANDARD,
        UseLikeLower => OPT_ILIKE_EXPRESSION_USE_LIKE_LOWER,
        UseLike => OPT_ILIKE_EXPRESSION_USE_LIKE,
    }

    enum RLikeExpression rlike_expression {
        Unsupported => OPT_RLIKE_EXPRESSION_UNSUPPORTED,
        Standard => OPT_RLIKE_EXPRESSION_STANDARD,
        Tilde => OPT_RLIKE_EXPRESSION_TILDE,
        RegexpLike => OPT_RLIKE_EXPRESSION_REGEXP_LIKE,
        RegexpContains => OPT_RLIKE_EXPRESSION_REGEXP_CONTAINS,
        RegexpMatches => OPT_RLIKE_EXPRESSION_REGEXP_MATCHES,
    }

    enum StringConcat string_concat {
        Pipes => OPT_STRING_CONCAT_PIPES,
        Plus => OPT_STRING_CONCAT_PLUS,
        ConcatFunction => OPT_STRING_CONCAT_CONCAT_FUNCTION,
    }

    enum BitXor bitxor {
        Unsupported => OPT_BITXOR_UNSUPPORTED,
        Caret => OPT_BITXOR_CARET,
        Hash => OPT_BITXOR_HASH,
        XORFunction => OPT_BITXOR_XOR_FUNCTION,
        BitXORFunction => OPT_BITXOR_BIT_XOR_FUNCTION,
    }

    enum Modulo modulo {
        Percent => OPT_MODULO_PERCENT,
        Function => OPT_MODULO_FUNCTION,
        Infix => OPT_MODULO_INFIX,
    }

    enum IsDistinctFrom is_distinct_from {
        Unsupported => OPT_IS_DISTINCT_FROM_UNSUPPORTED,
        Standard => OPT_IS_DISTINCT_FROM_STANDARD,
        NullSafeEqual => OPT_IS_DISTINCT_FROM_NULL_SAFE_EQUAL,
    }

    enum TryCast try_cast {
        Unsupported => OPT_TRY_CAST_UNSUPPORTED,
        TryCast => OPT_TRY_CAST_TRY_CAST,
        TryCastWord => OPT_TRY_CAST_TRY_CAST_WORD,
        SafeCast => OPT_TRY_CAST_SAFE_CAST,
    }

    enum Trim trim {
        SqlFrom => OPT_TRIM_SQL_FROM,
        Function => OPT_TRIM_FUNCTION,
        SpacesOnly => OPT_TRIM_SPACES_ONLY,
    }

    enum Collate collate {
        Unsupported => OPT_COLLATE_UNSUPPORTED,
        InfixIdent => OPT_COLLATE_INFIX_IDENT,
        FunctionString => OPT_COLLATE_FUNCTION_STRING,
    }

    enum Bitwise bitwise {
        Infix => OPT_BITWISE_INFIX,
        InfixAndShift => OPT_BITWISE_INFIX_AND_SHIFT,
        InfixNoShift => OPT_BITWISE_INFIX_NO_SHIFT,
        BitShiftFunctions => OPT_BITWISE_BIT_SHIFT_FUNCTIONS,
        ShiftFunctions => OPT_BITWISE_SHIFT_FUNCTIONS,
        BitAndOnly => OPT_BITWISE_BIT_AND_ONLY,
    }

    enum Tuple tuple {
        Unsupported => OPT_TUPLE_UNSUPPORTED,
        Row => OPT_TUPLE_ROW,
        RowRejectEmpty => OPT_TUPLE_ROW_REJECT_EMPTY,
        Struct => OPT_TUPLE_STRUCT,
        Parentheses => OPT_TUPLE_PARENTHESES,
    }

    enum BitFormat bit_format {
        Unsupported => OPT_BIT_FORMAT_UNSUPPORTED,
        PrefixBigB => OPT_BIT_FORMAT_PREFIX_BIG_B,
        PrefixSmallB => OPT_BIT_FORMAT_PREFIX_SMALL_B,
        PostfixBigB => OPT_BIT_FORMAT_POSTFIX_BIG_B,
    }

    enum BoolFormat bool_format {
        Keyword => OPT_BOOL_FORMAT_KEYWORD,
        Integer => OPT_BOOL_FORMAT_INTEGER,
    }

    enum IdentifierFormat identifier_format {
        DoubleQuote => OPT_IDENTIFIER_FORMAT_DOUBLE_QUOTE,
        Backtick => OPT_IDENTIFIER_FORMAT_BACKTICK,
        SquareBracket => OPT_IDENTIFIER_FORMAT_SQUARE_BRACKET,
    }

    enum IdentifierEscape identifier_escape {
        Double => OPT_IDENTIFIER_ESCAPE_DOUBLE,
        Backslash => OPT_IDENTIFIER_ESCAPE_BACKSLASH,
    }

    enum StringFormat string_format {
        SingleQuote => OPT_STRING_FORMAT_SINGLE_QUOTE,
        PrefixBigE => OPT_STRING_FORMAT_PREFIX_BIG_E,
        PrefixBigN => OPT_STRING_FORMAT_PREFIX_BIG_N,
    }

    enum StringEscape string_escape {
        Quote => OPT_STRING_ESCAPE_QUOTE,
        BackslashHex => OPT_STRING_ESCAPE_BACKSLASH_HEX,
        BackslashMysql => OPT_STRING_ESCAPE_BACKSLASH_MYSQL,
        BackslashUnicode => OPT_STRING_ESCAPE_BACKSLASH_UNICODE,
        BackslashHexZero => OPT_STRING_ESCAPE_BACKSLASH_HEX_ZERO,
        BackslashUnicodeQuote => OPT_STRING_ESCAPE_BACKSLASH_UNICODE_QUOTE,
    }

    enum BytesFormat bytes_format {
        PrefixBigX => OPT_BYTES_FORMAT_PREFIX_BIG_X,
        PrefixZeroX => OPT_BYTES_FORMAT_PREFIX_ZERO_X,
        SlashXBytea => OPT_BYTES_FORMAT_SLASH_X_BYTEA,
        PrefixBigBSlashX => OPT_BYTES_FORMAT_PREFIX_BIG_B_SLASH_X,
        HexToRaw => OPT_BYTES_FORMAT_HEX_TO_RAW,
        Unhex => OPT_BYTES_FORMAT_UNHEX,
        ToVarbyte => OPT_BYTES_FORMAT_TO_VARBYTE,
        PostfixXbv => OPT_BYTES_FORMAT_POSTFIX_XBV,
        ToBinary => OPT_BYTES_FORMAT_TO_BINARY,
        SlashXBlob => OPT_BYTES_FORMAT_SLASH_X_BLOB,
    }

    enum FunctionFilter function_filter {
        Unsupported => OPT_FUNCTION_FILTER_UNSUPPORTED,
        Filter => OPT_FUNCTION_FILTER_FILTER,
        FilterWhere => OPT_FUNCTION_FILTER_FILTER_WHERE,
        InsideWhere => OPT_FUNCTION_FILTER_INSIDE_WHERE,
    }

    enum ArrayFormat array_format {
        Unsupported => OPT_ARRAY_FORMAT_UNSUPPORTED,
        Constructor => OPT_ARRAY_FORMAT_CONSTRUCTOR,
        List => OPT_ARRAY_FORMAT_LIST,
        Function => OPT_ARRAY_FORMAT_FUNCTION,
    }

    enum DateFormat date_format {
        Quoted => OPT_DATE_FORMAT_QUOTED,
        Keyword => OPT_DATE_FORMAT_KEYWORD,
    }

    enum TimeFormat time_format {
        Quoted => OPT_TIME_FORMAT_QUOTED,
        Keyword => OPT_TIME_FORMAT_KEYWORD,
    }

    enum TimestampFormat timestamp_format {
        Quoted => OPT_TIMESTAMP_FORMAT_QUOTED,
        Timestamptz => OPT_TIMESTAMP_FORMAT_TIMESTAMPTZ,
        WithTimezone => OPT_TIMESTAMP_FORMAT_WITH_TIMEZONE,
    }

    enum InetFormat inet_format {
        Unsupported => OPT_INET_FORMAT_UNSUPPORTED,
        Quoted => OPT_INET_FORMAT_QUOTED,
        Cast => OPT_INET_FORMAT_CAST,
    }

    enum MacAddrFormat macaddr_format {
        Unsupported => OPT_MACADDR_FORMAT_UNSUPPORTED,
        Quoted => OPT_MACADDR_FORMAT_QUOTED,
        Cast => OPT_MACADDR_FORMAT_CAST,
    }

    enum UuidFormat uuid_format {
        Quoted => OPT_UUID_FORMAT_QUOTED,
        UuidKeyword => OPT_UUID_FORMAT_UUID_KEYWORD,
        CastUuid => OPT_UUID_FORMAT_CAST_UUID,
    }

    enum JsonFormat json_format {
        Quoted => OPT_JSON_FORMAT_QUOTED,
        JsonKeyword => OPT_JSON_FORMAT_JSON_KEYWORD,
        ParseJson => OPT_JSON_FORMAT_PARSE_JSON,
    }

    bool supports_similar_to = false;
    bool parenthesize_in_between = false;
    bool supports_double_colon = false;
}

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

    fn __eq__(&self, other: &pyo3::Bound<'_, Self>) -> bool {
        self.0 == other.get().0
    }

    fn __hash__(&self) -> u64 {
        use std::hash::Hash;
        use std::hash::Hasher;

        let mut hasher = std::hash::DefaultHasher::default();
        self.0.hash(&mut hasher);
        hasher.finish()
    }
}
