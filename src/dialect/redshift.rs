// • Strings: single-quoted; `''` for embedded single-quote.
//            Redshift is PostgreSQL-derived but does NOT support E'…' by default.
//            Backslash in default mode is literal (not escape); so we double quotes only.
// • Identifiers: double-quote quoted.
// • Bytes:  VARBYTE from hex via FROM_HEX or TO_VARBYTE.
// • Bits:   No native bit literal; cast integer.
// • Bool:   TRUE / FALSE keywords (BOOLEAN type).
//
// Ref: https://docs.aws.amazon.com/redshift/latest/dg/r_Constants.html
//      https://docs.aws.amazon.com/redshift/latest/dg/r_VARBYTE_type.html

implement_pyclass! {
    /// A dialect for [RedShift](https://aws.amazon.com/redshift/)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    [skip_from_py_object] PyRedshiftDialect as "RedshiftDialect";
}

impl super::Dialect for PyRedshiftDialect {
    fn name(&self) -> &'static str {
        "Redshift"
    }

    fn bool_format(&self) -> super::BoolFormat {
        super::BoolFormat::Keyword
    }

    fn identifier_quotes(&self) -> Option<(char, char)> {
        Some(('"', '"'))
    }

    fn escape_string(&self, s: &str) -> String {
        // Redshift: backslash is NOT an escape character in standard mode.
        // Only `'` needs doubling.
        // NUL (U+0000) is not supported in VARCHAR literals.
        super::escape_single_quote(s).replace('\0', "")
    }

    fn quote_string(&self, s: &mut String) {
        super::wrap_single_quote(s);
    }

    fn encode_bytes(&self, bytes: &[u8]) -> String {
        format!("TO_VARBYTE('{}', 'hex')", super::bytes_to_hex(bytes))
    }

    fn encode_bits(&self, bits: &str) -> String {
        debug_assert!(
            bits.chars().all(|c| c == '0' || c == '1'),
            "encode_bits: invalid input: {bits}"
        );

        format!(
            "CAST({} AS BIGINT)",
            i64::from_str_radix(bits, 2).unwrap_or(0)
        )
    }
}

super::implement_dialect_pymethods!(PyRedshiftDialect);
