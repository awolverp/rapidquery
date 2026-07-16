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

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "Teradata";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Keyword;

pub(super) const IDENTIFIER_QUOTE: Option<IdentifierQuote> = Some(IdentifierQuote::SingleQuote);

/// This is not natively supported, and should be rendered on client-side.
pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::Colon;

pub(super) fn escape_string(s: &str) -> String {
    // Teradata: only single-quote doubling; no backslash interpretation.
    // NUL bytes are not supported.
    super::escape_single_quote(s).replace('\0', "")
}

pub(super) fn quote_string(s: &str) -> String {
    super::wrap_single_quote(s)
}

pub(super) fn encode_bytes(bytes: &[u8]) -> String {
    // Teradata byte literal: '<HEX>'XBV
    format!("'{}'XBV", super::bytes_to_hex(bytes))
}

pub(super) fn encode_bits(bits: &str) -> String {
    debug_assert!(
        bits.chars().all(|c| c == '0' || c == '1'),
        "encode_bits: invalid input: {bits}"
    );

    // Teradata bit literal: '<bits>'B
    format!("'{}'B", bits)
}
