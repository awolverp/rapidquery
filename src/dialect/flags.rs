use bitfield_struct::bitenum;
use bitfield_struct::bitfield;

use crate::internal::utils;

use super::errors::FormatError;

/// Positional parameter syntax
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Placeholder {
    /// e.g. `'?'`
    #[fallback]
    Simple,
    /// e.g. `'?1'`
    Numbered,
}

/// The way dialect supports `{IGNORE|RESPECT} NULLS` expressions in `Function`s.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NullTreatment {
    /// Is not supported
    #[fallback]
    Unsupported,
    /// e.g. `FIRST_VALUE(x IGNORE NULLS)`
    InsideParens,
    /// e.g. `FIRST_VALUE(x) IGNORE NULLS`
    Postfix,
}

/// Where an inlined `LIKE` `ESCAPE` clause is legal.
///
/// `escape: None` writes nothing. A set escape character on a form this
/// variant does not allow raises.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LikeEscape {
    /// No `ESCAPE` clause.
    Unsupported,
    /// Scalar `LIKE` and an `OR` expansion. Native `LIKE ANY` has no slot.
    Scalar,
    /// Scalar, `OR` expansion, and `LIKE ANY (p, …) ESCAPE`.
    #[fallback]
    IncludingAny,
}

/// How `LIKE ANY` / `ILIKE ANY` is rendered.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LikeAny {
    /// `(x LIKE p1 OR x LIKE p2)`.
    /// Useful when it isn't supported natively by dialect
    #[fallback]
    OrExpansion,
    /// `LIKE ANY (ARRAY[p, …])`.
    ArrayConstructor,
    /// `LIKE ANY (p, …)`.
    PatternList,
}

/// Does dialect support `ILIKE` expression or we should fallback to another options?
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ILikeExpression {
    #[fallback]
    Standard,
    /// Use `LIKE` expression and `LOWER()` function
    UseLikeLower,
    /// Fallback to `LIKE` expression
    UseLike,
}

/// How regex `RLike` is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RLikeExpression {
    /// Is not supported.
    #[fallback]
    Unsupported,
    /// `expr [NOT] RLIKE pattern`
    Standard,
    /// `expr ~ pattern` / `expr !~ pattern`
    Tilde,
    /// `REGEXP_LIKE(expr, pattern)`
    RegexpLike,
    /// `REGEXP_CONTAINS(expr, pattern)`
    RegexpContains,
    /// `regexp_matches(expr, pattern)`
    RegexpMatches,
}

/// How binary operation StringConcat (`||`) is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StringConcat {
    /// `left || right`.
    #[fallback]
    Pipes,
    /// `left + right`. SQL Server `||` is 2025+; classic concat is `+`.
    Plus,
    /// `CONCAT(left, right)`. MySQL `||` is logical OR unless `PIPES_AS_CONCAT`.
    ConcatFunction,
}

/// How binary operation bit xor (`^`) is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BitXor {
    /// Is not supported
    Unsupported,
    /// `left ^ right`.
    #[fallback]
    Caret,
    /// `left # right`. PostgreSQL `^` is exponent.
    Hash,
    /// `xor(left, right)`.
    XORFunction,
    /// `BITXOR(left, right)`.
    BitXORFunction,
}

/// How binary operation mod (`%`) is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modulo {
    /// `left % right`.
    #[fallback]
    Percent,
    /// `MOD(left, right)`.
    Function,
    /// `left MOD right`.
    Infix,
}

/// How `IS [NOT] DISTINCT FROM` is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IsDistinctFrom {
    /// Is not supported
    Unsupported,
    /// `left IS [NOT] DISTINCT FROM right`.
    #[fallback]
    Standard,
    /// `<=>` is null-safe equal (`IS NOT DISTINCT FROM`).
    /// `IS DISTINCT FROM` is `NOT (left <=> right)`.
    NullSafeEqual,
}

/// How `TRY_CAST` is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TryCast {
    /// Is not supported
    Unsupported,
    /// `TRY_CAST(expr AS type)`.
    #[fallback]
    TryCast,
    /// `TRYCAST(expr AS type)`.
    TryCastWord,
    /// `SAFE_CAST(expr AS type)`. BigQuery's spelling of a null-on-failure cast.
    SafeCast,
}

/// How `TRIM` is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trim {
    /// `TRIM([LEADING|TRAILING] [chars FROM] expr)`.
    #[fallback]
    SqlFrom,
    /// `TRIM` / `LTRIM` / `RTRIM` `(expr[, chars])`.
    Function,
    /// `TRIM` / `LTRIM` / `RTRIM` `(expr)`. A character set raises.
    SpacesOnly,
}

/// How `COLLATE` is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Collate {
    /// Is not supported
    Unsupported,
    /// `expr COLLATE ident`.
    #[fallback]
    InfixIdent,
    /// `COLLATE(expr, 'body')`. The identifier body is a string literal.
    FunctionString,
}

/// How `&`, `|`, `~`, `<<`, and `>>` are spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Bitwise {
    /// All raises.
    Unsupported,
    /// `& | ~ << >>`.
    #[fallback]
    Infix,
    /// `& << >>`. `|` and `~` raise.
    ///
    /// Redshift documents those three and not bitwise OR or NOT
    /// (`^` is exponentiation and stays on [`BitXorSyntax`]).
    InfixAndShift,
    /// `& | ~`. Shifts raise (SQL Server).
    InfixNoShift,
    /// `BITAND` / `BITOR` / `BITNOT` / `BITSHIFTLEFT` / `BITSHIFTRIGHT`.
    BitShiftFunctions,
    /// `BITAND` / `BITOR` / `BITNOT` / `SHIFTLEFT` / `SHIFTRIGHT` (Teradata).
    ShiftFunctions,
    /// `BITAND` only. `|`, `~`, `<<`, and `>>` raise (Oracle).
    BitAndOnly,
}

/// How a tuple / row value is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tuple {
    /// Is not supported.
    Unsupported,
    /// `ROW(...)`. `empty_ok` allows arity 0 (`ROW()`).
    Row,
    /// `ROW(...)`. Rejects empty (`ROW()`).
    RowRejectEmpty,
    /// `struct(...)`. Arity 0 is unsupported.
    Struct,
    /// `(a, b, ...)`. Arity 0 and 1 are unsupported.
    #[fallback]
    Parentheses,
}

/// The way dialect accepts literal bit values
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BitFormat {
    /// Is not supported.
    Unsupported,
    /// e.g. `B'101'`
    #[fallback]
    PrefixBigB,
    /// e.g. `b'101'`
    PrefixSmallB,
    /// e.g. `'101'B`
    PostfixBigB,
}

/// How a boolean literal is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoolFormat {
    /// `TRUE` / `FALSE`
    #[fallback]
    Keyword,
    /// `1` / `0`
    Integer,
}

/// Delimiters wrapped around a quoted identifier.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentifierFormat {
    /// `"name"`
    #[fallback]
    DoubleQuote,
    /// `` `name` ``
    Backtick,
    /// `[name]`
    SquareBracket,
}

/// How to escape identifiers.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentifierEscape {
    /// Escape identifier delimiters with doubling.
    #[fallback]
    Double,
    /// Escape identifier delimiters with backslash.
    Backslash,
}

/// Delimiters wrapped around a literal string.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StringFormat {
    /// `'string'`
    #[fallback]
    SingleQuote,
    /// `E'string'`
    PrefixBigE,
    /// `N'string'`
    PrefixBigN,
}

/// How to escape literal strings.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StringEscape {
    /// `'` → `''`. Other characters are unchanged.
    #[fallback]
    Quote,
    /// `\\`, `\'`, `\n`, `\r`, `\t`; other C0/DEL as `\xNN`.
    BackslashHex,
    /// Also `\"` and `\Z` for 0x1A; tab and other controls pass through.
    BackslashMysql,
    /// Other C0/DEL as `\uNNNN`. No `\"`.
    BackslashUnicode,
    /// Other C0/DEL as `\xNN`.
    BackslashHexZero,
    /// [`Self::BackslashUnicode`] plus `\"`.
    BackslashUnicodeQuote,
}

/// The way dialect accepts literal bytes values.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BytesFormat {
    /// `X'DEADBEEF'`
    #[fallback]
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

/// Function `FILTER` syntax.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FunctionFilter {
    /// Is not supported.
    Unsupported,
    /// e.g. FIRST_VALUE(x) FILTER (id > 10)
    Filter,
    /// e.g. FIRST_VALUE(x) FILTER (WHERE id > 10)
    #[fallback]
    FilterWhere,
    /// e.g. FIRST_VALUE(x WHERE id > 10)
    InsideWhere,
}

/// The way dialect accepts array values.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArrayFormat {
    /// Is not supported.
    #[fallback]
    Unsupported,
    /// `ARRAY[…]`
    Constructor,
    /// `[…]`
    List,
    /// `array(…)`
    Function,
}

/// How a date literal is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DateFormat {
    /// Quoted string.
    #[fallback]
    Quoted,
    /// `DATE`, a space, then a quoted literal.
    Keyword,
}

/// How a time literal is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeFormat {
    /// Quoted string.
    #[fallback]
    Quoted,
    /// `TIME`, a space, then a quoted literal.
    Keyword,
}

/// How a timestamp literal is spelled.
///
/// Naive values use `TIMESTAMP` when a keyword is configured. Aware values
/// (`datetime` with a tzinfo, or a UTC epoch number) use the keyword this
/// variant names.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimestampFormat {
    /// Both naive and aware values are a quoted string. No keyword.
    Quoted,
    /// Naive: `TIMESTAMP '…'`. Aware: `TIMESTAMPTZ '…'`.
    Timestamptz,
    /// Naive: `TIMESTAMP '…'`. Aware: `TIMESTAMP WITH TIMEZONE '…'`.
    #[fallback]
    WithTimezone,
}

/// How an inet literal is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InetFormat {
    /// Is not supported.
    #[fallback]
    Unsupported,
    /// `'192.168.1.1/32'`
    Quoted,
    /// `'192.168.1.1/32'::inet`
    Cast,
}

/// How a MAC address literal is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MacAddrFormat {
    /// Is not supported.
    #[fallback]
    Unsupported,
    /// `'01:23:45:67:89:AB'`
    Quoted,
    /// `'01:23:45:67:89:AB'::macaddr`
    Cast,
}

/// How a UUID literal is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UuidFormat {
    /// Quoted hyphenated text.
    #[fallback]
    Quoted,
    /// `UUID '…'`
    UuidKeyword,
    /// `'…'::uuid`
    CastUuid,
}

/// How a JSON literal is spelled.
#[bitenum]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JsonFormat {
    /// Quoted JSON text.
    #[fallback]
    Quoted,
    /// `JSON '…'`
    JsonKeyword,
    /// `PARSE_JSON('…')`
    ParseJson,
}

#[bitfield(u128)]
#[derive(PartialEq, Eq, Hash)]
pub struct DialectFlags {
    /// Placeholder character as u32
    ///
    /// Always is a valid character
    #[bits(32)]
    pub placeholder_character: u32,
    /// Positional parameter syntax
    #[bits(1)]
    pub placeholder: Placeholder,
    /// The way dialect supports `{IGNORE|RESPECT} NULLS` expressions in `Function`s.
    #[bits(2)]
    pub null_treatment: NullTreatment,
    /// Where an inlined `LIKE` `ESCAPE` clause is legal.
    #[bits(2)]
    pub like_escape: LikeEscape,
    /// How `LIKE ANY` / `ILIKE ANY` is rendered.
    #[bits(2)]
    pub like_any: LikeAny,
    /// Does dialect support `ILIKE` expression or we should fallback to another options?
    #[bits(2)]
    pub ilike_expression: ILikeExpression,
    /// How regex `RLike` is spelled.
    #[bits(3)]
    pub rlike_expression: RLikeExpression,
    /// How binary operation StringConcat (`||`) is spelled.
    #[bits(2)]
    pub string_concat: StringConcat,
    /// How binary operation bit xor (`^`) is spelled.
    #[bits(3)]
    pub bitxor: BitXor,
    /// How binary operation mod (`%`) is spelled.
    #[bits(2)]
    pub modulo: Modulo,
    /// How `IS [NOT] DISTINCT FROM` is spelled.
    #[bits(2)]
    pub is_distinct_from: IsDistinctFrom,
    /// How `TRY_CAST` is spelled.
    #[bits(2)]
    pub try_cast: TryCast,
    /// How TRIM is spelled.
    #[bits(2)]
    pub trim: Trim,
    /// How `COLLATE` is spelled.
    #[bits(2)]
    pub collate: Collate,
    /// How `&`, `|`, `~`, `<<`, and `>>` are spelled.
    #[bits(3)]
    pub bitwise: Bitwise,
    /// How a tuple / row value is spelled.
    #[bits(3)]
    pub tuple: Tuple,
    /// The way dialect accepts literal bit values.
    #[bits(2)]
    pub bit_format: BitFormat,
    /// How a boolean literal is spelled.
    #[bits(1)]
    pub bool_format: BoolFormat,
    /// Delimiters wrapped around a quoted identifier.
    #[bits(2)]
    pub identifier_format: IdentifierFormat,
    /// How to escape identifiers.
    #[bits(1)]
    pub identifier_escape: IdentifierEscape,
    /// Delimiters wrapped around a literal string.
    #[bits(2)]
    pub string_format: StringFormat,
    /// How to escape literal strings.
    #[bits(3)]
    pub string_escape: StringEscape,
    /// The way dialect accepts literal bytes values.
    #[bits(4)]
    pub bytes_format: BytesFormat,
    /// Function `FILTER` syntax.
    #[bits(3)]
    pub function_filter: FunctionFilter,
    /// The way dialect accepts array values.
    #[bits(3)]
    pub array_format: ArrayFormat,
    /// How a date literal is spelled.
    #[bits(3)]
    pub date_format: DateFormat,
    /// How a time literal is spelled.
    #[bits(3)]
    pub time_format: TimeFormat,
    /// How a timestamp literal is spelled.
    #[bits(2)]
    pub timestamp_format: TimestampFormat,
    /// How an inet literal is spelled.
    #[bits(2)]
    pub inet_format: InetFormat,
    /// How a MAC address literal is spelled.
    #[bits(2)]
    pub macaddr_format: MacAddrFormat,
    /// How a UUID literal is spelled.
    #[bits(2)]
    pub uuid_format: UuidFormat,
    /// How a JSON literal is spelled.
    #[bits(2)]
    pub json_format: JsonFormat,
    /// Whether `SIMILAR TO` is supported.
    #[bits(1)]
    pub supports_similar_to: bool,
    /// Wrap `IN` / `BETWEEN` in parentheses.
    ///
    /// SQL Server ranks `AND` above `IN`/`BETWEEN`/`OR`, and those four share
    /// a level evaluated left to right. The parentheses keep the predicate
    /// from being parsed as part of the left operand.
    #[bits(1)]
    pub parenthesize_in_between: bool,
    /// Whether `expr::type` is supported.
    #[bits(1)]
    pub supports_double_colon: bool,

    // Padding
    #[bits(23)]
    __: u32,
}

impl DialectFlags {
    /// Write a boolean literal (`TRUE`/`FALSE` or `1`/`0`).
    #[inline]
    pub fn write_boolean<W: std::fmt::Write>(&self, writer: &mut W, value: bool) -> Result<(), FormatError> {
        match (self.bool_format(), value) {
            (BoolFormat::Integer, false) => writer.write_str("0")?,
            (BoolFormat::Integer, true) => writer.write_str("1")?,
            (BoolFormat::Keyword, false) => writer.write_str("FALSE")?,
            (BoolFormat::Keyword, true) => writer.write_str("TRUE")?,
        }
        Ok(())
    }

    /// Write `string` with special characters neutralized, without wrapping it.
    #[inline]
    pub fn escape_string<W: std::fmt::Write>(&self, writer: &mut W, string: &str) -> Result<(), FormatError> {
        let result = match self.string_escape() {
            StringEscape::Quote => utils::replace_to(string, '\'', "''", writer),
            StringEscape::BackslashHex => write_backslash_hex(writer, string),
            StringEscape::BackslashMysql => write_backslash_mysql(writer, string),
            StringEscape::BackslashUnicode => write_backslash_unicode(writer, string, false),
            StringEscape::BackslashHexZero => write_backslash_hex_zero(writer, string),
            StringEscape::BackslashUnicodeQuote => write_backslash_unicode(writer, string, true),
        };
        result.map_err(Into::into)
    }

    /// Escape `ident` and wrap it in this dialect's identifier delimiters.
    #[inline]
    pub fn write_identifier<W: std::fmt::Write>(&self, writer: &mut W, ident: &str) -> std::fmt::Result {
        let (start, end) = match self.identifier_format() {
            IdentifierFormat::DoubleQuote => ('"', '"'),
            IdentifierFormat::Backtick => ('`', '`'),
            IdentifierFormat::SquareBracket => ('[', ']'),
        };
        writer.write_char(start)?;

        match self.identifier_escape() {
            IdentifierEscape::Double => match self.identifier_format() {
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
            IdentifierEscape::Backslash => match self.identifier_format() {
                IdentifierFormat::DoubleQuote => {
                    utils::replace_to(ident, '"', "\\\"", writer)?;
                }
                IdentifierFormat::Backtick => {
                    write_identifier_backtick_escape(writer, ident)?;
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
    pub fn write_string_literal<W: std::fmt::Write>(&self, writer: &mut W, raw: &str) -> Result<(), FormatError> {
        if raw.contains('\0') {
            return Err(FormatError::NulInString);
        }

        match self.string_format() {
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
    pub fn write_bits<W: std::fmt::Write>(&self, writer: &mut W, bits: &str) -> Result<(), FormatError> {
        debug_assert!(
            bits.chars().all(|c| c == '0' || c == '1'),
            "write_bits: invalid input: {bits}"
        );
        match self.bit_format() {
            BitFormat::Unsupported => Err(FormatError::Unsupported("BitSerializer")),
            BitFormat::PrefixBigB => write!(writer, "B'{bits}'").map_err(Into::into),
            BitFormat::PrefixSmallB => write!(writer, "b'{bits}'").map_err(Into::into),
            BitFormat::PostfixBigB => write!(writer, "'{bits}'B").map_err(Into::into),
        }
    }

    /// Write `bytes` as a native blob/bytea/varbyte literal.
    #[inline]
    pub fn write_bytes<W: std::fmt::Write>(&self, writer: &mut W, bytes: &[u8]) -> Result<(), FormatError> {
        let result = match self.bytes_format() {
            BytesFormat::PrefixBigX => {
                writer.write_str("X'")?;
                bytes_to_hex(writer, bytes)?;
                writer.write_char('\'')
            }
            BytesFormat::PrefixZeroX => {
                writer.write_str("0x")?;
                bytes_to_hex(writer, bytes)
            }
            BytesFormat::SlashXBytea => {
                writer.write_str("'\\x")?;
                bytes_to_hex(writer, bytes)?;
                writer.write_str("'::bytea")
            }
            BytesFormat::PrefixBigBSlashX => {
                writer.write_str("B'")?;
                write_bytes_slash_x(writer, bytes)?;
                writer.write_char('\'')
            }
            BytesFormat::HexToRaw => {
                writer.write_str("HEXTORAW('")?;
                bytes_to_hex(writer, bytes)?;
                writer.write_str("')")
            }
            BytesFormat::Unhex => {
                writer.write_str("UNHEX('")?;
                bytes_to_hex(writer, bytes)?;
                writer.write_str("')")
            }
            BytesFormat::ToVarbyte => {
                writer.write_str("TO_VARBYTE('")?;
                bytes_to_hex(writer, bytes)?;
                writer.write_str("', 'hex')")
            }
            BytesFormat::PostfixXbv => {
                writer.write_char('\'')?;
                bytes_to_hex(writer, bytes)?;
                writer.write_str("'XBV")
            }
            BytesFormat::ToBinary => {
                writer.write_str("TO_BINARY('")?;
                bytes_to_hex(writer, bytes)?;
                writer.write_str("', 'HEX')")
            }
            BytesFormat::SlashXBlob => {
                if bytes.is_empty() {
                    writer.write_str("''::BLOB")
                } else {
                    writer.write_char('\'')?;
                    write_bytes_slash_x(writer, bytes)?;
                    writer.write_str("'::BLOB")
                }
            }
        };
        result.map_err(Into::into)
    }
}

const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

/// Write `byte` as two uppercase hex digits.
#[inline]
fn write_hex2<W: std::fmt::Write>(writer: &mut W, byte: u8) -> std::fmt::Result {
    let buf = [HEX_DIGITS[(byte >> 4) as usize], HEX_DIGITS[(byte & 0x0F) as usize]];
    // SAFETY: HEX_DIGITS is ASCII, so `buf` is valid UTF-8.
    writer.write_str(unsafe { std::str::from_utf8_unchecked(&buf) })
}

/// Write `bytes` as a contiguous uppercase hex run (`DEADBEEF`).
///
/// 64 raw bytes fill a 128-byte stack buffer and are written once.
#[inline]
fn bytes_to_hex<W: std::fmt::Write>(writer: &mut W, bytes: &[u8]) -> std::fmt::Result {
    let mut buf = [0u8; 128];
    for chunk in bytes.chunks(64) {
        for (i, &byte) in chunk.iter().enumerate() {
            let o = i * 2;
            buf[o] = HEX_DIGITS[(byte >> 4) as usize];
            buf[o + 1] = HEX_DIGITS[(byte & 0x0F) as usize];
        }
        let n = chunk.len() * 2;
        // SAFETY: `buf[..n]` is filled only from `HEX_DIGITS` (ASCII).
        writer.write_str(unsafe { std::str::from_utf8_unchecked(&buf[..n]) })?;
    }
    Ok(())
}

/// Write each byte as `\xNN` (uppercase hex).
///
/// 32 raw bytes fill a 128-byte stack buffer and are written once.
#[inline]
fn write_bytes_slash_x<W: std::fmt::Write>(writer: &mut W, bytes: &[u8]) -> std::fmt::Result {
    let mut buf = [0u8; 128];
    for chunk in bytes.chunks(32) {
        for (i, &byte) in chunk.iter().enumerate() {
            let o = i * 4;
            buf[o] = b'\\';
            buf[o + 1] = b'x';
            buf[o + 2] = HEX_DIGITS[(byte >> 4) as usize];
            buf[o + 3] = HEX_DIGITS[(byte & 0x0F) as usize];
        }
        let n = chunk.len() * 4;
        // SAFETY: `buf[..n]` is ASCII (`\`, `x`, and hex digits).
        writer.write_str(unsafe { std::str::from_utf8_unchecked(&buf[..n]) })?;
    }
    Ok(())
}

/// Write `\xNN` for a single byte (uppercase hex).
#[inline]
fn write_byte_hex_escape<W: std::fmt::Write>(writer: &mut W, byte: u8) -> std::fmt::Result {
    writer.write_str("\\x")?;
    write_hex2(writer, byte)
}

/// Write `\uNNNN` for a codepoint that fits in 16 bits.
#[inline]
fn write_unicode4_escape<W: std::fmt::Write>(writer: &mut W, code: u32) -> std::fmt::Result {
    debug_assert!(code <= 0xFFFF);
    writer.write_str("\\u")?;
    write_hex2(writer, (code >> 8) as u8)?;
    write_hex2(writer, code as u8)
}

/// Use the string escape table, plus `\``.
///
/// `'` is left as itself. Only known escapes are emitted.
fn write_identifier_backtick_escape<W: std::fmt::Write>(writer: &mut W, ident: &str) -> std::fmt::Result {
    write_span_escapes(writer, ident, |b| match b {
        b'\\' => SpanEscape::Literal("\\\\"),
        b'`' => SpanEscape::Literal("\\`"),
        b'\n' => SpanEscape::Literal("\\n"),
        b'\r' => SpanEscape::Literal("\\r"),
        b'\t' => SpanEscape::Literal("\\t"),
        b if b < 32 || b == 127 => SpanEscape::HexByte,
        _ => SpanEscape::Keep,
    })
}

/// Write `bytes[from..to]` as a string.
///
/// # Safety invariant
/// `bytes` is the raw view of a `&str`. Callers split only on ASCII bytes,
/// which are char boundaries, or at `bytes.len()`.
#[inline]
fn write_byte_span<W: std::fmt::Write>(writer: &mut W, bytes: &[u8], from: usize, to: usize) -> std::fmt::Result {
    if from == to {
        return Ok(());
    }
    // SAFETY: see invariant above.
    let chunk = unsafe { bytes.get_unchecked(from..to) };
    writer.write_str(unsafe { std::str::from_utf8_unchecked(chunk) })
}

enum SpanEscape {
    Keep,
    Literal(&'static str),
    HexByte,
    Unicode4,
}

/// Copy each plain span with one `write_str`. Special bytes are ASCII.
#[cfg_attr(feature = "nightly", optimize(speed))]
fn write_span_escapes<W: std::fmt::Write>(
    writer: &mut W,
    string: &str,
    mut classify: impl FnMut(u8) -> SpanEscape,
) -> std::fmt::Result {
    let bytes = string.as_bytes();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        match classify(bytes[i]) {
            SpanEscape::Keep => i += 1,
            SpanEscape::Literal(repl) => {
                write_byte_span(writer, bytes, start, i)?;
                writer.write_str(repl)?;
                i += 1;
                start = i;
            }
            SpanEscape::HexByte => {
                write_byte_span(writer, bytes, start, i)?;
                write_byte_hex_escape(writer, bytes[i])?;
                i += 1;
                start = i;
            }
            SpanEscape::Unicode4 => {
                write_byte_span(writer, bytes, start, i)?;
                write_unicode4_escape(writer, bytes[i] as u32)?;
                i += 1;
                start = i;
            }
        }
    }
    write_byte_span(writer, bytes, start, bytes.len())
}

fn write_backslash_hex<W: std::fmt::Write>(writer: &mut W, string: &str) -> std::fmt::Result {
    write_span_escapes(writer, string, |b| match b {
        b'\\' => SpanEscape::Literal("\\\\"),
        b'\'' => SpanEscape::Literal("\\'"),
        b'\n' => SpanEscape::Literal("\\n"),
        b'\r' => SpanEscape::Literal("\\r"),
        b'\t' => SpanEscape::Literal("\\t"),
        b if b < 32 || b == 127 => SpanEscape::HexByte,
        _ => SpanEscape::Keep,
    })
}

fn write_backslash_mysql<W: std::fmt::Write>(writer: &mut W, string: &str) -> std::fmt::Result {
    write_span_escapes(writer, string, |b| match b {
        b'\\' => SpanEscape::Literal("\\\\"),
        b'\'' => SpanEscape::Literal("\\'"),
        b'"' => SpanEscape::Literal("\\\""),
        0 => SpanEscape::Literal("\\0"),
        b'\n' => SpanEscape::Literal("\\n"),
        b'\r' => SpanEscape::Literal("\\r"),
        0x1a => SpanEscape::Literal("\\Z"),
        _ => SpanEscape::Keep,
    })
}

fn write_backslash_unicode<W: std::fmt::Write>(
    writer: &mut W,
    string: &str,
    escape_double_quote: bool,
) -> std::fmt::Result {
    write_span_escapes(writer, string, |b| match b {
        b'\\' => SpanEscape::Literal("\\\\"),
        b'\'' => SpanEscape::Literal("\\'"),
        b'"' if escape_double_quote => SpanEscape::Literal("\\\""),
        b'\n' => SpanEscape::Literal("\\n"),
        b'\r' => SpanEscape::Literal("\\r"),
        b'\t' => SpanEscape::Literal("\\t"),
        0 => SpanEscape::Literal("\\0"),
        b if b < 32 || b == 127 => SpanEscape::Unicode4,
        _ => SpanEscape::Keep,
    })
}

fn write_backslash_hex_zero<W: std::fmt::Write>(writer: &mut W, string: &str) -> std::fmt::Result {
    write_span_escapes(writer, string, |b| match b {
        b'\\' => SpanEscape::Literal("\\\\"),
        b'\'' => SpanEscape::Literal("\\'"),
        b'\n' => SpanEscape::Literal("\\n"),
        b'\r' => SpanEscape::Literal("\\r"),
        b'\t' => SpanEscape::Literal("\\t"),
        0 => SpanEscape::Literal("\\0"),
        b if b < 32 || b == 127 => SpanEscape::HexByte,
        _ => SpanEscape::Keep,
    })
}
