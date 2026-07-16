//! A dialect for [Databricks SQL](https://www.databricks.com/).
//!
//! See <https://docs.databricks.com/en/sql/language-manual/index.html>.
//!
//! • Strings: single-quoted; backslash-escape sequences.
//! • Identifiers: back-tick quoted.
//! • Bytes:  X'DEADBEEF' hex literal (BINARY type).
//! • Bits:   No native bit literal; we use CAST(… AS BINARY) from INT.
//! • Bool:   TRUE / FALSE keywords.
//!
//! Ref: <https://docs.databricks.com/en/sql/language-manual/sql-ref-literals.html>

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "Databricks";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Keyword;

pub(super) const IDENTIFIER_QUOTE: Option<IdentifierQuote> = Some(IdentifierQuote::Backtick);

pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::Colon;

pub(super) fn escape_string(s: &str) -> String {
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

pub(super) fn quote_string(s: &str) -> String {
    super::wrap_single_quote(s)
}

pub(super) fn encode_bytes(bytes: &[u8]) -> String {
    format!("X'{}'", super::bytes_to_hex(bytes))
}

pub(super) fn encode_bits(bits: &str) -> String {
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
