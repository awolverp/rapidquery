// • Strings: single-quoted with `''` doubling.
//            For control chars we use E'…' escape strings with \xNN.
// • Identifiers: double-quote quoted.
// • Bytes:  '\xDEADBEEF'::bytea  (hex format, PostgreSQL 9.0+).
// • Bits:   B'10110101'  bit string literal.
// • Bool:   TRUE / FALSE keywords.
//
// Ref: https://www.postgresql.org/docs/current/sql-syntax-lexical.html
//      https://www.postgresql.org/docs/current/datatype-binary.html
//      https://www.postgresql.org/docs/current/datatype-bit.html

implement_pyclass! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    [skip_from_py_object] PyPostgreSqlDialect as "PostgreSqlDialect";
}

impl super::Dialect for PyPostgreSqlDialect {
    fn name(&self) -> &'static str {
        "PostgreSQL"
    }

    fn bool_format(&self) -> super::BoolFormat {
        super::BoolFormat::Keyword
    }

    fn identifier_quotes(&self) -> Option<(char, char)> {
        Some(('"', '"'))
    }

    fn escape_string(&self, s: &str) -> String {
        // Use PostgreSQL escape-string syntax (E'…') when control chars present.
        // We always emit E'…' for safety; non-special chars pass through.
        let mut out = String::with_capacity(s.len() + s.len() / 4);

        for ch in s.chars() {
            match ch {
                '\\' => out.push_str("\\\\"),
                '\'' => out.push_str("\\'"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                '\0' => out.push_str("\\x00"),
                c if (c as u32) < 32 || c as u32 == 127 => {
                    out.push_str(&format!("\\x{:02X}", c as u32));
                }
                c => out.push(c),
            }
        }
        out
    }

    fn quote_string(&self, s: &mut String) {
        // Use E'…' escape string syntax so our \-sequences are interpreted.
        let mut out = String::with_capacity(s.len() + 3);
        out.push_str("E'");
        out.push_str(s);
        out.push('\'');
        *s = out;
    }

    fn encode_bytes(&self, bytes: &[u8]) -> String {
        // PostgreSQL hex-format bytea literal
        format!("'\\x{}'::bytea", super::bytes_to_hex(bytes))
    }

    fn encode_bits(&self, bits: &str) -> String {
        debug_assert!(
            bits.chars().all(|c| c == '0' || c == '1'),
            "encode_bits: invalid input: {bits}"
        );

        format!("B'{}'", bits)
    }
}

super::implement_dialect_pymethods!(PyPostgreSqlDialect);
