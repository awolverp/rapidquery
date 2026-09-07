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

use crate::internal::utils::{replace_ascii_to, replace_to};

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "Redshift";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Keyword;

pub(super) const IDENTIFIER_QUOTE: Option<IdentifierQuote> = Some(IdentifierQuote::DoubleQuote);

/// This is not natively supported, and should be rendered on client-side.
pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::Colon;

pub(super) fn escape_string<T: std::fmt::Write>(writer: &mut T, string: &str) -> std::fmt::Result {
    // Redshift: backslash is NOT an escape character in standard mode.
    // Only `'` needs doubling.
    // NUL (U+0000) is not supported in VARCHAR literals.
    const REPLACEMENTS: [(u8, &str); 2] = [(b'\'', "''"), (b'\0', "")];
    replace_ascii_to(string, &REPLACEMENTS, writer)
}

pub(super) fn quote_string<T: std::fmt::Write>(writer: &mut T, string: &str) -> std::fmt::Result {
    super::wrap_single_quote(writer, string)
}

pub(super) fn escape_identifier<T: std::fmt::Write>(
    writer: &mut T,
    ident: &str,
) -> std::fmt::Result {
    replace_to(ident, '"', "\"\"", writer)
}

pub(super) fn encode_bytes<T: std::fmt::Write>(writer: &mut T, bytes: &[u8]) -> std::fmt::Result {
    writer.write_str("TO_VARBYTE('")?;
    super::bytes_to_hex(writer, bytes)?;
    writer.write_str("', 'hex')")
}

pub(super) fn encode_bits<T: std::fmt::Write>(writer: &mut T, bits: &str) -> std::fmt::Result {
    debug_assert!(
        bits.chars().all(|c| c == '0' || c == '1'),
        "encode_bits: invalid input: {bits}"
    );

    write!(
        writer,
        "CAST({} AS BIGINT)",
        i64::from_str_radix(bits, 2).unwrap_or(0)
    )
}
