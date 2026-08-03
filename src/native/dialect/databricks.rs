//! A dialect for [Databricks SQL](https://www.databricks.com/).
//!
//! See <https://docs.databricks.com/en/sql/language-manual/index.html>.
//!
//! • Strings: single-quoted; backslash-escape sequences.
//! • Identifiers: back-tick quoted.
//! • Bytes:  X'DEADBEEF' hex literal (BINARY type).
//! • Bits:   No native bit literal; we use CAST(… AS BINARY) from INT.
//! • Bool:   TRUE / FALSE keywords.
//!
//! Ref: <https://docs.databricks.com/en/sql/language-manual/sql-ref-literals.html>

use crate::internal::utils::str_replace_to;

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "Databricks";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Keyword;

pub(super) const IDENTIFIER_QUOTE: Option<IdentifierQuote> = Some(IdentifierQuote::Backtick);

pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::Colon;

pub(super) fn escape_string<T: std::fmt::Write>(writer: &mut T, string: &str) -> std::fmt::Result {
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
                writer.write_str(&format!("\\u{:04X}", c as u32))?;
            }
            c => writer.write_char(c)?,
        }
    }

    Ok(())
}

pub(super) fn quote_string<T: std::fmt::Write>(writer: &mut T, string: &str) -> std::fmt::Result {
    super::wrap_single_quote(writer, string)
}

pub(super) fn escape_identifier<T: std::fmt::Write>(
    writer: &mut T,
    ident: &str,
) -> std::fmt::Result {
    str_replace_to(ident, '`', "``", writer)
}

pub(super) fn encode_bytes<T: std::fmt::Write>(writer: &mut T, bytes: &[u8]) -> std::fmt::Result {
    writer.write_str("X'")?;
    super::bytes_to_hex(writer, bytes)?;
    writer.write_char('\'')
}

pub(super) fn encode_bits<T: std::fmt::Write>(writer: &mut T, bits: &str) -> std::fmt::Result {
    debug_assert!(
        bits.chars().all(|c| c == '0' || c == '1'),
        "encode_bits: invalid input: {bits}"
    );

    // Databricks supports CAST with integer literals
    write!(
        writer,
        "CAST({}L AS BINARY)",
        i64::from_str_radix(bits, 2).unwrap_or(0)
    )
}
