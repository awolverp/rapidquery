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

use crate::internal::utils::{replace_many_ascii_to, str_replace_to};

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "Sqlite";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Int;

pub(super) const IDENTIFIER_QUOTE: Option<IdentifierQuote> = Some(IdentifierQuote::DoubleQuote);

pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::Colon;

pub(super) fn escape_string<T: std::fmt::Write>(writer: &mut T, string: &str) -> std::fmt::Result {
    // SQLite does not recognise backslash escape sequences.
    // NUL character (U+0000) terminates the string in the C library;
    // we remove it to be safe.
    const REPLACEMENTS: [(u8, &str); 2] = [(b'\'', "''"), (b'\0', "")];
    replace_many_ascii_to(string, &REPLACEMENTS, writer)
}

pub(super) fn quote_string<T: std::fmt::Write>(writer: &mut T, string: &str) -> std::fmt::Result {
    super::wrap_single_quote(writer, string)
}

pub(super) fn escape_identifier<T: std::fmt::Write>(
    writer: &mut T,
    ident: &str,
) -> std::fmt::Result {
    str_replace_to(ident, '"', "\"\"", writer)
}

pub(super) fn encode_bytes<T: std::fmt::Write>(writer: &mut T, bytes: &[u8]) -> std::fmt::Result {
    // write!(writer, "X'{}'", super::bytes_to_hex(writer, bytes))
    writer.write_str("X'")?;
    super::bytes_to_hex(writer, bytes)?;
    writer.write_char('\'')
}

pub(super) fn encode_bits<T: std::fmt::Write>(writer: &mut T, bits: &str) -> std::fmt::Result {
    debug_assert!(
        bits.chars().all(|c| c == '0' || c == '1'),
        "encode_bits: invalid input: {bits}"
    );

    write!(writer, "{}", i64::from_str_radix(bits, 2).unwrap_or(0))
}
