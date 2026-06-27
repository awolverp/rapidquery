// • Strings: single-quoted; escape with backslash sequences.
// • Identifiers: back-tick quoted.
// • Bytes:  B'\xNN…'  (byte literal with \x escapes).
// • Bits:   No native bit-string literal; cast from INT64.
//           We emit CAST(0b<bits> AS BYTES) for best portability.
// • Bool:   TRUE / FALSE keywords.
//
// Ref: https://cloud.google.com/bigquery/docs/reference/standard-sql/lexical

implement_pyclass! {
    /// A dialect for [Google Bigquery](https://cloud.google.com/bigquery/)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    [skip_from_py_object] PyBigQueryDialect as "BigQueryDialect";
}

impl super::Dialect for PyBigQueryDialect {
    fn name(&self) -> &'static str {
        "BigQuery"
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
                // ASCII control characters
                c if (c as u32) < 32 || c as u32 == 127 => {
                    out.push_str(&format!("\\x{:02X}", c as u32));
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
        // BigQuery byte literal: B'\xNN\xNN…'
        let mut out = String::with_capacity(4 + bytes.len() * 4);
        out.push_str("B'");
        for b in bytes {
            out.push_str(&format!("\\x{:02X}", b));
        }
        out.push('\'');
        out
    }

    fn encode_bits(&self, bits: &str) -> String {
        debug_assert!(
            bits.chars().all(|c| c == '0' || c == '1'),
            "encode_bits: invalid input: {bits}"
        );

        // BigQuery has no native bit literal; represent as INT64 binary literal.
        format!("CAST(0b{} AS INT64)", bits)
    }
}

super::implement_dialect_pymethods!(PyBigQueryDialect);
