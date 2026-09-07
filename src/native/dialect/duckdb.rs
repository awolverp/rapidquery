//! [DuckDB](https://duckdb.org/).
//!
//! • Strings: single-quoted; `''` for an embedded quote. Backslash is **not**
//!   special in default mode. NUL is passed through (VARCHAR is UTF-8).
//! • Identifiers: double-quote quoted; `"` doubled.
//! • Bytes: `X'DEADBEEF'` hex literal.
//! • Bits: `B'10110101'` bit-string literal.
//! • Bool: `TRUE` / `FALSE`.
//! • Placeholders: `$name` (native-ish; DuckDB also accepts `$1` / `?`).
//!
//! Ref: <https://duckdb.org/docs/sql/data_types/literal_types>

use crate::internal::utils::replace_to;

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "DuckDB";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Keyword;

pub(super) const IDENTIFIER_QUOTE: IdentifierQuote = IdentifierQuote::DoubleQuote;

pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::DollarNamed;

pub(super) fn escape_string<W: std::fmt::Write>(writer: &mut W, string: &str) -> std::fmt::Result {
    super::escape_single_quote(writer, string)
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

pub(super) fn encode_bits<W: std::fmt::Write>(writer: &mut W, bits: &str) -> std::fmt::Result {
    super::debug_assert_bits(bits);
    write!(writer, "B'{bits}'")
}
