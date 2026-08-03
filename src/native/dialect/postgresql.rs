//! A dialect for [PostgreSQL](https://www.postgresql.org/).
//!
//! • Strings: single-quoted with `''` doubling.
//!            For control chars we use E'…' escape strings with \xNN.
//! • Identifiers: double-quote quoted.
//! • Bytes:  '\xDEADBEEF'::bytea  (hex format, PostgreSQL 9.0+).
//! • Bits:   B'10110101'  bit string literal.
//! • Bool:   TRUE / FALSE keywords.
//!
//! Ref: <https://www.postgresql.org/docs/current/sql-syntax-lexical.html>
//!      <https://www.postgresql.org/docs/current/datatype-binary.html>
//!      <https://www.postgresql.org/docs/current/datatype-bit.html>

use crate::internal::utils::str_replace_to;

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "PostgreSql";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Keyword;

pub(super) const IDENTIFIER_QUOTE: Option<IdentifierQuote> = Some(IdentifierQuote::DoubleQuote);

/// This is not natively supported, and should be rendered on client-side, and many clients do.
pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::Colon;

pub(super) fn escape_string<T: std::fmt::Write>(writer: &mut T, string: &str) -> std::fmt::Result {
    // Use PostgreSQL escape-string syntax (E'…') when control chars present.
    // We always emit E'…' for safety; non-special chars pass through.
    for ch in string.chars() {
        match ch {
            '\\' => writer.write_str("\\\\")?,
            '\'' => writer.write_str("\\'")?,
            '\n' => writer.write_str("\\n")?,
            '\r' => writer.write_str("\\r")?,
            '\t' => writer.write_str("\\t")?,
            '\0' => writer.write_str("\\x00")?,
            c if (c as u32) < 32 || c as u32 == 127 => {
                writer.write_str(&format!("\\x{:02X}", c as u32))?;
            }
            c => writer.write_char(c)?,
        }
    }
    Ok(())
}

pub(super) fn quote_string<T: std::fmt::Write>(writer: &mut T, string: &str) -> std::fmt::Result {
    // Use E'…' escape string syntax so our \-sequences are interpreted.
    write!(writer, "E'{string}'")
}

pub(super) fn escape_identifier<T: std::fmt::Write>(
    writer: &mut T,
    ident: &str,
) -> std::fmt::Result {
    str_replace_to(ident, '"', "\"\"", writer)
}

pub(super) fn encode_bytes<T: std::fmt::Write>(writer: &mut T, bytes: &[u8]) -> std::fmt::Result {
    // PostgreSQL hex-format bytea literal
    writer.write_str("'\\x")?;
    super::bytes_to_hex(writer, bytes)?;
    writer.write_str("'::bytea")
}

pub(super) fn encode_bits<T: std::fmt::Write>(writer: &mut T, bits: &str) -> std::fmt::Result {
    debug_assert!(
        bits.chars().all(|c| c == '0' || c == '1'),
        "encode_bits: invalid input: {bits}"
    );

    write!(writer, "B'{}'", bits)
}
