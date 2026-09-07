//! [Amazon Redshift](https://aws.amazon.com/redshift/).
//!
//! Postgres-derived, but **not** Postgres: default mode does not treat
//! backslash as an escape, and `E'…'` is not used.
//!
//! • Strings: single-quoted; only `'` is special (`''`). NUL is stripped.
//! • Identifiers: double-quote quoted; `"` doubled.
//! • Bytes: `TO_VARBYTE('<hex>', 'hex')`.
//! • Bits: no native bit literal; `CAST(<i64> AS BIGINT)`.
//! • Bool: `TRUE` / `FALSE`.
//! • Placeholders: `:name` (not native; rewritten client-side).
//!
//! Ref: <https://docs.aws.amazon.com/redshift/latest/dg/r_Constants.html>
//!      <https://docs.aws.amazon.com/redshift/latest/dg/r_VARBYTE_type.html>

use crate::internal::utils::replace_to;

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "Redshift";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Keyword;

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
    writer.write_str("TO_VARBYTE('")?;
    super::bytes_to_hex(writer, bytes)?;
    writer.write_str("', 'hex')")
}

pub(super) fn encode_bits<W: std::fmt::Write>(writer: &mut W, bits: &str) -> std::fmt::Result {
    write!(writer, "CAST({} AS BIGINT)", super::bits_as_i64(bits))
}
