//! A dialect for [RedShift](https://aws.amazon.com/redshift/).
//!
//! • Strings: single-quoted; `''` for embedded single-quote.
//!            Redshift is PostgreSQL-derived but does NOT support E'…' by default.
//!            Backslash in default mode is literal (not escape); so we double quotes only.
//! • Identifiers: double-quote quoted.
//! • Bytes:  VARBYTE from hex via FROM_HEX or TO_VARBYTE.
//! • Bits:   No native bit literal; cast integer.
//! • Bool:   TRUE / FALSE keywords (BOOLEAN type).
//!
//! Ref: <https://docs.aws.amazon.com/redshift/latest/dg/r_Constants.html>
//!      <https://docs.aws.amazon.com/redshift/latest/dg/r_VARBYTE_type.html>

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "Redshift";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Keyword;

pub(super) const IDENTIFIER_QUOTE: Option<IdentifierQuote> = Some(IdentifierQuote::DoubleQuote);

/// This is not natively supported, and should be rendered on client-side.
pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::Colon;

pub(super) fn escape_string(s: &str) -> String {
    // Redshift: backslash is NOT an escape character in standard mode.
    // Only `'` needs doubling.
    // NUL (U+0000) is not supported in VARCHAR literals.
    super::escape_single_quote(s).replace('\0', "")
}

pub(super) fn quote_string(s: &str) -> String {
    super::wrap_single_quote(s)
}

pub(super) fn encode_bytes(bytes: &[u8]) -> String {
    format!("TO_VARBYTE('{}', 'hex')", super::bytes_to_hex(bytes))
}

pub(super) fn encode_bits(bits: &str) -> String {
    debug_assert!(
        bits.chars().all(|c| c == '0' || c == '1'),
        "encode_bits: invalid input: {bits}"
    );

    format!(
        "CAST({} AS BIGINT)",
        i64::from_str_radix(bits, 2).unwrap_or(0)
    )
}
