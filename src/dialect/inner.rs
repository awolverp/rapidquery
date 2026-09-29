use std::fmt::{self, Write};

use crate::internal::utils;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatError {
    Write,
    NulInString,
    Unsupported(&'static str),
}

impl From<fmt::Error> for FormatError {
    #[inline]
    fn from(_: fmt::Error) -> Self {
        Self::Write
    }
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Write => f.write_str("failed to write SQL"),
            Self::NulInString => f.write_str("NUL is not allowed in a string literal"),
            Self::Unsupported(name) => write!(f, "this dialect does not support {name}"),
        }
    }
}

impl std::error::Error for FormatError {}

/// Positional parameter syntax
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PositionalParameter {
    /// e.g. `'?'`
    Simple(char),
    /// e.g. `'$'` -> `'$1'`
    Numbered(char),
}

/// The way dialect accepts literal bit values
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BitFormat {
    /// e.g. `B'101'`
    PrefixBigB,
    /// e.g. `b'101'`
    PrefixSmallB,
    /// e.g. `'101'B`
    PostfixBigB,
}

impl BitFormat {
    /// Write `bits` as a native bit-string literal.
    #[inline]
    fn write<W: fmt::Write>(self, writer: &mut W, bits: &str) -> fmt::Result {
        match self {
            Self::PrefixBigB => write!(writer, "B'{bits}'"),
            Self::PrefixSmallB => write!(writer, "b'{bits}'"),
            Self::PostfixBigB => write!(writer, "'{bits}'B"),
        }
    }
}

/// The way dialect supports `{IGNORE|RESPECT} NULLS` expressions in `Function`s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NullTreatmentSyntax {
    /// e.g. `FIRST_VALUE(x IGNORE NULLS)`
    InsideParens,
    /// e.g. `FIRST_VALUE(x) IGNORE NULLS`
    Postfix,
}

/// How `LIKE ANY` / `ILIKE ANY` is rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LikeAnySyntax {
    /// `LIKE ANY (ARRAY[p, …])`.
    ArrayConstructor,
    /// `LIKE ANY (p, …)`.
    PatternList,
    /// `(x LIKE p1 OR x LIKE p2)`. Useful when it isn't supported natively by dialect
    OrExpansion,
}

/// Does dialect support `ILIKE` expression or we should fallback to another options?
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ILikeExpressionOptions {
    /// `ILIKE` is supported
    Supported,
    /// Use `LIKE` expression and `LOWER()` function
    UseLikeLower,
    /// Fallback to `LIKE` expression
    UseLike,
}

impl ILikeExpressionOptions {
    /// `(use_ilike, wrap_lower)` when the LIKE is case-insensitive.
    #[inline]
    pub fn case_insensitive_sql(self) -> (bool, bool) {
        match self {
            Self::Supported => (true, false),
            Self::UseLike => (false, false),
            Self::UseLikeLower => (false, true),
        }
    }
}

/// How regex `RLike` is spelled.
///
/// [`None`] on [`Dialect::rlike_expression_options`] means unsupported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RLikeExpressionOptions {
    /// `expr [NOT] RLIKE pattern` — MySQL, Hive, Spark, Databricks, Snowflake.
    InfixRLike,
    /// `expr ~ pattern` / `expr !~ pattern` — PostgreSQL, Redshift.
    Tilde,
    /// `REGEXP_LIKE(expr, pattern)` — Oracle.
    RegexpLike,
    /// `REGEXP_CONTAINS(expr, pattern)` — BigQuery.
    RegexpContains,
    /// `regexp_matches(expr, pattern)` — DuckDB (substring; DuckDB `~` is full match).
    RegexpMatches,
}

/// How [`crate::expr::enum_::BinaryOp::StringConcat`] (`||`) is spelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StringConcatSyntax {
    /// `left || right`.
    Pipes,
    /// `left + right`. SQL Server `||` is 2025+; classic concat is `+`.
    Plus,
    /// `CONCAT(left, right)`. MySQL `||` is logical OR unless `PIPES_AS_CONCAT`.
    ConcatFunction,
}

/// How [`crate::expr::enum_::BinaryOp::BitXor`] (`^`) is spelled.
///
/// [`None`] on [`Dialect::bit_xor_syntax`] means unsupported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BitXorSyntax {
    /// `left ^ right`.
    Caret,
    /// `left # right`. PostgreSQL `^` is exponent.
    Hash,
    /// `xor(left, right)`.
    XORFunction,
    /// `BITXOR(left, right)`.
    BitXORFunction,
}

/// How [`crate::expr::enum_::BinaryOp::Mod`] (`%`) is spelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModuloSyntax {
    /// `left % right`.
    Percent,
    /// `MOD(left, right)`.
    Function,
    /// `left MOD right`. Teradata's arithmetic operator; `MOD()` is an ODBC extension.
    Infix,
}

/// How `IS [NOT] DISTINCT FROM` is spelled.
///
/// [`None`] on [`Dialect::is_distinct_from_syntax`] means unsupported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IsDistinctFromSyntax {
    /// `left IS [NOT] DISTINCT FROM right`.
    Standard,
    /// `<=>` is null-safe equal (`IS NOT DISTINCT FROM`).
    /// `IS DISTINCT FROM` is `NOT (left <=> right)`.
    NullSafeEqual,
}

/// How `TRY_CAST` is spelled. [`None`] on [`Dialect::try_cast_syntax`] means unsupported.
///
/// Callers ask for `TRY_CAST`. BigQuery emits `SAFE_CAST`; Teradata emits `TRYCAST`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TryCastSyntax {
    /// `TRY_CAST(expr AS type)`.
    TryCast,
    /// `TRYCAST(expr AS type)`.
    TryCastWord,
    /// `SAFE_CAST(expr AS type)`. BigQuery's spelling of a null-on-failure cast.
    SafeCast,
}

impl TryCastSyntax {
    #[inline]
    pub fn as_sql(self) -> &'static str {
        match self {
            Self::TryCast => "TRY_CAST",
            Self::TryCastWord => "TRYCAST",
            Self::SafeCast => "SAFE_CAST",
        }
    }
}

/// How `TRIM` is spelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TrimSyntax {
    /// `TRIM([LEADING|TRAILING] [chars FROM] expr)`.
    SqlFrom,
    /// `TRIM` / `LTRIM` / `RTRIM` `(expr[, chars])`.
    Function,
    /// `TRIM` / `LTRIM` / `RTRIM` `(expr)`. A character set raises.
    SpacesOnly,
}

/// How `COLLATE` is spelled. [`None`] on [`Dialect::collate_syntax`] means unsupported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CollateSyntax {
    /// `expr COLLATE ident`.
    InfixIdent,
    /// `COLLATE(expr, 'body')`. The identifier body is a string literal.
    FunctionString,
}

/// How a tuple / row value is spelled.
///
/// [`None`] on [`Dialect::tuple_syntax`] means unsupported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TupleSyntax {
    /// `ROW(...)`. `empty_ok` allows arity 0 (`ROW()`).
    Row,
    /// `ROW(...)`. Rejects empty (`ROW()`).
    RowRejectEmpty,
    /// `struct(...)`. Arity 0 is unsupported.
    Struct,
    /// `(a, b, ...)`. Arity 0 and 1 are unsupported.
    Parentheses,
}

/// How a boolean literal is spelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BoolFormat {
    /// `TRUE` / `FALSE`
    Keyword,
    /// `1` / `0`
    Int,
}

/// Delimiters wrapped around a quoted identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IdentifierFormat {
    /// `"name"`
    DoubleQuote,
    /// `` `name` ``
    Backtick,
    /// `[name]`
    SquareBracket,
}

/// How to escape identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IdentifierEscape {
    /// Escape identifier delimiters with doubling.
    Double,
    /// Escape identifier delimiters with backslash.
    Backslash,
}

/// Delimiters wrapped around a literal string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StringFormat {
    /// `'string'`
    SingleQuote,
    /// `E'string'`
    PrefixBigE,
    /// `N'string'`
    PrefixBigN,
}

/// How to escape literal strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StringEscape {
    /// `'` → `''`. Other characters are unchanged.
    Quote,
    /// `\\`, `\'`, `\n`, `\r`, `\t`; other C0/DEL as `\xNN`.
    BackslashHex,
    /// MySQL: also `\"` and `\Z` for 0x1A; tab and other controls pass through.
    BackslashMysql,
    /// Hive: other C0/DEL as `\uNNNN`. No `\"`.
    BackslashUnicode,
    /// Snowflake: other C0/DEL as `\xNN`.
    BackslashHexZero,
    /// Spark/Databricks: [`Self::BackslashUnicode`] plus `\"`.
    BackslashUnicodeQuote,
}

impl StringEscape {
    /// Write `string` with special characters neutralized, without wrapping it.
    #[inline]
    fn write<W: fmt::Write>(self, writer: &mut W, string: &str) -> fmt::Result {
        match self {
            Self::Quote => utils::replace_to(string, '\'', "''", writer),
            Self::BackslashHex => write_backslash_hex(writer, string),
            Self::BackslashMysql => write_backslash_mysql(writer, string),
            Self::BackslashUnicode => write_backslash_unicode(writer, string, false),
            Self::BackslashHexZero => write_backslash_hex_zero(writer, string),
            Self::BackslashUnicodeQuote => write_backslash_unicode(writer, string, true),
        }
    }
}

/// The way dialect accepts literal bytes values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BytesFormat {
    /// `X'DEADBEEF'`
    PrefixBigX,
    /// `0xDEADBEEF`
    PrefixZeroX,
    /// `'\xDEADBEEF'::bytea`
    SlashXBytea,
    /// `B'\xNN…'` (each byte as `\xNN`)
    PrefixBigBSlashX,
    /// `HEXTORAW('DEADBEEF')`
    HexToRaw,
    /// `UNHEX('DEADBEEF')`
    Unhex,
    /// `TO_VARBYTE('DEADBEEF', 'hex')`
    ToVarbyte,
    /// `'DEADBEEF'XBV`
    PostfixXbv,
    /// `TO_BINARY('DEADBEEF', 'HEX')`
    ToBinary,
    /// `'\xHH…'::BLOB`; empty is `''::BLOB`
    SlashXBlob,
}

impl BytesFormat {
    /// Write `bytes` as a native blob/bytea/varbyte literal.
    #[inline]
    pub fn write<W: fmt::Write>(self, writer: &mut W, bytes: &[u8]) -> fmt::Result {
        match self {
            Self::PrefixBigX => {
                writer.write_str("X'")?;
                bytes_to_hex(writer, bytes)?;
                writer.write_char('\'')
            }
            Self::PrefixZeroX => {
                writer.write_str("0x")?;
                bytes_to_hex(writer, bytes)
            }
            Self::SlashXBytea => {
                writer.write_str("'\\x")?;
                bytes_to_hex(writer, bytes)?;
                writer.write_str("'::bytea")
            }
            Self::PrefixBigBSlashX => {
                writer.write_str("B'")?;
                for &b in bytes {
                    write_byte_hex_escape(writer, b)?;
                }
                writer.write_char('\'')
            }
            Self::HexToRaw => {
                writer.write_str("HEXTORAW('")?;
                bytes_to_hex(writer, bytes)?;
                writer.write_str("')")
            }
            Self::Unhex => {
                writer.write_str("UNHEX('")?;
                bytes_to_hex(writer, bytes)?;
                writer.write_str("')")
            }
            Self::ToVarbyte => {
                writer.write_str("TO_VARBYTE('")?;
                bytes_to_hex(writer, bytes)?;
                writer.write_str("', 'hex')")
            }
            Self::PostfixXbv => {
                writer.write_char('\'')?;
                bytes_to_hex(writer, bytes)?;
                writer.write_str("'XBV")
            }
            Self::ToBinary => {
                writer.write_str("TO_BINARY('")?;
                bytes_to_hex(writer, bytes)?;
                writer.write_str("', 'HEX')")
            }
            Self::SlashXBlob => {
                if bytes.is_empty() {
                    writer.write_str("''::BLOB")
                } else {
                    writer.write_char('\'')?;
                    for &b in bytes {
                        write_byte_hex_escape(writer, b)?;
                    }
                    writer.write_str("'::BLOB")
                }
            }
        }
    }
}

/// Function `FILTER` syntax
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FunctionFilterSyntax {
    /// e.g. FIRST_VALUE(x) FILTER (id > 10)
    Filter,
    /// e.g. FIRST_VALUE(x) FILTER (WHERE id > 10)
    FilterWhere,
    /// e.g. FIRST_VALUE(x WHERE id > 10)
    InsideWhere,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ArrayFormat {
    /// `ARRAY[…]`
    Constructor,
    /// `[…]`
    List,
    /// `array(…)`
    Function,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InetFormat {
    Quoted,
    /// `'…'::inet`
    Cast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MacFormat {
    Quoted,
    /// `'…'::macaddr`
    Cast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TemporalLiteral {
    Quoted,
    /// `DATE` / `TIME` / `TIMESTAMP` then a string literal.
    TypedKeyword,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JsonFormat {
    Quoted,
    /// `JSON '…'`
    JsonKeyword,
    /// `PARSE_JSON('…')`
    ParseJson,
    /// `'…'::json`
    CastJson,
    /// `'…'::jsonb`
    CastJsonb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UuidFormat {
    Quoted,
    /// `UUID '…'`
    UuidKeyword,
    /// `'…'::uuid`
    CastUuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dialect {
    /// Positional parameter syntax.
    pub positional_parameter: PositionalParameter,
    /// How the dialect accepts literal bit values if supported.
    ///
    /// Set [`Option::None`] if literal bit isn't supported.
    pub bit_format: Option<BitFormat>,
    pub array_format: Option<ArrayFormat>,
    pub inet_format: Option<InetFormat>,
    pub mac_format: Option<MacFormat>,
    /// How the dialect supports `{IGNORE|RESPECT} NULLS` expressions in `Function`s if supported.
    pub null_treatment_syntax: Option<NullTreatmentSyntax>,
    /// How `LIKE ANY` / `ILIKE ANY` is rendered.
    pub like_any_syntax: LikeAnySyntax,
    /// How the dialect accepts literal boolean values.
    pub bool_format: BoolFormat,
    /// Specifies how delimiters should wrap around a quoted identifier.
    pub identifier_format: IdentifierFormat,
    /// Specifies how to escape identifiers.
    pub identifier_escape: IdentifierEscape,
    /// Specifies how delimiters should wrap around a literal string value.
    pub string_format: StringFormat,
    /// Specifies how to escape literal string value.
    pub string_escape: StringEscape,
    /// How the dialect accepts literal bytes values.
    pub bytes_format: BytesFormat,
    pub date_format: TemporalLiteral,
    pub time_format: TemporalLiteral,
    pub timestamp_format: TemporalLiteral,
    pub json_format: JsonFormat,
    pub uuid_format: UuidFormat,
    /// How function calling `FILTER` keyword rendered if supported.
    pub function_filter_syntax: Option<FunctionFilterSyntax>,
    /// How case-insensitive `LIKE` is spelled (`ILIKE`, `LIKE`, or `LOWER` + `LIKE`).
    pub ilike_expression_options: ILikeExpressionOptions,
    /// Whether `SIMILAR TO` is documented.
    pub supports_similar_to: bool,
    /// How regex `RLike` is spelled. [`None`] means unsupported.
    pub rlike_expression_options: Option<RLikeExpressionOptions>,
    /// How `||` (string concat) is spelled.
    pub string_concat_syntax: StringConcatSyntax,
    /// How bitwise XOR (`^` in the AST) is spelled. [`None`] means unsupported.
    pub bit_xor_syntax: Option<BitXorSyntax>,
    /// How modulo (`%` in the AST) is spelled.
    pub modulo_syntax: ModuloSyntax,
    /// How `IS [NOT] DISTINCT FROM` is spelled. [`None`] means unsupported.
    pub is_distinct_from_syntax: Option<IsDistinctFromSyntax>,
    /// How a tuple value is spelled. [`None`] means unsupported.
    pub tuple_syntax: Option<TupleSyntax>,
    /// Wrap `IN` / `BETWEEN` in parentheses.
    ///
    /// SQL Server ranks `AND` above `IN`/`BETWEEN`/`OR`, and those four share
    /// a level evaluated left to right. The parentheses keep the predicate
    /// from being parsed as part of the left operand.
    pub parenthesize_in_between: bool,
    /// How `TRY_CAST` is spelled. [`None`] means unsupported.
    pub try_cast_syntax: Option<TryCastSyntax>,
    /// Whether `expr::type` is documented.
    pub supports_double_colon: bool,
    /// How `TRIM` / `LTRIM` / `RTRIM` is spelled.
    pub trim_syntax: TrimSyntax,
    /// How `COLLATE` is spelled. [`None`] means unsupported.
    pub collate_syntax: Option<CollateSyntax>,
}

impl Dialect {
    /// Write a boolean literal (`TRUE`/`FALSE` or `1`/`0`).
    #[inline]
    pub fn format_boolean<W: fmt::Write>(
        &self,
        writer: &mut W,
        value: bool,
    ) -> Result<(), FormatError> {
        match (self.bool_format, value) {
            (BoolFormat::Int, false) => writer.write_str("0")?,
            (BoolFormat::Int, true) => writer.write_str("1")?,
            (BoolFormat::Keyword, false) => writer.write_str("FALSE")?,
            (BoolFormat::Keyword, true) => writer.write_str("TRUE")?,
        }
        Ok(())
    }

    /// Write `string` with special characters neutralized, without wrapping it.
    #[inline]
    pub fn escape_string<W: fmt::Write>(
        &self,
        writer: &mut W,
        string: &str,
    ) -> Result<(), FormatError> {
        self.string_escape.write(writer, string).map_err(Into::into)
    }

    /// Escape `ident` and wrap it in this dialect's identifier delimiters.
    #[inline]
    pub fn quote_identifier<W: fmt::Write>(&self, writer: &mut W, ident: &str) -> fmt::Result {
        let (start, end) = match self.identifier_format {
            IdentifierFormat::DoubleQuote => ('"', '"'),
            IdentifierFormat::Backtick => ('`', '`'),
            IdentifierFormat::SquareBracket => ('[', ']'),
        };
        writer.write_char(start)?;

        match self.identifier_escape {
            IdentifierEscape::Double => match self.identifier_format {
                IdentifierFormat::DoubleQuote => {
                    utils::replace_to(ident, '"', "\"\"", writer)?;
                }
                IdentifierFormat::Backtick => {
                    utils::replace_to(ident, '`', "``", writer)?;
                }
                IdentifierFormat::SquareBracket => {
                    utils::replace_to(ident, ']', "]]", writer)?;
                }
            },
            IdentifierEscape::Backslash => match self.identifier_format {
                IdentifierFormat::DoubleQuote => {
                    utils::replace_to(ident, '"', "\\\"", writer)?;
                }
                IdentifierFormat::Backtick => {
                    utils::replace_to(ident, '`', "\\`", writer)?;
                }
                IdentifierFormat::SquareBracket => {
                    utils::replace_to(ident, ']', "\\]", writer)?;
                }
            },
        }

        writer.write_char(end)
    }

    /// Escape `raw` and wrap it as a complete string literal.
    #[inline]
    pub fn write_string_literal<W: fmt::Write>(
        &self,
        writer: &mut W,
        raw: &str,
    ) -> Result<(), FormatError> {
        if raw.contains('\0') {
            return Err(FormatError::NulInString);
        }

        if matches!(self.string_format, StringFormat::PrefixBigE) && !needs_c_escape(raw) {
            writer.write_char('\'')?;
            StringEscape::Quote.write(writer, raw)?;
            writer.write_char('\'')?;
            return Ok(());
        }

        match self.string_format {
            StringFormat::PrefixBigE => writer.write_str("E'")?,
            StringFormat::PrefixBigN => writer.write_str("N'")?,
            StringFormat::SingleQuote => writer.write_char('\'')?,
        }
        self.escape_string(writer, raw)?;
        writer.write_char('\'')?;
        Ok(())
    }

    /// Write a bit-string (`'0'`/`'1'` characters) as a native literal.
    ///
    /// Native formats embed `bits` as-is. Callers must pass only `'0'` and `'1'`;
    /// that is checked with `debug_assert!`.
    #[inline]
    pub fn format_bits<W: fmt::Write>(
        &self,
        writer: &mut W,
        bits: &str,
    ) -> Result<(), FormatError> {
        debug_assert!(
            bits.chars().all(|c| c == '0' || c == '1'),
            "format_bits: invalid input: {bits}"
        );
        match self.bit_format {
            Some(format) => format.write(writer, bits).map_err(Into::into),
            None => Err(FormatError::Unsupported("BitSerializer")),
        }
    }

    /// Write `bytes` as a native blob/bytea/varbyte literal.
    #[inline]
    pub fn format_bytes<W: fmt::Write>(
        &self,
        writer: &mut W,
        bytes: &[u8],
    ) -> Result<(), FormatError> {
        self.bytes_format.write(writer, bytes).map_err(Into::into)
    }
}

#[inline]
fn needs_c_escape(s: &str) -> bool {
    s.chars()
        .any(|c| matches!(c, '\\' | '\n' | '\r' | '\t') || (c as u32) < 32 || c as u32 == 127)
}

const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

/// Write `byte` as two uppercase hex digits.
#[inline]
fn write_hex2<W: Write>(writer: &mut W, byte: u8) -> fmt::Result {
    let buf = [
        HEX_DIGITS[(byte >> 4) as usize],
        HEX_DIGITS[(byte & 0x0F) as usize],
    ];
    // SAFETY: HEX_DIGITS is ASCII, so `buf` is valid UTF-8.
    writer.write_str(unsafe { std::str::from_utf8_unchecked(&buf) })
}

/// Write `bytes` as a contiguous uppercase hex run (`DEADBEEF`).
#[inline]
fn bytes_to_hex<W: Write>(writer: &mut W, bytes: &[u8]) -> fmt::Result {
    for &b in bytes {
        write_hex2(writer, b)?;
    }
    Ok(())
}

/// Write `\xNN` for a single byte (uppercase hex).
#[inline]
fn write_byte_hex_escape<W: Write>(writer: &mut W, byte: u8) -> fmt::Result {
    writer.write_str("\\x")?;
    write_hex2(writer, byte)
}

/// Write `\uNNNN` for a codepoint that fits in 16 bits.
#[inline]
fn write_unicode4_escape<W: Write>(writer: &mut W, code: u32) -> fmt::Result {
    debug_assert!(code <= 0xFFFF);
    writer.write_str("\\u")?;
    write_hex2(writer, (code >> 8) as u8)?;
    write_hex2(writer, code as u8)
}

#[cfg_attr(feature = "nightly", optimize(speed))]
fn write_backslash_hex<W: Write>(writer: &mut W, string: &str) -> fmt::Result {
    for ch in string.chars() {
        match ch {
            '\\' => writer.write_str("\\\\")?,
            '\'' => writer.write_str("\\'")?,
            '\n' => writer.write_str("\\n")?,
            '\r' => writer.write_str("\\r")?,
            '\t' => writer.write_str("\\t")?,
            '\0' => writer.write_str("\\x00")?,
            c if (c as u32) < 32 || c as u32 == 127 => {
                write_byte_hex_escape(writer, c as u8)?;
            }
            c => writer.write_char(c)?,
        }
    }
    Ok(())
}

#[cfg_attr(feature = "nightly", optimize(speed))]
fn write_backslash_mysql<W: Write>(writer: &mut W, string: &str) -> fmt::Result {
    for ch in string.chars() {
        match ch {
            '\\' => writer.write_str("\\\\")?,
            '\'' => writer.write_str("\\'")?,
            '"' => writer.write_str("\\\"")?,
            '\0' => writer.write_str("\\0")?,
            '\n' => writer.write_str("\\n")?,
            '\r' => writer.write_str("\\r")?,
            '\x1a' => writer.write_str("\\Z")?,
            c => writer.write_char(c)?,
        }
    }
    Ok(())
}

#[cfg_attr(feature = "nightly", optimize(speed))]
fn write_backslash_unicode<W: Write>(
    writer: &mut W,
    string: &str,
    escape_double_quote: bool,
) -> fmt::Result {
    for ch in string.chars() {
        match ch {
            '\\' => writer.write_str("\\\\")?,
            '\'' => writer.write_str("\\'")?,
            '"' if escape_double_quote => writer.write_str("\\\"")?,
            '\n' => writer.write_str("\\n")?,
            '\r' => writer.write_str("\\r")?,
            '\t' => writer.write_str("\\t")?,
            '\0' => writer.write_str("\\0")?,
            c if (c as u32) < 32 || c as u32 == 127 => {
                write_unicode4_escape(writer, c as u32)?;
            }
            c => writer.write_char(c)?,
        }
    }
    Ok(())
}

#[cfg_attr(feature = "nightly", optimize(speed))]
fn write_backslash_hex_zero<W: Write>(writer: &mut W, string: &str) -> fmt::Result {
    for ch in string.chars() {
        match ch {
            '\\' => writer.write_str("\\\\")?,
            '\'' => writer.write_str("\\'")?,
            '\n' => writer.write_str("\\n")?,
            '\r' => writer.write_str("\\r")?,
            '\t' => writer.write_str("\\t")?,
            '\0' => writer.write_str("\\0")?,
            c if (c as u32) < 32 || c as u32 == 127 => {
                write_byte_hex_escape(writer, c as u8)?;
            }
            c => writer.write_char(c)?,
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect_fmt(write: impl FnOnce(&mut String) -> fmt::Result) -> String {
        let mut buf = String::new();
        write(&mut buf).unwrap();
        buf
    }

    fn collect(write: impl FnOnce(&mut String) -> Result<(), FormatError>) -> String {
        let mut buf = String::new();
        write(&mut buf).unwrap();
        buf
    }

    fn escape(kind: StringEscape, string: &str) -> String {
        collect_fmt(|buf| kind.write(buf, string))
    }

    fn bytes(kind: BytesFormat, bytes: &[u8]) -> String {
        collect_fmt(|buf| kind.write(buf, bytes))
    }

    fn base() -> Dialect {
        Dialect {
            positional_parameter: PositionalParameter::Simple(':'),
            bit_format: None,
            array_format: None,
            inet_format: None,
            mac_format: None,
            null_treatment_syntax: Some(NullTreatmentSyntax::Postfix),
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Backslash,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::Quote,
            bytes_format: BytesFormat::PrefixBigX,
            date_format: TemporalLiteral::Quoted,
            time_format: TemporalLiteral::Quoted,
            timestamp_format: TemporalLiteral::Quoted,
            json_format: JsonFormat::Quoted,
            uuid_format: UuidFormat::Quoted,
            function_filter_syntax: None,
            ilike_expression_options: ILikeExpressionOptions::UseLike,
            supports_similar_to: false,
            rlike_expression_options: None,
            string_concat_syntax: StringConcatSyntax::Pipes,
            bit_xor_syntax: None,
            modulo_syntax: ModuloSyntax::Percent,
            is_distinct_from_syntax: None,
            tuple_syntax: None,
            parenthesize_in_between: false,
            try_cast_syntax: None,
            supports_double_colon: false,
            trim_syntax: TrimSyntax::SqlFrom,
            collate_syntax: None,
        }
    }

    #[test]
    fn quote_doubles_apostrophe() {
        assert_eq!(escape(StringEscape::Quote, "O'Brien"), "O''Brien");
        assert_eq!(escape(StringEscape::Quote, "'"), "''");
        assert_eq!(escape(StringEscape::Quote, "a\0b"), "a\0b");
    }

    #[test]
    fn backslash_variants() {
        let cases: [(StringEscape, &str, &str); 40] = [
            (StringEscape::BackslashHex, "'", "\\'"),
            (StringEscape::BackslashHex, "\\", "\\\\"),
            (StringEscape::BackslashHex, "\n", "\\n"),
            (StringEscape::BackslashHex, "\t", "\\t"),
            (StringEscape::BackslashHex, "\0", "\\x00"),
            (StringEscape::BackslashHex, "\"", "\""),
            (StringEscape::BackslashHex, "\x1a", "\\x1A"),
            (StringEscape::BackslashHex, "\x01", "\\x01"),
            (StringEscape::BackslashMysql, "'", "\\'"),
            (StringEscape::BackslashMysql, "\\", "\\\\"),
            (StringEscape::BackslashMysql, "\n", "\\n"),
            (StringEscape::BackslashMysql, "\t", "\t"),
            (StringEscape::BackslashMysql, "\0", "\\0"),
            (StringEscape::BackslashMysql, "\"", "\\\""),
            (StringEscape::BackslashMysql, "\x1a", "\\Z"),
            (StringEscape::BackslashMysql, "\x01", "\x01"),
            (StringEscape::BackslashUnicode, "'", "\\'"),
            (StringEscape::BackslashUnicode, "\\", "\\\\"),
            (StringEscape::BackslashUnicode, "\n", "\\n"),
            (StringEscape::BackslashUnicode, "\t", "\\t"),
            (StringEscape::BackslashUnicode, "\0", "\\0"),
            (StringEscape::BackslashUnicode, "\"", "\""),
            (StringEscape::BackslashUnicode, "\x1a", "\\u001A"),
            (StringEscape::BackslashUnicode, "\x01", "\\u0001"),
            (StringEscape::BackslashHexZero, "'", "\\'"),
            (StringEscape::BackslashHexZero, "\\", "\\\\"),
            (StringEscape::BackslashHexZero, "\n", "\\n"),
            (StringEscape::BackslashHexZero, "\t", "\\t"),
            (StringEscape::BackslashHexZero, "\0", "\\0"),
            (StringEscape::BackslashHexZero, "\"", "\""),
            (StringEscape::BackslashHexZero, "\x1a", "\\x1A"),
            (StringEscape::BackslashHexZero, "\x01", "\\x01"),
            (StringEscape::BackslashUnicodeQuote, "'", "\\'"),
            (StringEscape::BackslashUnicodeQuote, "\\", "\\\\"),
            (StringEscape::BackslashUnicodeQuote, "\n", "\\n"),
            (StringEscape::BackslashUnicodeQuote, "\t", "\\t"),
            (StringEscape::BackslashUnicodeQuote, "\0", "\\0"),
            (StringEscape::BackslashUnicodeQuote, "\"", "\\\""),
            (StringEscape::BackslashUnicodeQuote, "\x1a", "\\u001A"),
            (StringEscape::BackslashUnicodeQuote, "\x01", "\\u0001"),
        ];
        for (kind, input, expected) in cases {
            assert_eq!(
                escape(kind, input),
                expected,
                "{kind:?} escaped {input:?} incorrectly"
            );
        }
    }

    #[test]
    fn bytes_format_wrappers() {
        let empty: &[u8] = &[];
        let zero: &[u8] = &[0x00];
        let dead: &[u8] = &[0xDE, 0xAD];

        assert_eq!(bytes(BytesFormat::PrefixBigX, empty), "X''");
        assert_eq!(bytes(BytesFormat::PrefixBigX, zero), "X'00'");
        assert_eq!(bytes(BytesFormat::PrefixBigX, dead), "X'DEAD'");

        assert_eq!(bytes(BytesFormat::PrefixZeroX, empty), "0x");
        assert_eq!(bytes(BytesFormat::PrefixZeroX, zero), "0x00");
        assert_eq!(bytes(BytesFormat::PrefixZeroX, dead), "0xDEAD");

        assert_eq!(bytes(BytesFormat::SlashXBytea, empty), "'\\x'::bytea");
        assert_eq!(bytes(BytesFormat::SlashXBytea, zero), "'\\x00'::bytea");
        assert_eq!(bytes(BytesFormat::SlashXBytea, dead), "'\\xDEAD'::bytea");

        assert_eq!(bytes(BytesFormat::PrefixBigBSlashX, empty), "B''");
        assert_eq!(bytes(BytesFormat::PrefixBigBSlashX, zero), "B'\\x00'");
        assert_eq!(bytes(BytesFormat::PrefixBigBSlashX, dead), "B'\\xDE\\xAD'");

        assert_eq!(bytes(BytesFormat::HexToRaw, empty), "HEXTORAW('')");
        assert_eq!(bytes(BytesFormat::HexToRaw, zero), "HEXTORAW('00')");
        assert_eq!(bytes(BytesFormat::HexToRaw, dead), "HEXTORAW('DEAD')");

        assert_eq!(bytes(BytesFormat::Unhex, empty), "UNHEX('')");
        assert_eq!(bytes(BytesFormat::Unhex, zero), "UNHEX('00')");
        assert_eq!(bytes(BytesFormat::Unhex, dead), "UNHEX('DEAD')");

        assert_eq!(
            bytes(BytesFormat::ToVarbyte, empty),
            "TO_VARBYTE('', 'hex')"
        );
        assert_eq!(
            bytes(BytesFormat::ToVarbyte, zero),
            "TO_VARBYTE('00', 'hex')"
        );
        assert_eq!(
            bytes(BytesFormat::ToVarbyte, dead),
            "TO_VARBYTE('DEAD', 'hex')"
        );

        assert_eq!(bytes(BytesFormat::PostfixXbv, empty), "''XBV");
        assert_eq!(bytes(BytesFormat::PostfixXbv, zero), "'00'XBV");
        assert_eq!(bytes(BytesFormat::PostfixXbv, dead), "'DEAD'XBV");

        assert_eq!(bytes(BytesFormat::ToBinary, empty), "TO_BINARY('', 'HEX')");
        assert_eq!(bytes(BytesFormat::ToBinary, zero), "TO_BINARY('00', 'HEX')");
        assert_eq!(
            bytes(BytesFormat::ToBinary, dead),
            "TO_BINARY('DEAD', 'HEX')"
        );

        assert_eq!(bytes(BytesFormat::SlashXBlob, empty), "''::BLOB");
        assert_eq!(bytes(BytesFormat::SlashXBlob, zero), "'\\x00'::BLOB");
        assert_eq!(bytes(BytesFormat::SlashXBlob, dead), "'\\xDE\\xAD'::BLOB");
    }

    #[test]
    fn bit_format_literals() {
        let bits = "10110101";
        assert_eq!(
            collect_fmt(|buf| BitFormat::PrefixBigB.write(buf, bits)),
            "B'10110101'"
        );
        assert_eq!(
            collect_fmt(|buf| BitFormat::PrefixSmallB.write(buf, bits)),
            "b'10110101'"
        );
        assert_eq!(
            collect_fmt(|buf| BitFormat::PostfixBigB.write(buf, bits)),
            "'10110101'B"
        );

        let mut cfg = base();
        cfg.bit_format = Some(BitFormat::PrefixBigB);
        assert_eq!(collect(|buf| cfg.format_bits(buf, bits)), "B'10110101'");
        assert_eq!(
            collect(|buf| cfg.format_bytes(buf, &[0xDE, 0xAD])),
            "X'DEAD'"
        );
    }

    #[test]
    fn format_bits_errors_when_unsupported() {
        let cfg = base();
        assert_eq!(
            cfg.format_bits(&mut String::new(), "1"),
            Err(FormatError::Unsupported("BitSerializer"))
        );
    }

    #[test]
    fn write_string_literal_rejects_nul() {
        let cfg = base();
        assert_eq!(
            cfg.write_string_literal(&mut String::new(), "a\0b"),
            Err(FormatError::NulInString)
        );
    }

    #[test]
    fn write_string_literal_prefix_big_e_when_needed() {
        let mut cfg = base();
        cfg.string_format = StringFormat::PrefixBigE;
        cfg.string_escape = StringEscape::BackslashHex;
        assert_eq!(
            collect(|buf| cfg.write_string_literal(buf, "hello")),
            "'hello'"
        );
        assert_eq!(
            collect(|buf| cfg.write_string_literal(buf, "O'Brien")),
            "'O''Brien'"
        );
        assert_eq!(
            collect(|buf| cfg.write_string_literal(buf, "hello'world\n")),
            "E'hello\\'world\\n'"
        );
    }
}
