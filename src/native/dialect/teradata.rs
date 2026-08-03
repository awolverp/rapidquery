//! A Dialect for [Teradata](https://docs.teradata.com/).
//!
//! • Strings: single-quoted; `''` for embedded single-quote.
//!            Teradata supports escape sequences only in named character sets;
//!            safest approach is quote-doubling only.
//! • Identifiers: double-quote quoted.
//! • Bytes:  '…'XB  byte literal (hex digits in single quotes with XB suffix).
//!           OR  '…'XBV for VARBYTE.
//! • Bits:   '10110101'B  bit literal.
//! • Bool:   TRUE / FALSE (Teradata 13.x+ with BYTEINT or BOOLEAN).
//!
//! Ref: <https://docs.teradata.com/r/Teradata-VantageCloud-Lake/SQL-Fundamentals/SQL-Data-Types-and-Literals>

use crate::internal::utils::{replace_many_ascii_to, str_replace_to};

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "Teradata";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Keyword;

pub(super) const IDENTIFIER_QUOTE: Option<IdentifierQuote> = Some(IdentifierQuote::SingleQuote);

/// This is not natively supported, and should be rendered on client-side.
pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::Colon;

pub(super) fn escape_string<T: std::fmt::Write>(writer: &mut T, string: &str) -> std::fmt::Result {
    // Teradata: only single-quote doubling; no backslash interpretation.
    // NUL bytes are not supported.
    const REPLACEMENTS: [(u8, &str); 2] = [(b'\'', "''"), (b'\0', "")];
    replace_many_ascii_to(string, &REPLACEMENTS, writer)
}

pub(super) fn quote_string<T: std::fmt::Write>(writer: &mut T, string: &str) -> std::fmt::Result {
    super::wrap_single_quote(writer, string)
}

pub(super) fn escape_identifier<T: std::fmt::Write>(
    writer: &mut T,
    ident: &str,
) -> std::fmt::Result {
    str_replace_to(ident, '\'', "''", writer)
}

pub(super) fn encode_bytes<T: std::fmt::Write>(writer: &mut T, bytes: &[u8]) -> std::fmt::Result {
    // Teradata byte literal: '<HEX>'XBV
    writer.write_str("'")?;
    super::bytes_to_hex(writer, bytes)?;
    writer.write_str("'XBV")
}

pub(super) fn encode_bits<T: std::fmt::Write>(writer: &mut T, bits: &str) -> std::fmt::Result {
    debug_assert!(
        bits.chars().all(|c| c == '0' || c == '1'),
        "encode_bits: invalid input: {bits}"
    );

    write!(writer, "'{}'B", bits)
}
