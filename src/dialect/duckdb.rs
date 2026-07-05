// • Strings: single-quoted; `''` for embedded single-quote.
// • Identifiers: double-quote quoted.
// • Bytes:  '\xNN…'::BLOB  or  X'DEADBEEF' hex literal.
// • Bits:   0b<bits>::INTEGER  or  BIT literal via BITSTRING.
// • Bool:   TRUE / FALSE keywords.
//
// Ref: https://duckdb.org/docs/sql/data_types/literal_types

implement_pyclass! {
    /// A dialect for [DuckDB](https://duckdb.org/)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    [skip_from_py_object, extends=super::base::PyBaseDialect] PyDuckDbDialect as "DuckDbDialect";
}

impl super::Dialect for PyDuckDbDialect {
    const NAME: &'static str = "DuckDB";

    const IDENTIFIER_QUOTE: Option<super::IdentifierQuote> =
        Some(super::IdentifierQuote::DoubleQuote);

    const PLACEHOLDER_SYNTAX: super::PlaceholderSyntax = super::PlaceholderSyntax::DollarNamed;

    fn escape_string(&self, s: &str) -> String {
        // DuckDB uses '' for quote escaping (ANSI-compatible).
        // Backslash is NOT a special escape character in default mode.
        super::escape_single_quote(s)
    }

    fn quote_string(&self, s: &str) -> String {
        super::wrap_single_quote(s)
    }

    fn encode_bytes(&self, bytes: &[u8]) -> String {
        format!("X'{}'", super::bytes_to_hex(bytes))
    }

    fn encode_bits(&self, bits: &str) -> String {
        debug_assert!(
            bits.chars().all(|c| c == '0' || c == '1'),
            "encode_bits: invalid input: {bits}"
        );

        // DuckDB supports bitstring literals with B prefix
        format!("B'{}'", bits)
    }
}

super::implement_dialect_pymethods!(PyDuckDbDialect);
