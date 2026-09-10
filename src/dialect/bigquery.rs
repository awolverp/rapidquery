//! [Google BigQuery](https://cloud.google.com/bigquery/) Standard SQL.
//!
//! • Strings: single-quoted; backslash-escape sequences (`\n`, `\xNN`, …).
//! • Identifiers: backtick-quoted. The body uses the same escaping as a
//!   string (BigQuery does not double backticks).
//! • Bytes: `B'\xNN…'` byte literal.
//! • Bits: no native bit-string literal; emit `CAST(0b<bits> AS INT64)`.
//! • Bool: `TRUE` / `FALSE`.
//! • Placeholders: `@name` (native).
//!
//! Ref: <https://cloud.google.com/bigquery/docs/reference/standard-sql/lexical>

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "BigQuery";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Keyword;

pub(super) const IDENTIFIER_QUOTE: IdentifierQuote = IdentifierQuote::Backtick;

pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::AtSign;

#[cfg_attr(feature = "nightly", optimize(speed))]
pub(super) fn escape_string<W: std::fmt::Write>(writer: &mut W, string: &str) -> std::fmt::Result {
    for ch in string.chars() {
        match ch {
            '\\' => writer.write_str("\\\\")?,
            '\'' => writer.write_str("\\'")?,
            '\n' => writer.write_str("\\n")?,
            '\r' => writer.write_str("\\r")?,
            '\t' => writer.write_str("\\t")?,
            '\0' => writer.write_str("\\x00")?,
            c if (c as u32) < 32 || c as u32 == 127 => {
                super::write_byte_hex_escape(writer, c as u8)?;
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
    escape_string(writer, ident)
}

pub(super) fn encode_bytes<W: std::fmt::Write>(writer: &mut W, bytes: &[u8]) -> std::fmt::Result {
    writer.write_str("B'")?;
    for &b in bytes {
        super::write_byte_hex_escape(writer, b)?;
    }
    writer.write_char('\'')
}

pub(super) fn encode_bits<W: std::fmt::Write>(writer: &mut W, bits: &str) -> std::fmt::Result {
    super::debug_assert_bits(bits);
    write!(writer, "CAST(0b{bits} AS INT64)")
}
