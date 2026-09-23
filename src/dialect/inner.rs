use std::fmt::{self, Write};

use crate::internal::utils;

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
    OutsideParens,
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
    /// `'` → `''`. Other characters, including NUL, are unchanged.
    Quote,
    /// `'` → `''`. NUL is deleted.
    QuoteStripNul,
    /// `\\`, `\'`, `\n`, `\r`, `\t`; NUL and other C0/DEL as `\xNN`.
    BackslashHex,
    /// MySQL: also `\"` and `\Z` for 0x1A; NUL as `\0`; tab and other controls pass through.
    BackslashMysql,
    /// Hive: `\0` for NUL; other C0/DEL as `\uNNNN`. No `\"`.
    BackslashUnicode,
    /// Snowflake: `\0` for NUL; other C0/DEL as `\xNN`.
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
            Self::QuoteStripNul => {
                const REPLACEMENTS: [(u8, &str); 2] = [(b'\'', "''"), (b'\0', "")];
                utils::replace_ascii_to(string, &REPLACEMENTS, writer)
            }
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
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dialect {
    /// Positional parameter syntax.
    pub positional_parameter: PositionalParameter,
    /// How the dialect accepts literal bit values if supported.
    ///
    /// Set [`Option::None`] if literal bit isn't supported.
    pub bit_format: Option<BitFormat>,
    /// Whether the dialect supports `ARRAY[...]` data type.
    pub supports_array: bool,
    /// Whether the dialect supports `INET`/`CIDR` data types.
    pub supports_inet: bool,
    /// Whether the dialect supports `MACADDR` data type.
    pub supports_mac_address: bool,
    /// How the dialect supports `{IGNORE|RESPECT} NULLS` expressions in `Function`s.
    pub null_treatment_syntax: NullTreatmentSyntax,
    /// How `LIKE ANY` / `ILIKE ANY` is rendered.
    pub like_any_syntax: LikeAnySyntax,
    /// How the dialect accepts literal boolean values.
    pub bool_format: BoolFormat,
    /// Specifies how delimiters should wrap around a quoted identifier.
    pub identifier_format: IdentifierFormat,
    /// Specifies how to escape identifiers
    pub identifier_escape: IdentifierEscape,
    /// Specifies how delimiters should wrap around a literal string value.
    pub string_format: StringFormat,
    /// Specifies how to escape literal string value.
    pub string_escape: StringEscape,
    /// How the dialect accepts literal bytes values.
    pub bytes_format: BytesFormat,
}

impl Dialect {
    /// Write a boolean literal (`TRUE`/`FALSE` or `1`/`0`).
    #[inline]
    pub fn format_boolean<W: fmt::Write>(&self, writer: &mut W, value: bool) -> fmt::Result {
        match (self.bool_format, value) {
            (BoolFormat::Int, false) => write!(writer, "0"),
            (BoolFormat::Int, true) => write!(writer, "1"),
            (BoolFormat::Keyword, false) => write!(writer, "FALSE"),
            (BoolFormat::Keyword, true) => write!(writer, "TRUE"),
        }
    }

    /// Write `string` with special characters neutralized, without wrapping it.
    #[inline]
    pub fn escape_string<W: fmt::Write>(&self, writer: &mut W, string: &str) -> fmt::Result {
        self.string_escape.write(writer, string)
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
    pub fn write_string_literal<W: fmt::Write>(&self, writer: &mut W, raw: &str) -> fmt::Result {
        match self.string_format {
            StringFormat::PrefixBigE => writer.write_str("E'")?,
            StringFormat::PrefixBigN => writer.write_str("N'")?,
            StringFormat::SingleQuote => writer.write_char('\'')?,
        }
        self.escape_string(writer, raw)?;
        writer.write_char('\'')
    }

    /// Write a bit-string (`'0'`/`'1'` characters) as a native literal.
    ///
    /// Native formats embed `bits` as-is. Callers must pass only `'0'` and `'1'`;
    /// that is checked with `debug_assert!`.
    ///
    /// # Panics
    ///
    /// Panics when the dialect does not support literal bit values and
    /// [`Self::bit_format`] is [`None`]. Callers should check
    /// [`Option::is_some`] on [`Self::bit_format`] before calling this method.
    #[inline]
    pub fn format_bits<W: fmt::Write>(&self, writer: &mut W, bits: &str) -> fmt::Result {
        debug_assert!(
            bits.chars().all(|c| c == '0' || c == '1'),
            "format_bits: invalid input: {bits}"
        );
        match self.bit_format {
            Some(format) => format.write(writer, bits),
            None => panic!(
                "format_bits: dialect does not support literal bit values (bit_format is None)"
            ),
        }
    }

    /// Write `bytes` as a native blob/bytea/varbyte literal.
    #[inline]
    pub fn format_bytes<W: fmt::Write>(&self, writer: &mut W, bytes: &[u8]) -> fmt::Result {
        self.bytes_format.write(writer, bytes)
    }
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

    fn collect(write: impl FnOnce(&mut String) -> fmt::Result) -> String {
        let mut buf = String::new();
        write(&mut buf).unwrap();
        buf
    }

    fn escape(kind: StringEscape, string: &str) -> String {
        collect(|buf| kind.write(buf, string))
    }

    fn bytes(kind: BytesFormat, bytes: &[u8]) -> String {
        collect(|buf| kind.write(buf, bytes))
    }

    fn base() -> Dialect {
        Dialect {
            positional_parameter: PositionalParameter::Simple(':'),
            bit_format: None,
            supports_array: false,
            supports_inet: false,
            supports_mac_address: false,
            null_treatment_syntax: NullTreatmentSyntax::OutsideParens,
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Backslash,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::Quote,
            bytes_format: BytesFormat::PrefixBigX,
        }
    }

    #[test]
    fn quote_vs_quote_strip_nul() {
        for kind in [StringEscape::Quote, StringEscape::QuoteStripNul] {
            assert_eq!(escape(kind, "O'Brien"), "O''Brien");
            assert_eq!(escape(kind, "'"), "''");
        }
        assert_eq!(escape(StringEscape::Quote, "a\0b"), "a\0b");
        assert_eq!(escape(StringEscape::QuoteStripNul, "a\0b"), "ab");
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
    }

    #[test]
    fn bit_format_literals() {
        let bits = "10110101";
        assert_eq!(
            collect(|buf| BitFormat::PrefixBigB.write(buf, bits)),
            "B'10110101'"
        );
        assert_eq!(
            collect(|buf| BitFormat::PrefixSmallB.write(buf, bits)),
            "b'10110101'"
        );
        assert_eq!(
            collect(|buf| BitFormat::PostfixBigB.write(buf, bits)),
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
    #[should_panic(expected = "bit_format is None")]
    fn format_bits_panics_when_unsupported() {
        let cfg = base();
        let _ = cfg.format_bits(&mut String::new(), "1");
    }

    #[test]
    fn write_string_literal_prefix_big_e_backslash_hex() {
        let mut cfg = base();
        cfg.string_format = StringFormat::PrefixBigE;
        cfg.string_escape = StringEscape::BackslashHex;
        assert_eq!(
            collect(|buf| cfg.write_string_literal(buf, "hello'world\n")),
            "E'hello\\'world\\n'"
        );
    }
}
