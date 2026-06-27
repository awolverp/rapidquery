// • Strings: single-quoted; `''` for embedded single-quote.
//            Teradata supports escape sequences only in named character sets;
//            safest approach is quote-doubling only.
// • Identifiers: double-quote quoted.
// • Bytes:  '…'XB  byte literal (hex digits in single quotes with XB suffix).
//           OR  '…'XBV for VARBYTE.
// • Bits:   '10110101'B  bit literal.
// • Bool:   TRUE / FALSE (Teradata 13.x+ with BYTEINT or BOOLEAN).
//
// Ref: https://docs.teradata.com/r/Teradata-VantageCloud-Lake/SQL-Fundamentals/SQL-Data-Types-and-Literals

implement_pyclass! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    [skip_from_py_object] PyTeradataDialect as "TeradataDialect";
}

impl super::Dialect for PyTeradataDialect {
    fn name(&self) -> &'static str {
        "Teradata"
    }

    fn bool_format(&self) -> super::BoolFormat {
        super::BoolFormat::Keyword
    }

    fn identifier_quotes(&self) -> Option<(char, char)> {
        Some(('"', '"'))
    }

    fn escape_string(&self, s: &str) -> String {
        // Teradata: only single-quote doubling; no backslash interpretation.
        // NUL bytes are not supported.
        super::escape_single_quote(s).replace('\0', "")
    }

    fn quote_string(&self, s: &mut String) {
        super::wrap_single_quote(s);
    }

    fn encode_bytes(&self, bytes: &[u8]) -> String {
        // Teradata byte literal: '<HEX>'XBV
        format!("'{}'XBV", super::bytes_to_hex(bytes))
    }

    fn encode_bits(&self, bits: &str) -> String {
        debug_assert!(
            bits.chars().all(|c| c == '0' || c == '1'),
            "encode_bits: invalid input: {bits}"
        );

        // Teradata bit literal: '<bits>'B
        format!("'{}'B", bits)
    }
}

super::implement_dialect_pymethods!(PyTeradataDialect);
