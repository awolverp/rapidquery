//! [Snowflake](https://www.snowflake.com/).
//!
//! • Strings: single-quoted; backslash-escape sequences (`\xNN` for other
//!   control chars).
//! • Identifiers: double-quote quoted; `"` doubled.
//! • Bytes: `X'DEADBEEF'` hex literal (BINARY).
//! • Bits: no native bit literal; `TO_BINARY(LPAD(TO_HEX(<i64>), 16, '0'), 'HEX')`.
//! • Bool: `TRUE` / `FALSE`.
//! • Placeholders: `:name` (native-ish; Snowflake also binds by position).
//!
//! Ref: <https://docs.snowflake.com/en/sql-reference/data-types-text>
//!      <https://docs.snowflake.com/en/sql-reference/literals>

use crate::internal::utils::replace_to;

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "Snowflake";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Keyword;

pub(super) const IDENTIFIER_QUOTE: IdentifierQuote = IdentifierQuote::DoubleQuote;

pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::Colon;

#[cfg_attr(feature = "nightly", optimize(speed))]
pub(super) fn escape_string<W: std::fmt::Write>(writer: &mut W, string: &str) -> std::fmt::Result {
    for ch in string.chars() {
        match ch {
            '\\' => writer.write_str("\\\\")?,
            '\'' => writer.write_str("\\'")?,
            '\n' => writer.write_str("\\n")?,
            '\r' => writer.write_str("\\r")?,
            '\t' => writer.write_str("\\t")?,
            '\0' => writer.write_str("\\0")?,
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
    replace_to(ident, '"', "\"\"", writer)
}

pub(super) fn encode_bytes<W: std::fmt::Write>(writer: &mut W, bytes: &[u8]) -> std::fmt::Result {
    super::write_x_hex_literal(writer, bytes)
}

// Unsupported: Serializer::write_sql raises NotImplementedError via supports_bit.
pub(super) fn encode_bits<W: std::fmt::Write>(writer: &mut W, bits: &str) -> std::fmt::Result {
    write!(
        writer,
        "TO_BINARY(LPAD(TO_HEX({}), 16, '0'), 'HEX')",
        super::bits_as_i64(bits)
    )
}
