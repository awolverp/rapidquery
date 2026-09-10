//! [Oracle Database](https://docs.oracle.com/en/database/oracle/oracle-database/).
//!
//! • Strings: single-quoted; only `'` is special (`''`). NUL is stripped
//!   (`VARCHAR2` cannot hold it).
//! • Identifiers: double-quote quoted; `"` doubled.
//! • Bytes: `HEXTORAW('DEADBEEF')`.
//! • Bits: no native bit literal; integer value.
//! • Bool: `1` / `0`. SQL BOOLEAN exists only from 23c; this keeps pre-23c
//!   compatibility.
//! • Placeholders: `:name` (native).
//!
//! Ref: <https://docs.oracle.com/en/database/oracle/oracle-database/21/sqlrf/Literals.html>

use crate::internal::utils::replace_to;

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "Oracle";

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
    writer.write_str("HEXTORAW('")?;
    super::bytes_to_hex(writer, bytes)?;
    writer.write_str("')")
}

pub(super) fn encode_bits<W: std::fmt::Write>(writer: &mut W, bits: &str) -> std::fmt::Result {
    write!(writer, "{}", super::bits_as_i64(bits))
}
