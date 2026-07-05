// • Strings: single-quoted; backslash-escape sequences.
// • Identifiers: back-tick quoted.
// • Bytes:  X'DEADBEEF' hex literal (BINARY type).
// • Bits:   No native bit literal; we use CAST(… AS BINARY) from INT.
// • Bool:   TRUE / FALSE keywords.
//
// Ref: https://docs.databricks.com/en/sql/language-manual/sql-ref-literals.html

implement_pyclass! {
    /// A dialect for [Databricks SQL](https://www.databricks.com/)
    ///
    /// See <https://docs.databricks.com/en/sql/language-manual/index.html>.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    [skip_from_py_object, extends=super::base::PyBaseDialect] PyDatabricksDialect as "DatabricksDialect";
}

impl super::Dialect for PyDatabricksDialect {
    const NAME: &'static str = "Databricks";

    const IDENTIFIER_QUOTE: Option<super::IdentifierQuote> = Some(super::IdentifierQuote::Backtick);

    const PLACEHOLDER_SYNTAX: super::PlaceholderSyntax = super::PlaceholderSyntax::Colon;

    fn escape_string(&self, s: &str) -> String {
        let mut out = String::with_capacity(s.len() + s.len() / 4);
        for ch in s.chars() {
            match ch {
                '\\' => out.push_str("\\\\"),
                '\'' => out.push_str("\\'"),
                '"' => out.push_str("\\\""),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                '\0' => out.push_str("\\0"),
                c if (c as u32) < 32 || c as u32 == 127 => {
                    out.push_str(&format!("\\u{:04X}", c as u32));
                }
                c => out.push(c),
            }
        }
        out
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

        // Databricks supports CAST with integer literals
        format!(
            "CAST({}L AS BINARY)",
            i64::from_str_radix(bits, 2).unwrap_or(0)
        )
    }
}

super::implement_dialect_pymethods!(PyDatabricksDialect);
