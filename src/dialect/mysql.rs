// • Strings: single-quoted; backslash-escape sequences.
// • Identifiers: back-tick quoted.
// • Bytes:  X'DEADBEEF'  or  0xDEADBEEF.
// • Bits:   b'10110101'  bit literal.
// • Bool:   1 / 0 (TINYINT; no native BOOLEAN literal).
//
// Ref: https://dev.mysql.com/doc/refman/8.0/en/string-literals.html
//      https://dev.mysql.com/doc/refman/8.0/en/bit-value-literals.html

implement_pyclass! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    [skip_from_py_object] PyMySqlDialect as "MySqlDialect";
}

impl super::Dialect for PyMySqlDialect {
    fn name(&self) -> &'static str {
        "MySQL"
    }

    fn bool_format(&self) -> super::BoolFormat {
        super::BoolFormat::Int
    }

    fn identifier_quotes(&self) -> Option<(char, char)> {
        Some(('`', '`'))
    }

    fn escape_string(&self, s: &str) -> String {
        let mut out = String::with_capacity(s.len() + s.len() / 4);
        for ch in s.chars() {
            match ch {
                '\\' => out.push_str("\\\\"),
                '\'' => out.push_str("\\'"),
                '"' => out.push_str("\\\""),
                '\0' => out.push_str("\\0"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\x1a' => out.push_str("\\Z"), // Ctrl+Z / SUB (Windows EOF)
                c => out.push(c),
            }
        }
        out
    }

    fn quote_string(&self, s: &mut String) {
        super::wrap_single_quote(s);
    }

    fn encode_bytes(&self, bytes: &[u8]) -> String {
        format!("X'{}'", super::bytes_to_hex(bytes))
    }

    fn encode_bits(&self, bits: &str) -> String {
        debug_assert!(
            bits.chars().all(|c| c == '0' || c == '1'),
            "encode_bits: invalid input: {bits}"
        );

        format!("b'{}'", bits)
    }
}

super::implement_dialect_pymethods!(PyMySqlDialect);
