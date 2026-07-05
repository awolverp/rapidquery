// • Strings: single-quoted; `''` for embedded single-quote.
//            Unicode strings prefixed with N'…'.
// • Identifiers: square-bracket quoted: [name].
// • Bytes:  0xDEADBEEF  (no quotes).
// • Bits:   Cast integer to BIT; MSSQL BIT is boolean-like (0/1).
// • Bool:   1 / 0 (BIT type; no TRUE/FALSE keyword).
//
// Ref: https://learn.microsoft.com/en-us/sql/t-sql/data-types/constants-transact-sql

implement_pyclass! {
/// A dialect for [Microsoft SQL Server](https://www.microsoft.com/en-us/sql-server/)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    [skip_from_py_object, extends=super::base::PyBaseDialect] PyMsSqlDialect as "MsSqlDialect";
}

impl super::Dialect for PyMsSqlDialect {
    const NAME: &'static str = "MsSQL";

    const BOOL_FORMAT: super::BoolFormat = super::BoolFormat::Int;

    const IDENTIFIER_QUOTE: Option<super::IdentifierQuote> =
        Some(super::IdentifierQuote::SquareBracket);

    const PLACEHOLDER_SYNTAX: super::PlaceholderSyntax = super::PlaceholderSyntax::AtSign;

    fn escape_string(&self, s: &str) -> String {
        // T-SQL: only single-quote needs doubling.
        // NUL character (U+0000) is not allowed in T-SQL string literals.
        // We remove it to stay safe.
        super::escape_single_quote(s).replace('\0', "")
    }

    fn quote_string(&self, s: &str) -> String {
        // Prefix with N for Unicode safety (NVARCHAR)
        let mut out = String::with_capacity(s.len() + 3);
        out.push_str("N'");
        out.push_str(s);
        out.push('\'');
        out
    }

    fn encode_bytes(&self, bytes: &[u8]) -> String {
        // T-SQL binary literal: 0xDEADBEEF
        format!("0x{}", super::bytes_to_hex(bytes))
    }

    fn encode_bits(&self, bits: &str) -> String {
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
}

super::implement_dialect_pymethods!(PyMsSqlDialect);
