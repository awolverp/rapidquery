//! A dialect for [Microsoft SQL Server](https://www.microsoft.com/en-us/sql-server/).
//!
//! • Strings: single-quoted; `''` for embedded single-quote.
//!            Unicode strings prefixed with N'…'.
//! • Identifiers: square-bracket quoted: [name].
//! • Bytes:  0xDEADBEEF  (no quotes).
//! • Bits:   Cast integer to BIT; MSSQL BIT is boolean-like (0/1).
//! • Bool:   1 / 0 (BIT type; no TRUE/FALSE keyword).
//!
//! Ref: <https://learn.microsoft.com/en-us/sql/t-sql/data-types/constants-transact-sql>

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "MsSql";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Int;

pub(super) const IDENTIFIER_QUOTE: Option<IdentifierQuote> = Some(IdentifierQuote::SquareBracket);

pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::AtSign;

pub(super) fn escape_string(s: &str) -> String {
    // T-SQL: only single-quote needs doubling.
    // NUL character (U+0000) is not allowed in T-SQL string literals.
    // We remove it to stay safe.
    super::escape_single_quote(s).replace('\0', "")
}

pub(super) fn quote_string(s: &str) -> String {
    // Prefix with N for Unicode safety (NVARCHAR)
    let mut out = String::with_capacity(s.len() + 3);
    out.push_str("N'");
    out.push_str(s);
    out.push('\'');
    out
}

pub(super) fn encode_bytes(bytes: &[u8]) -> String {
    // T-SQL binary literal: 0xDEADBEEF
    format!("0x{}", super::bytes_to_hex(bytes))
}

pub(super) fn encode_bits(bits: &str) -> String {
    debug_assert!(
        bits.chars().all(|c| c == '0' || c == '1'),
        "encode_bits: invalid input: {bits}"
    );

    // MSSQL BIT is a 1-bit integer; for multi-bit we cast to integer
    format!(
        "CAST({} AS BIGINT)",
        i64::from_str_radix(bits, 2).unwrap_or(0)
    )
}
