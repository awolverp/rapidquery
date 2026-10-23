use crate::internal::template;

use super::base;
use super::errors;

impl base::LiteralConfig {
    /// Write a boolean literal (`TRUE`/`FALSE` or `1`/`0`).
    #[inline]
    pub fn write_boolean(&self, writer: &mut String, value: bool) {
        match (self.boolean, value) {
            (base::BoolFormat::Integer, false) => writer.push('0'),
            (base::BoolFormat::Integer, true) => writer.push('1'),
            (base::BoolFormat::Keyword, false) => writer.push_str("FALSE"),
            (base::BoolFormat::Keyword, true) => writer.push_str("TRUE"),
        }
    }

    /// Write a bit-string (`'0'`/`'1'` characters) as a native literal.
    ///
    /// Native formats embed `bits` as-is. Callers must pass only `'0'` and `'1'`;
    /// that is checked with `debug_assert!`.
    #[inline]
    pub fn write_bits(&self, writer: &mut String, bits: &str) -> Result<(), errors::FormatError> {
        debug_assert!(
            bits.chars().all(|c| c == '0' || c == '1'),
            "write_bits: invalid input: {bits}"
        );

        if let Some(template) = &self.bit {
            template.write_to(writer, |_, out| out.push_str(bits));
            Ok(())
        } else {
            Err(errors::FormatError::Unsupported("BitSerializer"))
        }
    }

    /// Write `bytes` as a native blob/bytea/varbyte literal.
    #[inline]
    pub fn write_bytes(&self, writer: &mut String, bytes: &[u8]) {
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

impl base::StringConfig {
    /// Escape `raw` and wrap it as a complete string literal.
    #[inline]
    pub fn write_string_literal(&self, writer: &mut String, raw: &str) -> Result<(), errors::FormatError> {
        if raw.contains('\0') {
            return Err(errors::FormatError::NulInString);
        }

        self.template.write_to(
            writer,
            // quote: Escape ' → ''. Other characters are unchanged.
            // hex: Escape \\, \', \n, \r, \t; other C0/DEL as \xNN.
            // mysql: (MySQL-specific) Like hex, but also \" and \Z for 0x1A; tab and other controls pass through.
            // unicode: Other C0/DEL as \uNNNN. No \".
            // hex_zero: Other C0/DEL as \xNN.
            // unicode_quote: unicode plus \".
            |key, out| match key {
                "quote" => template::replace_to(raw, '\'', "''", out),
                "hex" => write_backslash_hex(out, raw),
                "mysql" => write_backslash_mysql(out, raw),
                "unicode" => write_backslash_unicode(out, raw, false),
                "hex_zero" => write_backslash_hex_zero(out, raw),
                "unicode_quote" => write_backslash_unicode(out, raw, true),
                _ => unreachable!("invalid key - write_string_literal"),
            },
        );
        Ok(())
    }
}

impl base::PyDialect {
    /// Escape `ident` and wrap it in this dialect's identifier delimiters.
    #[inline]
    pub fn write_identifier(&self, writer: &mut String, ident: &str) {
        self.identifier.write_to(
            writer,
            // double: Escape identifier delimiters with doubling.
            // backslash: Escape identifier delimiters with backslash
            |key, out| match key {
                "double" => template::replace_to(ident, '"', "\"\"", out),
                "backslash" => write_identifier_backtick_escape(out, ident),
                _ => unreachable!("invalid key - write_identifier"),
            },
        )
    }
}

const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

/// Write `byte` as two uppercase hex digits.
#[inline]
fn write_hex2(writer: &mut String, byte: u8) {
    let buf = [HEX_DIGITS[(byte >> 4) as usize], HEX_DIGITS[(byte & 0x0F) as usize]];
    // SAFETY: HEX_DIGITS is ASCII, so `buf` is valid UTF-8.
    writer.push_str(unsafe { std::str::from_utf8_unchecked(&buf) })
}

/// Write `bytes` as a contiguous uppercase hex run (`DEADBEEF`).
///
/// 64 raw bytes fill a 128-byte stack buffer and are written once.
#[inline]
fn bytes_to_hex(writer: &mut String, bytes: &[u8]) {
    let mut buf = [0u8; 128];
    for chunk in bytes.chunks(64) {
        for (i, &byte) in chunk.iter().enumerate() {
            let o = i * 2;
            buf[o] = HEX_DIGITS[(byte >> 4) as usize];
            buf[o + 1] = HEX_DIGITS[(byte & 0x0F) as usize];
        }
        let n = chunk.len() * 2;
        // SAFETY: `buf[..n]` is filled only from `HEX_DIGITS` (ASCII).
        writer.push_str(unsafe { std::str::from_utf8_unchecked(&buf[..n]) });
    }
}

/// Write each byte as `\xNN` (uppercase hex).
///
/// 32 raw bytes fill a 128-byte stack buffer and are written once.
#[inline]
fn write_bytes_slash_x(writer: &mut String, bytes: &[u8]) {
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
        writer.push_str(unsafe { std::str::from_utf8_unchecked(&buf[..n]) });
    }
}

/// Write `\xNN` for a single byte (uppercase hex).
#[inline]
fn write_byte_hex_escape(writer: &mut String, byte: u8) {
    writer.push_str("\\x");
    write_hex2(writer, byte)
}

/// Write `\uNNNN` for a codepoint that fits in 16 bits.
#[inline]
fn write_unicode4_escape(writer: &mut String, code: u32) {
    debug_assert!(code <= 0xFFFF);
    writer.push_str("\\u");
    write_hex2(writer, (code >> 8) as u8);
    write_hex2(writer, code as u8)
}

/// Use the string escape table, plus `\``.
///
/// `'` is left as itself. Only known escapes are emitted.
fn write_identifier_backtick_escape(writer: &mut String, ident: &str) {
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
fn write_byte_span(writer: &mut String, bytes: &[u8], from: usize, to: usize) {
    if from == to {
        return;
    }
    // SAFETY: see invariant above.
    let chunk = unsafe { bytes.get_unchecked(from..to) };
    writer.push_str(unsafe { std::str::from_utf8_unchecked(chunk) })
}

enum SpanEscape {
    Keep,
    Literal(&'static str),
    HexByte,
    Unicode4,
}

/// Copy each plain span with one `write_str`. Special bytes are ASCII.
#[cfg_attr(feature = "nightly", optimize(speed))]
fn write_span_escapes(writer: &mut String, string: &str, mut classify: impl FnMut(u8) -> SpanEscape) {
    let bytes = string.as_bytes();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        match classify(bytes[i]) {
            SpanEscape::Keep => i += 1,
            SpanEscape::Literal(repl) => {
                write_byte_span(writer, bytes, start, i);
                writer.push_str(repl);
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

fn write_backslash_hex(writer: &mut String, string: &str) {
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

fn write_backslash_mysql(writer: &mut String, string: &str) {
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

fn write_backslash_unicode(writer: &mut String, string: &str, escape_double_quote: bool) {
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

fn write_backslash_hex_zero(writer: &mut String, string: &str) {
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
