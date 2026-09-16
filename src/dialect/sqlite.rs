//! [SQLite](https://www.sqlite.org/).
//!
//! • Strings: single-quoted; only `'` is special (`''`). Backslash is
//!   **not** an escape. NUL is stripped because the C API treats U+0000 as
//!   a terminator.
//! • Identifiers: double-quote quoted (SQLite also accepts backticks and
//!   `[…]`; we standardize on `"`).
//! • Bytes: `X'DEADBEEF'` BLOB hex literal.
//! • Bits: no native bit literal; integer value.
//! • Bool: `1` / `0` (no native BOOLEAN type).
//! • Placeholders: `:name` (native; SQLite also accepts `?` / `$n` / `@name`).
//!
//! Ref: <https://www.sqlite.org/lang_expr.html>

use crate::internal::utils::replace_to;

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "SQLite";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Int;

pub(super) const IDENTIFIER_QUOTE: IdentifierQuote = IdentifierQuote::DoubleQuote;

pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::Colon;

pub(super) fn escape_string<W: std::fmt::Write>(writer: &mut W, string: &str) -> std::fmt::Result {
    super::escape_quote_and_strip_nul(writer, string)
}

pub(super) fn quote_string<W: std::fmt::Write>(writer: &mut W, string: &str) -> std::fmt::Result {
    super::wrap_single_quote(writer, string)
}

pub(super) fn escape_identifier<W: std::fmt::Write>(
    writer: &mut W,
    ident: &str,
) -> std::fmt::Result {
    replace_to(ident, '"', "\"\"", writer)
}

pub(super) fn encode_bytes<W: std::fmt::Write>(writer: &mut W, bytes: &[u8]) -> std::fmt::Result {
    super::write_x_hex_literal(writer, bytes)
}

// Unsupported: Serializer::write_sql raises NotImplementedError via supports_bit.
pub(super) fn encode_bits<W: std::fmt::Write>(writer: &mut W, bits: &str) -> std::fmt::Result {
    write!(writer, "{}", super::bits_as_i64(bits))
}
