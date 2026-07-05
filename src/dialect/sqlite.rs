// • Strings: single-quoted; `''` for embedded single-quote.
//            SQLite does NOT interpret backslash escapes in string literals.
// • Identifiers: double-quote quoted (also supports back-tick and […]).
// • Bytes:  X'DEADBEEF'  BLOB hex literal.
// • Bits:   No native bit literal; use integer value.
// • Bool:   1 / 0 (no native BOOLEAN type).
//
// Ref: https://www.sqlite.org/lang_expr.html

implement_pyclass! {
    /// A dialect for [SQLite](https://www.sqlite.org)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    [skip_from_py_object, extends=super::base::PyBaseDialect] PySQLiteDialect as "SQLiteDialect";
}

impl super::Dialect for PySQLiteDialect {
    const NAME: &'static str = "SQLite";

    const BOOL_FORMAT: super::BoolFormat = super::BoolFormat::Int;

    const IDENTIFIER_QUOTE: Option<super::IdentifierQuote> =
        Some(super::IdentifierQuote::DoubleQuote);

    const PLACEHOLDER_SYNTAX: super::PlaceholderSyntax = super::PlaceholderSyntax::Colon;

    fn escape_string(&self, s: &str) -> String {
        // SQLite does not recognise backslash escape sequences.
        // NUL character (U+0000) terminates the string in the C library;
        // we remove it to be safe.
        super::escape_single_quote(s).replace('\0', "")
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

        format!("{}", i64::from_str_radix(bits, 2).unwrap_or(0))
    }
}

super::implement_dialect_pymethods!(PySQLiteDialect);
