//! A dialect for [SQLite](https://www.sqlite.org).
//!
//! • Strings: single-quoted; `''` for embedded single-quote.
//!            SQLite does NOT interpret backslash escapes in string literals.
//! • Identifiers: double-quote quoted (also supports back-tick and […]).
//! • Bytes:  X'DEADBEEF'  BLOB hex literal.
//! • Bits:   No native bit literal; use integer value.
//! • Bool:   1 / 0 (no native BOOLEAN type).
//!
//! Ref: <https://www.sqlite.org/lang_expr.html>

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "Sqlite";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Int;

pub(super) const IDENTIFIER_QUOTE: Option<IdentifierQuote> = Some(IdentifierQuote::DoubleQuote);

pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::Colon;

pub(super) fn escape_string(s: &str) -> String {
    // SQLite does not recognise backslash escape sequences.
    // NUL character (U+0000) terminates the string in the C library;
    // we remove it to be safe.
    super::escape_single_quote(s).replace('\0', "")
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

    format!("{}", i64::from_str_radix(bits, 2).unwrap_or(0))
}
