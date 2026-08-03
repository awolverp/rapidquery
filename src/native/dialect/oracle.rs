//! A dialect for [Oracle Databases](https://docs.oracle.com/en/database/oracle/oracle-database/21/sqlrf/index.html).
//!
//! Ref: <https://docs.oracle.com/en/database/oracle/oracle-database/21/sqlrf/Literals.html>
//!
//! • Strings: single-quoted; `''` for embedded single-quote.
//!            Alternatively q'[…]' alternative quoting.
//! • Identifiers: double-quote quoted.
//! • Bytes:  HEXTORAW('DEADBEEF').
//! • Bits:   No native bit literal; use integer value.
//! • Bool:   Oracle < 23c has no BOOLEAN in SQL; 23c onwards has TRUE/FALSE.
//!           We default to 1/0 for broadest compatibility.

use crate::internal::utils::{replace_many_ascii_to, str_replace_to};

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "Oracle";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Int;

pub(super) const IDENTIFIER_QUOTE: Option<IdentifierQuote> = Some(IdentifierQuote::DoubleQuote);

pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::Colon;

pub(super) fn escape_string<T: std::fmt::Write>(writer: &mut T, string: &str) -> std::fmt::Result {
    // Oracle: only single-quote doubling; no backslash escaping.
    // NUL bytes are not supported in VARCHAR2 literals.
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
    str_replace_to(ident, '"', "\"\"", writer)
}

pub(super) fn encode_bytes<T: std::fmt::Write>(writer: &mut T, bytes: &[u8]) -> std::fmt::Result {
    writer.write_str("HEXTORAW('")?;
    super::bytes_to_hex(writer, bytes)?;
    writer.write_str("')")
}

pub(super) fn encode_bits<T: std::fmt::Write>(writer: &mut T, bits: &str) -> std::fmt::Result {
    debug_assert!(
        bits.chars().all(|c| c == '0' || c == '1'),
        "encode_bits: invalid input: {bits}"
    );

    write!(writer, "{}", i64::from_str_radix(bits, 2).unwrap_or(0))
}
