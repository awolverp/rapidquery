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

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "Oracle";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Int;

pub(super) const IDENTIFIER_QUOTE: Option<IdentifierQuote> = Some(IdentifierQuote::DoubleQuote);

pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::Colon;

pub(super) fn escape_string(s: &str) -> String {
    // Oracle: only single-quote doubling; no backslash escaping.
    // NUL bytes are not supported in VARCHAR2 literals.
    super::escape_single_quote(s).replace('\0', "")
}

pub(super) fn quote_string(s: &str) -> String {
    super::wrap_single_quote(s)
}

pub(super) fn encode_bytes(bytes: &[u8]) -> String {
    format!("HEXTORAW('{}')", super::bytes_to_hex(bytes))
}

pub(super) fn encode_bits(bits: &str) -> String {
    debug_assert!(
        bits.chars().all(|c| c == '0' || c == '1'),
        "encode_bits: invalid input: {bits}"
    );

    format!("{}", i64::from_str_radix(bits, 2).unwrap_or(0))
}
