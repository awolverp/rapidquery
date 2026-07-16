//! A dialect for [PostgreSQL](https://www.postgresql.org/).
//!
//! • Strings: single-quoted with `''` doubling.
//!            For control chars we use E'…' escape strings with \xNN.
//! • Identifiers: double-quote quoted.
//! • Bytes:  '\xDEADBEEF'::bytea  (hex format, PostgreSQL 9.0+).
//! • Bits:   B'10110101'  bit string literal.
//! • Bool:   TRUE / FALSE keywords.
//!
//! Ref: <https://www.postgresql.org/docs/current/sql-syntax-lexical.html>
//!      <https://www.postgresql.org/docs/current/datatype-binary.html>
//!      <https://www.postgresql.org/docs/current/datatype-bit.html>

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "PostgreSql";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Keyword;

pub(super) const IDENTIFIER_QUOTE: Option<IdentifierQuote> = Some(IdentifierQuote::DoubleQuote);

/// This is not natively supported, and should be rendered on client-side, and many clients do.
pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::Colon;

pub(super) fn escape_string(s: &str) -> String {
    // Use PostgreSQL escape-string syntax (E'…') when control chars present.
    // We always emit E'…' for safety; non-special chars pass through.
    let mut out = String::with_capacity(s.len() + s.len() / 4);

    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\0' => out.push_str("\\x00"),
            c if (c as u32) < 32 || c as u32 == 127 => {
                out.push_str(&format!("\\x{:02X}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

pub(super) fn quote_string(s: &str) -> String {
    // Use E'…' escape string syntax so our \-sequences are interpreted.
    let mut out = String::with_capacity(s.len() + 3);
    out.push_str("E'");
    out.push_str(s);
    out.push('\'');
    out
}

pub(super) fn encode_bytes(bytes: &[u8]) -> String {
    // PostgreSQL hex-format bytea literal
    format!("'\\x{}'::bytea", super::bytes_to_hex(bytes))
}

pub(super) fn encode_bits(bits: &str) -> String {
    debug_assert!(
        bits.chars().all(|c| c == '0' || c == '1'),
        "encode_bits: invalid input: {bits}"
    );

    format!("B'{}'", bits)
}
