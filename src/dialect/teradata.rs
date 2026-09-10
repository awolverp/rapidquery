//! [Teradata](https://docs.teradata.com/).
//!
//! • Strings: single-quoted; only `'` is special (`''`). Escape sequences
//!   depend on the session character set, so we never emit them. NUL is
//!   stripped.
//! • Identifiers: single-quote quoted (`'name'`); `'` doubled.
//! • Bytes: `'<HEX>'XBV` (VARBYTE).
//! • Bits: `'10110101'B` bit literal.
//! • Bool: `TRUE` / `FALSE` (13.x+).
//! • Placeholders: `:name` (not native; rewritten client-side).
//!
//! Ref: <https://docs.teradata.com/r/Teradata-VantageCloud-Lake/SQL-Fundamentals/SQL-Data-Types-and-Literals>

use crate::internal::utils::replace_to;

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "Teradata";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Keyword;

pub(super) const IDENTIFIER_QUOTE: IdentifierQuote = IdentifierQuote::SingleQuote;

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
    replace_to(ident, '\'', "''", writer)
}

pub(super) fn encode_bytes<W: std::fmt::Write>(writer: &mut W, bytes: &[u8]) -> std::fmt::Result {
    writer.write_char('\'')?;
    super::bytes_to_hex(writer, bytes)?;
    writer.write_str("'XBV")
}

pub(super) fn encode_bits<W: std::fmt::Write>(writer: &mut W, bits: &str) -> std::fmt::Result {
    super::debug_assert_bits(bits);
    write!(writer, "'{bits}'B")
}
