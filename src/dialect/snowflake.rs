// • Strings: single-quoted; backslash-escape sequences.
// • Identifiers: double-quote quoted.
// • Bytes:  X'DEADBEEF'  hex literal (BINARY type).
// • Bits:   No native bit literal; use TO_BINARY with integer.
// • Bool:   TRUE / FALSE keywords.
//
// Ref: https://docs.snowflake.com/en/sql-reference/data-types-text
//      https://docs.snowflake.com/en/sql-reference/literals

implement_pyclass! {
    /// A dialect for [Snowflake](https://www.snowflake.com/)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    [skip_from_py_object, extends=super::base::PyBaseDialect] PySnowflakeDialect as "SnowflakeDialect";
}

impl super::Dialect for PySnowflakeDialect {
    const NAME: &'static str = "Snowflake";

    const IDENTIFIER_QUOTE: Option<super::IdentifierQuote> =
        Some(super::IdentifierQuote::DoubleQuote);

    const PLACEHOLDER_SYNTAX: super::PlaceholderSyntax = super::PlaceholderSyntax::Colon;

    fn escape_string(&self, s: &str) -> String {
        let mut out = String::with_capacity(s.len() + s.len() / 4);
        for ch in s.chars() {
            match ch {
                '\\' => out.push_str("\\\\"),
                '\'' => out.push_str("\\'"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                '\0' => out.push_str("\\0"),
                c if (c as u32) < 32 || c as u32 == 127 => {
                    out.push_str(&format!("\\x{:02X}", c as u32));
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

        let val = i64::from_str_radix(bits, 2).unwrap_or(0);
        format!("TO_BINARY(LPAD(TO_HEX({}), 16, '0'), 'HEX')", val)
    }
}

super::implement_dialect_pymethods!(PySnowflakeDialect);
