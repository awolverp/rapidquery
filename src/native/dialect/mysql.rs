//! [MySQL](https://www.mysql.com/).
//!
//! • Strings: single-quoted; backslash-escape sequences. A raw tab is left
//!   as-is (not a syntax hazard). `\x1A` (Ctrl+Z / Windows EOF) becomes `\Z`.
//! • Identifiers: backtick-quoted; backtick doubled.
//! • Bytes: `X'DEADBEEF'`.
//! • Bits: `b'10110101'` bit literal.
//! • Bool: `1` / `0` (`TINYINT`; no native BOOLEAN literal).
//! • Placeholders: `:name` (not native; MySQL uses `?`. Common clients rewrite).
//!
//! Ref: <https://dev.mysql.com/doc/refman/8.0/en/string-literals.html>
//!      <https://dev.mysql.com/doc/refman/8.0/en/bit-value-literals.html>

use crate::internal::utils::replace_to;

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "MySql";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Int;

pub(super) const IDENTIFIER_QUOTE: IdentifierQuote = IdentifierQuote::Backtick;

pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::Colon;

pub(super) fn escape_string<W: std::fmt::Write>(writer: &mut W, string: &str) -> std::fmt::Result {
    for ch in string.chars() {
        match ch {
            '\\' => writer.write_str("\\\\")?,
            '\'' => writer.write_str("\\'")?,
            '"' => writer.write_str("\\\"")?,
            '\0' => writer.write_str("\\0")?,
            '\n' => writer.write_str("\\n")?,
            '\r' => writer.write_str("\\r")?,
            '\x1a' => writer.write_str("\\Z")?,
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

pub(super) fn encode_bits<W: std::fmt::Write>(writer: &mut W, bits: &str) -> std::fmt::Result {
    super::debug_assert_bits(bits);
    write!(writer, "b'{bits}'")
}
