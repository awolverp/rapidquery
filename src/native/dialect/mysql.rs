//! A dialect for [MySQL](https://www.mysql.com/).
//!
//! • Strings: single-quoted; backslash-escape sequences.
//! • Identifiers: back-tick quoted.
//! • Bytes:  X'DEADBEEF'  or  0xDEADBEEF.
//! • Bits:   b'10110101'  bit literal.
//! • Bool:   1 / 0 (TINYINT; no native BOOLEAN literal).
//!
//! Ref: <https://dev.mysql.com/doc/refman/8.0/en/string-literals.html>
//!      <https://dev.mysql.com/doc/refman/8.0/en/bit-value-literals.html>

use crate::internal::utils::replace_to;

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "MySql";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Int;

pub(super) const IDENTIFIER_QUOTE: Option<IdentifierQuote> = Some(IdentifierQuote::Backtick);

/// This is not natively supported, and should be rendered on client-side, and many clients do.
pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::Colon;

pub(super) fn escape_string<T: std::fmt::Write>(writer: &mut T, string: &str) -> std::fmt::Result {
    for ch in string.chars() {
        match ch {
            '\\' => writer.write_str("\\\\")?,
            '\'' => writer.write_str("\\'")?,
            '"' => writer.write_str("\\\"")?,
            '\0' => writer.write_str("\\0")?,
            '\n' => writer.write_str("\\n")?,
            '\r' => writer.write_str("\\r")?,
            '\x1a' => writer.write_str("\\Z")?, // Ctrl+Z / SUB (Windows EOF)
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
    replace_to(ident, '`', "``", writer)
}

pub(super) fn encode_bytes<T: std::fmt::Write>(writer: &mut T, bytes: &[u8]) -> std::fmt::Result {
    // write!(writer, "X'{}'", super::bytes_to_hex(writer, bytes))
    writer.write_str("X'")?;
    super::bytes_to_hex(writer, bytes)?;
    writer.write_char('\'')
}

pub(super) fn encode_bits<T: std::fmt::Write>(writer: &mut T, bits: &str) -> std::fmt::Result {
    debug_assert!(
        bits.chars().all(|c| c == '0' || c == '1'),
        "encode_bits: invalid input: {bits}"
    );

    write!(writer, "b'{}'", bits)
}
