use crate::error;
use crate::internal::template;
use crate::internal::writer;

use super::base;

impl base::LiteralConfig {
    /// Write a boolean literal (`TRUE`/`FALSE` or `1`/`0`).
    #[inline]
    pub fn write_boolean<W: writer::FastWrite>(&self, writer: &mut W, value: bool) {
        match (self.boolean, value) {
            (base::BoolFormat::Integer, false) => {
                writer.write('0');
            }
            (base::BoolFormat::Integer, true) => {
                writer.write('1');
            }
            (base::BoolFormat::Keyword, false) => {
                writer.write_str("FALSE");
            }
            (base::BoolFormat::Keyword, true) => {
                writer.write_str("TRUE");
            }
        }
    }

    /// Write a bit-string (`'0'`/`'1'` characters) as a native literal.
    ///
    /// Native formats embed `bits` as-is. Callers must pass only `'0'` and `'1'`;
    /// that is checked with `debug_assert!`.
    #[inline]
    pub fn write_bits<W: writer::FastWrite>(&self, writer: &mut W, bits: &str) -> error::Result<()> {
        // FIXME: return an error for invalid bits
        debug_assert!(
            bits.chars().all(|c| c == '0' || c == '1'),
            "write_bits: invalid input: {bits}"
        );

        if let Some(template) = &self.bit {
            template.write_to(writer, |_, out| out.write_str(bits));
            Ok(())
        } else {
            Err(error::Error::Unsupported {
                feature: "bit strings are not supported by this dialect",
                fix: "set literal.bit to a template such as \"B'{}'\"",
            })
        }
    }

    /// Write `bytes` as a native blob/bytea/varbyte literal.
    #[inline]
    pub fn write_bytes<W: writer::FastWrite>(&self, writer: &mut W, bytes: &[u8]) {
        self.bytes.write_to(
            writer,
            // hex: bytes as hex (e.g. DEADBEEF)
            // slashx: each byte as \xNN
            |key, out| match key {
                "hex" => bytes_to_hex(out, bytes),
                "slashx" => write_bytes_slash_x(out, bytes),
                _ => unreachable!("invalid key - write_bytes"),
            },
        );
    }
}

#[pyo3::pymethods]
impl base::LiteralConfig {
    #[pyo3(signature = (value, /))]
    fn serialize_boolean(&self, value: bool) -> String {
        let mut buf = String::new();
        self.write_boolean(&mut buf, value);
        buf
    }

    #[pyo3(signature = (value, /))]
    fn serialize_bits(&self, value: &str) -> error::Result<String> {
        if value.is_empty() || !value.bytes().all(|b| b == b'0' || b == b'1') {
            return Err(error::Error::InvalidBitString {
                value: value.to_owned(),
            });
        }

        let mut buf = String::new();
        self.write_bits(&mut buf, value)?;
        Ok(buf)
    }

    #[pyo3(signature = (value, /))]
    fn serialize_bytes(&self, value: pyo3::Bound<'_, pyo3::types::PyBytes>) -> String {
        use pyo3::types::PyBytesMethods;

        let mut buf = String::new();
        self.write_bytes(&mut buf, value.as_bytes());
        buf
    }
}

impl base::StringConfig {
    /// Escape `raw` and wrap it as a complete string literal.
    #[inline]
    pub fn write_string_literal<W: writer::FastWrite>(&self, writer: &mut W, raw: &str) -> error::Result<()> {
        // FIXME: we can accept NUL on some dialects
        if raw.contains('\0') {
            return Err(error::Error::NulInString);
        }

        self.template.write_to(
            writer,
            // single: Escape ' → ''. Other characters are unchanged.
            // double: Escape " → "". Other characters are unchanged.
            // hex: Escape \\, \', \n, \r, \t; other C0/DEL as \xNN.
            // hex_quote: hex plus \".
            // mysql: (MySQL-specific) Like hex, but also \" and \Z for 0x1A; tab and other controls pass through.
            // unicode: Other C0/DEL as \uNNNN. No \".
            // unicode_quote: unicode plus \".
            |key, out| match key {
                "single" => template::replace_to(raw, '\'', "''", out),
                "double" => template::replace_to(raw, '"', "\"\"", out),
                "hex" => write_backslash_hex(out, raw, false),
                "hex_quote" => write_backslash_hex(out, raw, true),
                "mysql" => write_backslash_mysql(out, raw),
                "unicode" => write_backslash_unicode(out, raw, false),
                "unicode_quote" => write_backslash_unicode(out, raw, true),
                _ => unreachable!("invalid key - write_string_literal"),
            },
        );
        Ok(())
    }
}

#[pyo3::pymethods]
impl base::StringConfig {
    #[pyo3(signature = (value, /))]
    fn serialize_string(&self, value: &str) -> error::Result<String> {
        let mut buf = String::new();
        self.write_string_literal(&mut buf, value)?;
        Ok(buf)
    }
}

impl base::PyDialect {
    /// Escape `ident` and wrap it in this dialect's identifier delimiters.
    #[inline]
    pub fn write_identifier<W: writer::FastWrite>(&self, writer: &mut W, ident: &str) {
        self.identifier.write_to(
            writer,
            // double:   `"` -> `""`
            // single:   `'` -> `''`
            // bracket:  `]` -> `]]`
            // backtick: `` ` `` -> ``` `` ```
            // slash*:   backslash style, see `write_identifier_backslash_escape`
            |key, out| match key {
                "double" => template::replace_to(ident, '"', "\"\"", out),
                "single" => template::replace_to(ident, '\'', "''", out),
                "bracket" => template::replace_to(ident, ']', "]]", out),
                "backtick" => template::replace_to(ident, '`', "``", out),
                "slashdouble" => write_identifier_backslash_escape(out, ident, b'"', "\\\""),
                "slashsingle" => write_identifier_backslash_escape(out, ident, b'\'', "\\'"),
                "slashbracket" => write_identifier_backslash_escape(out, ident, b']', "\\]"),
                "slashbacktick" => write_identifier_backslash_escape(out, ident, b'`', "\\`"),
                _ => unreachable!("invalid key - write_identifier"),
            },
        )
    }
}

const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

/// Write `byte` as two uppercase hex digits.
#[inline]
fn write_hex2<W: writer::FastWrite>(writer: &mut W, byte: u8) {
    let buf = [HEX_DIGITS[(byte >> 4) as usize], HEX_DIGITS[(byte & 0x0F) as usize]];
    // SAFETY: HEX_DIGITS is ASCII, so `buf` is valid UTF-8.
    writer.write_str(unsafe { std::str::from_utf8_unchecked(&buf) })
}

/// Write `bytes` as a contiguous uppercase hex run (`DEADBEEF`).
///
/// 64 raw bytes fill a 128-byte stack buffer and are written once.
#[inline]
fn bytes_to_hex<W: writer::FastWrite>(writer: &mut W, bytes: &[u8]) {
    let mut buf = [0u8; 128];
    for chunk in bytes.chunks(64) {
        for (i, &byte) in chunk.iter().enumerate() {
            let o = i * 2;
            buf[o] = HEX_DIGITS[(byte >> 4) as usize];
            buf[o + 1] = HEX_DIGITS[(byte & 0x0F) as usize];
        }
        let n = chunk.len() * 2;
        // SAFETY: `buf[..n]` is filled only from `HEX_DIGITS` (ASCII).
        writer.write_str(unsafe { std::str::from_utf8_unchecked(&buf[..n]) });
    }
}

/// Write each byte as `\xNN` (uppercase hex).
///
/// 32 raw bytes fill a 128-byte stack buffer and are written once.
#[inline]
fn write_bytes_slash_x<W: writer::FastWrite>(writer: &mut W, bytes: &[u8]) {
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
        writer.write_str(unsafe { std::str::from_utf8_unchecked(&buf[..n]) });
    }
}

/// Write `\xNN` for a single byte (uppercase hex).
#[inline]
fn write_byte_hex_escape<W: writer::FastWrite>(writer: &mut W, byte: u8) {
    writer.write_str("\\x");
    write_hex2(writer, byte)
}

/// Write `\uNNNN` for a codepoint that fits in 16 bits.
#[inline]
fn write_unicode4_escape<W: writer::FastWrite>(writer: &mut W, code: u32) {
    debug_assert!(code <= 0xFFFF);
    writer.write_str("\\u");
    write_hex2(writer, (code >> 8) as u8);
    write_hex2(writer, code as u8)
}

/// Backslash-style identifier escape for the delimiter byte `quote`.
///
/// `quote` must be an ASCII byte (so it is always a char boundary in UTF-8) and
/// `escaped` is what it becomes, e.g. `b'"'` and `"\\\""`. The backslash itself
/// is always escaped, otherwise an identifier ending in `\` would swallow the
/// closing delimiter. Other quote characters are left as they are.
fn write_identifier_backslash_escape<W: writer::FastWrite>(
    writer: &mut W,
    ident: &str,
    quote: u8,
    escaped: &'static str,
) {
    debug_assert!(quote.is_ascii());
    write_span_escapes(writer, ident, |b| match b {
        b'\\' => SpanEscape::Literal("\\\\"),
        b if b == quote => SpanEscape::Literal(escaped),
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
fn write_byte_span<W: writer::FastWrite>(writer: &mut W, bytes: &[u8], from: usize, to: usize) {
    if from == to {
        return;
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
fn write_span_escapes<W: writer::FastWrite>(writer: &mut W, string: &str, mut classify: impl FnMut(u8) -> SpanEscape) {
    let bytes = string.as_bytes();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        match classify(bytes[i]) {
            SpanEscape::Keep => i += 1,
            SpanEscape::Literal(repl) => {
                write_byte_span(writer, bytes, start, i);
                writer.write_str(repl);
                i += 1;
                start = i;
            }
            SpanEscape::HexByte => {
                write_byte_span(writer, bytes, start, i);
                write_byte_hex_escape(writer, bytes[i]);
                i += 1;
                start = i;
            }
            SpanEscape::Unicode4 => {
                write_byte_span(writer, bytes, start, i);
                write_unicode4_escape(writer, bytes[i] as u32);
                i += 1;
                start = i;
            }
        }
    }
    write_byte_span(writer, bytes, start, bytes.len())
}

fn write_backslash_hex<W: writer::FastWrite>(writer: &mut W, string: &str, escape_double_quote: bool) {
    write_span_escapes(writer, string, |b| match b {
        b'\\' => SpanEscape::Literal("\\\\"),
        b'\'' => SpanEscape::Literal("\\'"),
        b'"' if escape_double_quote => SpanEscape::Literal("\\\""),
        b'\n' => SpanEscape::Literal("\\n"),
        b'\r' => SpanEscape::Literal("\\r"),
        b'\t' => SpanEscape::Literal("\\t"),
        b if b < 32 || b == 127 => SpanEscape::HexByte,
        _ => SpanEscape::Keep,
    })
}

fn write_backslash_mysql<W: writer::FastWrite>(writer: &mut W, string: &str) {
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

fn write_backslash_unicode<W: writer::FastWrite>(writer: &mut W, string: &str, escape_double_quote: bool) {
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

#[cfg(test)]
mod tests {
    use crate::dialect::PyDialect;
    use crate::dialect::StringConfig;
    use crate::error;
    use crate::internal::template::Template;

    const KEYS: &[(&str, bool)] = &[
        ("double", false),
        ("single", false),
        ("bracket", false),
        ("backtick", false),
        ("slashdouble", false),
        ("slashsingle", false),
        ("slashbracket", false),
        ("slashbacktick", false),
    ];

    fn render(tmpl: &str, ident: &str) -> String {
        let d = PyDialect {
            identifier: Template::parse(tmpl, KEYS, 1, 1).unwrap(),
            ..Default::default()
        };

        let mut out = String::new();
        d.write_identifier(&mut out, ident);
        out
    }

    #[test]
    fn doubling_keys() {
        assert_eq!(render(r#""{double}""#, r#"a"b"#), r#""a""b""#);
        assert_eq!(render("'{single}'", "a'b"), "'a''b'");
        assert_eq!(render("[{bracket}]", "a]b"), "[a]]b]");
        assert_eq!(render("`{backtick}`", "a`b"), "`a``b`");
    }

    #[test]
    fn doubling_keys_leave_other_delimiters_alone() {
        assert_eq!(render(r#""{double}""#, "it's `x` [y]"), r#""it's `x` [y]""#);
        assert_eq!(render("`{backtick}`", r#"a"b"#), r#"`a"b`"#);
        assert_eq!(render("[{bracket}]", "a[b"), "[a[b]");
    }

    #[test]
    fn slash_keys_escape_delimiter() {
        assert_eq!(render(r#""{slashdouble}""#, r#"a"b"#), r#""a\"b""#);
        assert_eq!(render("'{slashsingle}'", "a'b"), r"'a\'b'");
        assert_eq!(render("[{slashbracket}]", "a]b"), r"[a\]b]");
        assert_eq!(render("`{slashbacktick}`", "a`b"), r"`a\`b`");
    }

    #[test]
    fn slash_keys_escape_the_backslash_itself() {
        // An identifier ending in `\` must not escape the closing delimiter.
        assert_eq!(render(r#""{slashdouble}""#, r"a\"), r#""a\\""#);
        assert_eq!(render("`{slashbacktick}`", r"a\`"), r"`a\\\``");
    }

    #[test]
    fn slash_keys_escape_control_characters() {
        assert_eq!(render("`{slashbacktick}`", "a\nb\tc\rd"), r"`a\nb\tc\rd`");
        assert_eq!(render("`{slashbacktick}`", "a\u{1}b"), r"`a\x01b`");
    }

    #[test]
    fn non_ascii_is_kept() {
        assert_eq!(render("`{slashbacktick}`", "é`"), r"`é\``");
        assert_eq!(render("`{backtick}`", "日本`"), "`日本```");
    }

    const STRING_KEYS: &[(&str, bool)] = &[
        ("single", false),
        ("double", false),
        ("hex", false),
        ("hex_quote", false),
        ("mysql", false),
        ("unicode", false),
        ("unicode_quote", false),
    ];

    fn render_string(tmpl: &str, raw: &str) -> error::Result<String> {
        let c = StringConfig {
            template: Template::parse(tmpl, STRING_KEYS, 1, 1).unwrap(),
            ..Default::default()
        };

        let mut out = String::new();
        c.write_string_literal(&mut out, raw)?;
        Ok(out)
    }

    #[test]
    fn string_doubling_keys() {
        assert_eq!(render_string("'{single}'", "it's").unwrap(), "'it''s'");
        assert_eq!(
            render_string("\"{double}\"", "say \"hi\"").unwrap(),
            "\"say \"\"hi\"\"\""
        );
        // The other quote is left alone.
        assert_eq!(render_string("'{single}'", "a\"b").unwrap(), "'a\"b'");
        assert_eq!(render_string("\"{double}\"", "it's").unwrap(), "\"it's\"");
    }

    #[test]
    fn string_hex_keys() {
        assert_eq!(render_string("'{hex}'", "a'b\\c\n").unwrap(), r"'a\'b\\c\n'");
        assert_eq!(render_string("'{hex}'", "a\"b").unwrap(), "'a\"b'");
        assert_eq!(render_string("'{hex}'", "a\u{1}").unwrap(), r"'a\x01'");
        assert_eq!(render_string("\"{hex_quote}\"", "a\"b'c\\").unwrap(), r#""a\"b\'c\\""#);
        assert_eq!(render_string("\"{hex_quote}\"", "a\u{1}").unwrap(), r#""a\x01""#);
    }

    #[test]
    fn string_unicode_keys() {
        assert_eq!(render_string("'{unicode}'", "a\u{1}\"").unwrap(), "'a\\u0001\"'");
        assert_eq!(
            render_string("'{unicode_quote}'", "a\u{1}\"").unwrap(),
            "'a\\u0001\\\"'"
        );
    }

    #[test]
    fn string_mysql_key() {
        assert_eq!(render_string("'{mysql}'", "a\"b\u{1a}\t").unwrap(), "'a\\\"b\\Z\t'");
    }

    #[test]
    fn string_rejects_nul() {
        assert!(matches!(
            render_string("'{single}'", "a\0b"),
            Err(error::Error::NulInString)
        ));
    }
}
