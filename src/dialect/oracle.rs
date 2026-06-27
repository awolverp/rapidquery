// Ref: https://docs.oracle.com/en/database/oracle/oracle-database/21/sqlrf/Literals.html
//
// • Strings: single-quoted; `''` for embedded single-quote.
//            Alternatively q'[…]' alternative quoting.
// • Identifiers: double-quote quoted.
// • Bytes:  HEXTORAW('DEADBEEF').
// • Bits:   No native bit literal; use integer value.
// • Bool:   Oracle < 23c has no BOOLEAN in SQL; 23c onwards has TRUE/FALSE.
//           We default to 1/0 for broadest compatibility.

implement_pyclass! {
    /// A [`Dialect`] for [Oracle Databases](https://docs.oracle.com/en/database/oracle/oracle-database/21/sqlrf/index.html)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    [skip_from_py_object] PyOracleDialect as "OracleDialect";
}

impl super::Dialect for PyOracleDialect {
    fn name(&self) -> &'static str {
        "Oracle"
    }

    fn bool_format(&self) -> super::BoolFormat {
        // TRUE/FALSE only available in Oracle 23c+; default to Int for safety
        super::BoolFormat::Int
    }

    fn identifier_quotes(&self) -> Option<(char, char)> {
        Some(('"', '"'))
    }

    fn escape_string(&self, s: &str) -> String {
        // Oracle: only single-quote doubling; no backslash escaping.
        // NUL bytes are not supported in VARCHAR2 literals.
        super::escape_single_quote(s).replace('\0', "")
    }

    fn quote_string(&self, s: &mut String) {
        super::wrap_single_quote(s);
    }

    fn encode_bytes(&self, bytes: &[u8]) -> String {
        format!("HEXTORAW('{}')", super::bytes_to_hex(bytes))
    }

    fn encode_bits(&self, bits: &str) -> String {
        debug_assert!(
            bits.chars().all(|c| c == '0' || c == '1'),
            "encode_bits: invalid input: {bits}"
        );

        format!("{}", i64::from_str_radix(bits, 2).unwrap_or(0))
    }
}

super::implement_dialect_pymethods!(PyOracleDialect);
