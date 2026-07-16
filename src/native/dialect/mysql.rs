//! A dialect for [MySQL](https://www.mysql.com/).
//!
//! • Strings: single-quoted; backslash-escape sequences.
//! • Identifiers: back-tick quoted.
//! • Bytes:  X'DEADBEEF'  or  0xDEADBEEF.
//! • Bits:   b'10110101'  bit literal.
//! • Bool:   1 / 0 (TINYINT; no native BOOLEAN literal).
//!
//! Ref: <https://dev.mysql.com/doc/refman/8.0/en/string-literals.html>
//!      <https://dev.mysql.com/doc/refman/8.0/en/bit-value-literals.html>

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "MySql";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Int;

pub(super) const IDENTIFIER_QUOTE: Option<IdentifierQuote> = Some(IdentifierQuote::Backtick);

/// This is not natively supported, and should be rendered on client-side, and many clients do.
pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::Colon;

pub(super) fn escape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + s.len() / 4);
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '"' => out.push_str("\\\""),
            '\0' => out.push_str("\\0"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x1a' => out.push_str("\\Z"), // Ctrl+Z / SUB (Windows EOF)
            c => out.push(c),
        }
    }
    out
}

pub(super) fn quote_string(s: &str) -> String {
    super::wrap_single_quote(s)
}

pub(super) fn encode_bytes(bytes: &[u8]) -> String {
    format!("X'{}'", super::bytes_to_hex(bytes))
}

pub(super) fn encode_bits(bits: &str) -> String {
    debug_assert!(
        bits.chars().all(|c| c == '0' || c == '1'),
        "encode_bits: invalid input: {bits}"
    );

    format!("b'{}'", bits)
}
