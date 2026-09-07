//! A dialect for [DuckDB](https://duckdb.org/).
//!
//! • Strings: single-quoted; `''` for embedded single-quote.
//! • Identifiers: double-quote quoted.
//! • Bytes:  '\xNN…'::BLOB  or  X'DEADBEEF' hex literal.
//! • Bits:   0b<bits>::INTEGER  or  BIT literal via BITSTRING.
//! • Bool:   TRUE / FALSE keywords.
//!
//! Ref: <https://duckdb.org/docs/sql/data_types/literal_types>

use crate::internal::utils::replace_to;

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "DuckDb";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Keyword;

pub(super) const IDENTIFIER_QUOTE: Option<IdentifierQuote> = Some(IdentifierQuote::DoubleQuote);

pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::DollarNamed;

pub(super) fn escape_string<T: std::fmt::Write>(writer: &mut T, string: &str) -> std::fmt::Result {
    // DuckDB uses '' for quote escaping (ANSI-compatible).
    // Backslash is NOT a special escape character in default mode.
    super::escape_single_quote(writer, string)
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

    // DuckDB supports bitstring literals with B prefix
    write!(writer, "B'{}'", bits)
}
