// • Strings: single-quoted; backslash-escape sequences (Java conventions).
// • Identifiers: back-tick quoted.
// • Bytes:  No native byte literal; use CAST with hex string.
// • Bits:   No native bit literal; use CAST with integer.
// • Bool:   TRUE / FALSE keywords.
//
// Ref: https://cwiki.apache.org/confluence/display/Hive/LanguageManual+Types

implement_pyclass! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    [skip_from_py_object] PyHiveDialect as "HiveDialect";
}

impl super::Dialect for PyHiveDialect {
    fn name(&self) -> &'static str {
        "Hive"
    }

    fn bool_format(&self) -> super::BoolFormat {
        super::BoolFormat::Keyword
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

    fn quote_string(&self, s: &mut String) {
        super::wrap_single_quote(s);
    }

    fn encode_bytes(&self, bytes: &[u8]) -> String {
        // Hive: BINARY from hex string via unhex()
        format!("UNHEX('{}')", super::bytes_to_hex(bytes))
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

super::implement_dialect_pymethods!(PyHiveDialect);
