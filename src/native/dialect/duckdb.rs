//! A dialect for [DuckDB](https://duckdb.org/).
//!
//! • Strings: single-quoted; `''` for embedded single-quote.
//! • Identifiers: double-quote quoted.
//! • Bytes:  '\xNN…'::BLOB  or  X'DEADBEEF' hex literal.
//! • Bits:   0b<bits>::INTEGER  or  BIT literal via BITSTRING.
//! • Bool:   TRUE / FALSE keywords.
//!
//! Ref: <https://duckdb.org/docs/sql/data_types/literal_types>

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "DuckDb";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Keyword;

pub(super) const IDENTIFIER_QUOTE: Option<IdentifierQuote> = Some(IdentifierQuote::DoubleQuote);

pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::DollarNamed;

pub(super) fn escape_string(s: &str) -> String {
    // DuckDB uses '' for quote escaping (ANSI-compatible).
    // Backslash is NOT a special escape character in default mode.
    super::escape_single_quote(s)
}

pub(super) fn quote_string(s: &str) -> String {
    super::wrap_single_quote(s)
}

pub(super) fn encode_bytes(bytes: &[u8]) -> String {
    format!("X'{}'", super::bytes_to_hex(bytes))
}

pub(super) fn encode_bits(bits: &str) -> String {
    debug_assert!(
        bits.chars().all(|c| c == '0' || c == '1'),
        "encode_bits: invalid input: {bits}"
    );

    // DuckDB supports bitstring literals with B prefix
    format!("B'{}'", bits)
}
