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

use crate::internal::utils::{replace_many_ascii_to, str_replace_to};

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "MsSql";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Int;

pub(super) const IDENTIFIER_QUOTE: Option<IdentifierQuote> = Some(IdentifierQuote::SquareBracket);

pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::AtSign;

pub(super) fn escape_string<T: std::fmt::Write>(writer: &mut T, string: &str) -> std::fmt::Result {
    // T-SQL: only single-quote needs doubling.
    // NUL character (U+0000) is not allowed in T-SQL string literals.
    // We remove it to stay safe.
    const REPLACEMENTS: [(u8, &str); 2] = [(b'\'', "''"), (b'\0', "")];
    replace_many_ascii_to(string, &REPLACEMENTS, writer)
}

pub(super) fn quote_string<T: std::fmt::Write>(writer: &mut T, string: &str) -> std::fmt::Result {
    // Prefix with N for Unicode safety (NVARCHAR)
    write!(writer, "N'{string}'")
}

pub(super) fn escape_identifier<T: std::fmt::Write>(
    writer: &mut T,
    ident: &str,
) -> std::fmt::Result {
    str_replace_to(ident, ']', "]]", writer)
}

pub(super) fn encode_bytes<T: std::fmt::Write>(writer: &mut T, bytes: &[u8]) -> std::fmt::Result {
    // T-SQL binary literal: 0xDEADBEEF
    writer.write_str("0x")?;
    super::bytes_to_hex(writer, bytes)
}

pub(super) fn encode_bits<T: std::fmt::Write>(writer: &mut T, bits: &str) -> std::fmt::Result {
    debug_assert!(
        bits.chars().all(|c| c == '0' || c == '1'),
        "encode_bits: invalid input: {bits}"
    );

    // MSSQL BIT is a 1-bit integer; for multi-bit we cast to integer
    write!(
        writer,
        "CAST({} AS BIGINT)",
        i64::from_str_radix(bits, 2).unwrap_or(0)
    )
}
