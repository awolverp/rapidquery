use crate::internal::utils;

use super::errors::FormatError;

/// Packed options that decide how one dialect spells SQL.
///
/// ```
/// use rapidquery::dialect::config::BoolFormat;
/// use rapidquery::dialect::config::DialectConfig;
///
/// let mut c = DialectConfig { bool_format: BoolFormat::Integer, ..Default::default() };
///
/// let mut out = String::new();
/// c.write_boolean(&mut out, true).unwrap();
/// assert_eq!(out, "1");
/// ```
#[rapidquery_macros::define_options]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct DialectConfig {
    pub placeholder_character: char,

    /// Positional parameter syntax
    ///
    /// - Simple: `'?'`
    /// - Numbered: `'?1'`
    #[enum_(#[default] Simple, Numbered)]
    pub placeholder_syntax: PlaceholderSyntax,

    /// Where an inlined `LIKE` `ESCAPE` clause is legal.
    ///
    /// Empty writes no `ESCAPE` and rejects a set escape character.
    /// `SCALAR | ANY` allows both a scalar pattern and native `LIKE ANY`.
    ///
    /// - SCALAR: Scalar `LIKE` and an `OR` expansion.
    /// - ANY: Native `LIKE ANY (p, …) ESCAPE`.
    #[bit(SCALAR = 1 << 0, ANY = 1 << 1)]
    pub like_escape: LikeEscape,

    /// How `LIKE ANY` / `ILIKE ANY` is rendered.
    ///
    /// - OrExpansion: `(x LIKE p1 OR x LIKE p2)`. Useful when it isn't supported natively by dialect.
    /// - ArrayConstructor: `LIKE ANY (ARRAY[p, …])`.
    /// - PatternList: `LIKE ANY (p, …)`.
    #[enum_(#[default] OrExpansion, ArrayConstructor, PatternList)]
    pub like_any: LikeAny,

    /// Does dialect support `ILIKE` expression or we should fallback to another options?
    ///
    /// - Standard
    /// - UseLikeLower: Use `LIKE` expression and `LOWER()` function
    /// - UseLike: Fallback to `LIKE` expression
    #[enum_(#[default] Standard, UseLikeLower, UseLike)]
    pub ilike_expression: ILikeExpression,

    /// How regex `RLike` is spelled.
    ///
    /// - Unsupported: Is not supported.
    /// - Standard: `expr [NOT] RLIKE pattern`
    /// - Tilde: `expr ~ pattern` / `expr !~ pattern`
    /// - RegexpLike: `REGEXP_LIKE(expr, pattern)`
    /// - RegexpContains: `REGEXP_CONTAINS(expr, pattern)`
    /// - RegexpMatches: `regexp_matches(expr, pattern)`
    #[enum_(#[default] Unsupported, Standard, Tilde, RegexpLike, RegexpContains, RegexpMatches)]
    pub rlike_expression: RLikeExpression,

    pub supports_similar_to: bool,

    /// The way dialect supports `{IGNORE|RESPECT} NULLS` expressions in `Function`s.
    ///
    /// - Unsupported: Is not supported
    /// - InsideParens: e.g. `FIRST_VALUE(x IGNORE NULLS)`
    /// - Postfix: e.g. `FIRST_VALUE(x) IGNORE NULLS`
    #[enum_(#[default] Unsupported, InsideParens, Postfix)]
    pub null_treatment: NullTreatment,

    /// Function `FILTER` syntax.
    ///
    /// - Unsupported: Is not supported.
    /// - Filter: e.g. FIRST_VALUE(x) FILTER (id > 10)
    /// - FilterWhere: e.g. FIRST_VALUE(x) FILTER (WHERE id > 10)
    /// - InsideWhere: e.g. FIRST_VALUE(x WHERE id > 10)
    #[enum_(Unsupported, Filter, #[default] FilterWhere, InsideWhere)]
    pub function_filter: FunctionFilter,

    /// Which bitwise operators a dialect can render.
    ///
    /// A missing flag rejects that operator. AND, OR, NOT, and the shifts share
    /// one spelling ([`BitCall`]). XOR uses [`XorSpelling`].
    #[bit(AND = 1 << 0, OR = 1 << 1, NOT = 1 << 2, SHL = 1 << 3, SHR = 1 << 4)]
    pub bitwise_operations: BitwiseOperations,

    /// Spelling shared by `&`, `|`, `~`, `<<`, and `>>`.
    ///
    /// - Infix: `& | ~ << >>`.
    /// - BitFunction: `BITAND` / `BITOR` / `BITNOT` / `BITSHIFTLEFT` / `BITSHIFTRIGHT`.
    /// - ShiftFunction: `BITAND` / `BITOR` / `BITNOT` / `SHIFTLEFT` / `SHIFTRIGHT`.
    #[enum_(#[default] Infix, BitFunction, ShiftFunction)]
    pub bitwise_call_syntax: BitwiseCallSyntax,

    /// How `^` is spelled when [`BitOp::XOR`] is set.
    ///
    /// - Unsupported
    /// - Caret: `left ^ right`.
    /// - Hash: `left # right`. PostgreSQL `^` is exponent.
    /// - Function: `xor(left, right)`.
    /// - BitFunction: `BITXOR(left, right)`.
    #[enum_(Unsupported, #[default] Caret, Hash, Function, BitFunction)]
    pub bitwise_xor_spelling: BitwiseXorSpelling,

    /// Delimiters wrapped around a quoted identifier.
    ///
    /// - DoubleQuote: `"name"`
    /// - Backtick: `` `name` ``
    /// - SquareBracket: `[name]`
    #[enum_(#[default] DoubleQuote, Backtick, SquareBracket)]
    pub identifier_format: IdentifierFormat,

    /// How to escape identifiers.
    ///
    /// - Double: Escape identifier delimiters with doubling.
    /// - Backslash: Escape identifier delimiters with backslash.
    #[enum_(#[default] Double, Backslash)]
    pub identifier_escape: IdentifierEscape,

    /// Delimiters wrapped around a literal string.
    ///
    /// - SingleQuote: `'string'`
    /// - PrefixBigE: `E'string'`
    /// - PrefixBigN: `N'string'`
    #[enum_(#[default] SingleQuote, PrefixBigE, PrefixBigN)]
    pub string_format: StringFormat,

    /// How to escape literal strings.
    ///
    /// - Quote: `'` → `''`. Other characters are unchanged.
    /// - BackslashHex: `\\`, `\'`, `\n`, `\r`, `\t`; other C0/DEL as `\xNN`.
    /// - BackslashMysql: Also `\"` and `\Z` for 0x1A; tab and other controls pass through.
    /// - BackslashUnicode: Other C0/DEL as `\uNNNN`. No `\"`.
    /// - BackslashHexZero: Other C0/DEL as `\xNN`.
    /// - BackslashUnicodeQuote: [`Self::BackslashUnicode`] plus `\"`.
    #[enum_(#[default] Quote, BackslashHex, BackslashMysql, BackslashUnicode, BackslashHexZero, BackslashUnicodeQuote)]
    pub string_escape: StringEscape,

    /// How binary operation StringConcat (`||`) is spelled.
    ///
    /// - Pipes: `left || right`.
    /// - Plus: `left + right`. SQL Server `||` is 2025+; classic concat is `+`.
    /// - ConcatFunction: `CONCAT(left, right)`. MySQL `||` is logical OR unless `PIPES_AS_CONCAT`.
    #[enum_(#[default] Pipes, Plus, ConcatFunction)]
    pub string_concat: StringConcat,

    /// How `TRY_CAST` is spelled.
    ///
    /// - Unsupported: Is not supported
    /// - TryCast: `TRY_CAST(expr AS type)`.
    /// - TryCastWord: `TRYCAST(expr AS type)`.
    /// - SafeCast: `SAFE_CAST(expr AS type)`. BigQuery's spelling of a null-on-failure cast.
    #[enum_(Unsupported, #[default] TryCast, TryCastWord, SafeCast)]
    pub cast_try_cast: TryCast,

    pub cast_double_colon: bool,

    /// How a tuple / row value is spelled.
    ///
    /// Which arities are legal is [`TupleArity`], not another syntax variant.
    ///
    /// - Unsupported: Is not supported.
    /// - Row: `ROW(...)`.
    /// - Struct: `struct(...)`.
    /// - Parentheses: `(a, b, ...)`.
    #[enum_(#[default] Unsupported, Row, Struct, Parentheses)]
    pub tuple_syntax: TupleSyntax,

    /// Arities below 2 that [`TupleSyntax`] accepts.
    ///
    /// Arity 2 and above is legal whenever the syntax is not [`TupleSyntax::Unsupported`].
    /// Empty means a tuple needs at least two elements.
    ///
    /// - EMPTY: Arity 0.
    /// - ONE: Arity 1.
    #[bit(EMPTY = 1 << 0, ONE = 1 << 1)]
    pub tuple_arity: TupleArity,

    /// How a date literal is spelled.
    ///
    /// - Quoted: Quoted string.
    /// - Keyword: `DATE`, a space, then a quoted literal.
    #[enum_(#[default] Quoted, Keyword)]
    pub date_format: DateFormat,

    /// How a time literal is spelled.
    ///
    /// - Quoted: Quoted string.
    /// - Keyword: `TIME`, a space, then a quoted literal.
    #[enum_(#[default] Quoted, Keyword)]
    pub time_format: TimeFormat,

    /// How a timestamp literal is spelled.
    ///
    /// Naive values use `TIMESTAMP` when a keyword is configured. Aware values
    /// (`datetime` with a tzinfo, or a UTC epoch number) use the keyword this
    /// variant names.
    ///
    /// - Quoted: Both naive and aware values are a quoted string. No keyword.
    /// - Timestamptz: Naive: `TIMESTAMP '…'`. Aware: `TIMESTAMPTZ '…'`.
    /// - WithTimezone: Naive: `TIMESTAMP '…'`. Aware: `TIMESTAMP WITH TIMEZONE '…'`.
    #[enum_(Quoted, Timestamptz, #[default] WithTimezone)]
    pub timestamp_format: TimestampFormat,

    /// How a boolean literal is spelled.
    ///
    /// - Keyword: `TRUE` / `FALSE`
    /// - Integer: `1` / `0`
    #[enum_(#[default] Keyword, Integer)]
    pub bool_format: BoolFormat,

    /// The way dialect accepts literal bit values
    ///
    /// - Unsupported: Is not supported.
    /// - PrefixBigB: e.g. `B'101'`
    /// - PrefixSmallB: e.g. `b'101'`
    /// - PostfixBigB: e.g. `'101'B`
    #[enum_(Unsupported, #[default] PrefixBigB, PrefixSmallB, PostfixBigB)]
    pub bit_format: BitFormat,

    /// The way dialect accepts literal bytes values.
    ///
    /// - PrefixBigX: `X'DEADBEEF'`
    /// - PrefixZeroX: `0xDEADBEEF`
    /// - SlashXBytea: `'\xDEADBEEF'::bytea`
    /// - PrefixBigBSlashX: `B'\xNN…'` (each byte as `\xNN`)
    /// - HexToRaw: `HEXTORAW('DEADBEEF')`
    /// - Unhex: `UNHEX('DEADBEEF')`
    /// - ToVarbyte: `TO_VARBYTE('DEADBEEF', 'hex')`
    /// - PostfixXbv: `'DEADBEEF'XBV`
    /// - ToBinary: `TO_BINARY('DEADBEEF', 'HEX')`
    /// - SlashXBlob: `'\xHH…'::BLOB`; empty is `''::BLOB`
    #[enum_(#[default] PrefixBigX, PrefixZeroX, SlashXBytea, PrefixBigBSlashX, HexToRaw, Unhex, ToVarbyte, PostfixXbv, ToBinary, SlashXBlob)]
    pub bytes_format: BytesFormat,

    /// How an inet literal is spelled.
    ///
    /// - Unsupported: Is not supported.
    /// - Quoted: `'192.168.1.1/32'`
    /// - Cast: `'192.168.1.1/32'::inet`
    #[enum_(#[default] Unsupported, Quoted, Cast)]
    pub inet_format: InetFormat,

    /// How a MAC address literal is spelled.
    ///
    /// - Unsupported: Is not supported.
    /// - Quoted: `'01:23:45:67:89:AB'`
    /// - Cast: `'01:23:45:67:89:AB'::macaddr`
    #[enum_(#[default] Unsupported, Quoted, Cast)]
    pub macaddr_format: MacAddrFormat,

    /// How a UUID literal is spelled.
    ///
    /// - Quoted: Quoted hyphenated text.
    /// - UuidKeyword: `UUID '…'`
    /// - CastUuid: `'…'::uuid`
    #[enum_(#[default] Quoted, UuidKeyword, CastUuid)]
    pub uuid_format: UuidFormat,

    /// How a JSON literal is spelled.
    ///
    /// - Quoted: Quoted JSON text.
    /// - JsonKeyword: `JSON '…'`
    /// - ParseJson: `PARSE_JSON('…')`
    #[enum_(#[default] Quoted, JsonKeyword, ParseJson)]
    pub json_format: JsonFormat,

    /// The way dialect accepts array values.
    ///
    /// - Unsupported: Is not supported.
    /// - Constructor: `ARRAY[…]`
    /// - List: `[…]`
    /// - Function: `array(…)`
    #[enum_(#[default] Unsupported, Constructor, List, Function)]
    pub array_format: ArrayFormat,

    /// How binary operation mod (`%`) is spelled.
    ///
    /// - Percent: `left % right`.
    /// - Function: `MOD(left, right)`.
    /// - Infix: `left MOD right`.
    #[enum_(#[default] Percent, Function, Infix)]
    pub modulo_spelling: ModuloSpelling,

    /// How `IS [NOT] DISTINCT FROM` is spelled.
    ///
    /// - Unsupported: Is not supported
    /// - Standard: `left IS [NOT] DISTINCT FROM right`.
    /// - NullSafeEqual: `<=>` is null-safe equal (`IS NOT DISTINCT FROM`). `IS DISTINCT FROM` is `NOT (left <=> right)`.
    #[enum_(Unsupported, #[default] Standard, NullSafeEqual)]
    pub is_distinct_from_spelling: IsDistinctFromSpelling,

    /// How `TRIM` is spelled.
    ///
    /// - SqlFrom: `TRIM([LEADING|TRAILING] [chars FROM] expr)`.
    /// - Function: `TRIM` / `LTRIM` / `RTRIM` `(expr[, chars])`.
    /// - SpacesOnly: `TRIM` / `LTRIM` / `RTRIM` `(expr)`. A character set raises.
    #[enum_(#[default] SqlFrom, Function, SpacesOnly)]
    pub trim_spelling: TrimSpelling,

    /// How `COLLATE` is spelled.
    ///
    /// - Unsupported: Is not supported
    /// - InfixIdent: `expr COLLATE ident`.
    /// - FunctionString: `COLLATE(expr, 'body')`. The identifier body is a string literal.
    #[enum_(Unsupported, #[default] InfixIdent, FunctionString)]
    pub collate_spelling: CollateSpelling,

    // --- DDL Schema ---
    /// Supported independant schema features, or is it even supported?
    ///
    /// - UNSUPPORTED: Means dialect does not support schema.
    /// - IF_NOT_EXISTS: Is dialect support `IF NOT EXISTS` clause?
    /// - EMBEDDED_DDL: Is dialect support embedded DDL statements in `CREATE SCHEMA`?
    /// - IS_DATABASE: Schema has same meaning as database in this dialect.
    /// - DROP_CASCADE: Supports `CASCADE` keyword in `DROP SCHEMA`.
    /// - DROP_RESTRICT: Supports `RESTRICT` keyword in `DROP SCHEMA`.
    #[bit(UNSUPPORTED=1<<0, IF_NOT_EXISTS=1<<1, EMBEDDED_DDL=1<<2, IS_DATABASE=1<<3, DROP_CASCADE=1<<4, DROP_RESTRICT=1<<5)]
    pub schema_features: SchemaFeatures,

    /// `AUTHORIZATION` clause style. How the schema owner is specified.
    ///
    /// - Unsupported: Means dialect does not support `AUTHORIZATION` for schema.
    /// - Optional: Means `AUTHORIZATION` clause is supported and it's optional.
    #[enum_(Unsupported, #[default] Optional)]
    pub schema_authorization_style: SchemaAuthorizationStyle,

    /// How schema location is expressed.
    ///
    /// - Unsupported: Means dialect does not support location for schema at all.
    /// - LocationKeyword: `LOCATION location` clause.
    /// - ManagedLocationKeyword: `MANAGED LOCATION location` clause.
    /// - ExternalVolume: `EXTERNAL_VOLUME = location` clause.
    #[enum_(Unsupported, #[default] LocationKeyword, ManagedLocationKeyword, ExternalVolume)]
    pub schema_location_style: SchemaLocationStyle,
}

impl DialectConfig {
    /// Write a boolean literal (`TRUE`/`FALSE` or `1`/`0`).
    #[inline]
    pub fn write_boolean<W: std::fmt::Write>(&self, writer: &mut W, value: bool) -> Result<(), FormatError> {
        match (self.bool_format, value) {
            (BoolFormat::Integer, false) => writer.write_str("0")?,
            (BoolFormat::Integer, true) => writer.write_str("1")?,
            (BoolFormat::Keyword, false) => writer.write_str("FALSE")?,
            (BoolFormat::Keyword, true) => writer.write_str("TRUE")?,
        }

        Ok(())
    }

    /// Write `string` with special characters neutralized, without wrapping it.
    #[inline]
    pub(super) fn escape_string<W: std::fmt::Write>(&self, writer: &mut W, string: &str) -> Result<(), FormatError> {
        let result = match self.string_escape {
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
    pub fn write_bits<W: std::fmt::Write>(&self, writer: &mut W, bits: &str) -> Result<(), FormatError> {
        debug_assert!(
            bits.chars().all(|c| c == '0' || c == '1'),
            "write_bits: invalid input: {bits}"
        );
        match self.bit_format {
            BitFormat::Unsupported => Err(FormatError::Unsupported("BitSerializer")),
            BitFormat::PrefixBigB => write!(writer, "B'{bits}'").map_err(Into::into),
            BitFormat::PrefixSmallB => write!(writer, "b'{bits}'").map_err(Into::into),
            BitFormat::PostfixBigB => write!(writer, "'{bits}'B").map_err(Into::into),
        }
    }

    /// Write `bytes` as a native blob/bytea/varbyte literal.
    #[inline]
    pub fn write_bytes<W: std::fmt::Write>(&self, writer: &mut W, bytes: &[u8]) -> Result<(), FormatError> {
        let result = match self.bytes_format {
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
