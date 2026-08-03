//! A dialect for [Google BigQuery](https://cloud.google.com/bigquery/).
//!
//! • Strings: single-quoted; escape with backslash sequences.
//! • Identifiers: back-tick quoted.
//! • Bytes:  B'\xNN…'  (byte literal with \x escapes).
//! • Bits:   No native bit-string literal; cast from INT64.
//!           We emit CAST(0b<bits> AS INT64) for best portability.
//! • Bool:   TRUE / FALSE keywords.
//!
//! Ref: <https://cloud.google.com/bigquery/docs/reference/standard-sql/lexical>

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "BigQuery";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Keyword;

pub(super) const IDENTIFIER_QUOTE: Option<IdentifierQuote> = Some(IdentifierQuote::Backtick);

pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::AtSign;

pub(super) fn escape_string<T: std::fmt::Write>(writer: &mut T, string: &str) -> std::fmt::Result {
    for ch in string.chars() {
        match ch {
            '\\' => writer.write_str("\\\\")?,
            '\'' => writer.write_str("\\'")?,
            '\n' => writer.write_str("\\n")?,
            '\r' => writer.write_str("\\r")?,
            '\t' => writer.write_str("\\t")?,
            '\0' => writer.write_str("\\x00")?,
            // ASCII control characters
            c if (c as u32) < 32 || c as u32 == 127 => {
                writer.write_str(&format!("\\x{:02X}", c as u32))?;
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
    escape_string(writer, ident)
}

pub(super) fn encode_bytes<T: std::fmt::Write>(writer: &mut T, bytes: &[u8]) -> std::fmt::Result {
    // BigQuery byte literal: B'\xNN\xNN…'
    writer.write_str("B'")?;
    for b in bytes {
        writer.write_str(&format!("\\x{:02X}", b))?;
    }
    writer.write_char('\'')
}

pub(super) fn encode_bits<T: std::fmt::Write>(writer: &mut T, bits: &str) -> std::fmt::Result {
    debug_assert!(
        bits.chars().all(|c| c == '0' || c == '1'),
        "encode_bits: invalid input: {bits}"
    );

    // BigQuery has no native bit literal; represent as INT64 binary literal.
    write!(writer, "CAST(0b{} AS INT64)", bits)
}
