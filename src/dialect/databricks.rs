//! [Databricks SQL](https://www.databricks.com/).
//!
//! • Strings: single-quoted; backslash-escape sequences (Java-style `\uNNNN`
//!   for other control chars).
//! • Identifiers: backtick-quoted; backtick doubled.
//! • Bytes: `X'DEADBEEF'` hex literal (BINARY).
//! • Bits: no native bit literal; `CAST(<i64>L AS BINARY)`.
//! • Bool: `TRUE` / `FALSE`.
//! • Placeholders: `:name` (client-side; not a native Databricks feature).
//!
//! Ref: <https://docs.databricks.com/en/sql/language-manual/sql-ref-literals.html>

use crate::internal::utils::replace_to;

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "Databricks";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Keyword;

pub(super) const IDENTIFIER_QUOTE: IdentifierQuote = IdentifierQuote::Backtick;

pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::Colon;

#[cfg_attr(feature = "nightly", optimize(speed))]
pub(super) fn escape_string<W: std::fmt::Write>(writer: &mut W, string: &str) -> std::fmt::Result {
    for ch in string.chars() {
        match ch {
            '\\' => writer.write_str("\\\\")?,
            '\'' => writer.write_str("\\'")?,
            '"' => writer.write_str("\\\"")?,
            '\n' => writer.write_str("\\n")?,
            '\r' => writer.write_str("\\r")?,
            '\t' => writer.write_str("\\t")?,
            '\0' => writer.write_str("\\0")?,
            c if (c as u32) < 32 || c as u32 == 127 => {
                super::write_unicode4_escape(writer, c as u32)?;
            }
            c => writer.write_char(c)?,
        }
    }

    Ok(())
}

pub(super) fn quote_string<W: std::fmt::Write>(writer: &mut W, string: &str) -> std::fmt::Result {
    super::wrap_single_quote(writer, string)
}

pub(super) fn escape_identifier<W: std::fmt::Write>(
    writer: &mut W,
    ident: &str,
) -> std::fmt::Result {
    replace_to(ident, '`', "``", writer)
}

pub(super) fn encode_bytes<W: std::fmt::Write>(writer: &mut W, bytes: &[u8]) -> std::fmt::Result {
    super::write_x_hex_literal(writer, bytes)
}

// Unsupported: Serializer::write_sql raises NotImplementedError via supports_bit.
pub(super) fn encode_bits<W: std::fmt::Write>(writer: &mut W, bits: &str) -> std::fmt::Result {
    write!(writer, "CAST({}L AS BINARY)", super::bits_as_i64(bits))
}
