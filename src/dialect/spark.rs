// • Strings: single-quoted; backslash-escape sequences (Java/Scala conventions).
// • Identifiers: back-tick quoted.
// • Bytes:  X'DEADBEEF'  hex literal.
// • Bits:   No native bit literal; cast integer.
// • Bool:   TRUE / FALSE keywords.
//
// Ref: https://spark.apache.org/docs/latest/sql-ref-literals.html

implement_pyclass! {
    /// A [`Dialect`] for [Apache Spark SQL](https://spark.apache.org/docs/latest/sql-ref.html).
    ///
    /// See <https://spark.apache.org/docs/latest/sql-ref-syntax.html>.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    [skip_from_py_object, extends=super::base::PyBaseDialect] PySparkDialect as "SparkDialect";
}

impl super::Dialect for PySparkDialect {
    const NAME: &'static str = "Spark";

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

        format!(
            "CAST({} AS BIGINT)",
            i64::from_str_radix(bits, 2).unwrap_or(0)
        )
    }
}

super::implement_dialect_pymethods!(PySparkDialect);
