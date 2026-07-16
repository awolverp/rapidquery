//! A dialect for [Google BigQuery](https://cloud.google.com/bigquery/).
//!
//! • Strings: single-quoted; escape with backslash sequences.
//! • Identifiers: back-tick quoted.
//! • Bytes:  B'\xNN…'  (byte literal with \x escapes).
//! • Bits:   No native bit-string literal; cast from INT64.
//!           We emit CAST(0b<bits> AS INT64) for best portability.
//! • Bool:   TRUE / FALSE keywords.
//!
//! Ref: <https://cloud.google.com/bigquery/docs/reference/standard-sql/lexical>

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "BigQuery";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Keyword;

pub(super) const IDENTIFIER_QUOTE: Option<IdentifierQuote> = Some(IdentifierQuote::Backtick);

pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::AtSign;

pub(super) fn escape_string(s: &str) -> String {
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

pub(super) fn quote_string(s: &str) -> String {
    super::wrap_single_quote(s)
}

pub(super) fn encode_bytes(bytes: &[u8]) -> String {
    // BigQuery byte literal: B'\xNN\xNN…'
    let mut out = String::with_capacity(4 + bytes.len() * 4);
    out.push_str("B'");
    for b in bytes {
        out.push_str(&format!("\\x{:02X}", b));
    }
    out.push('\'');
    out
}

pub(super) fn encode_bits(bits: &str) -> String {
    debug_assert!(
        bits.chars().all(|c| c == '0' || c == '1'),
        "encode_bits: invalid input: {bits}"
    );

    // BigQuery has no native bit literal; represent as INT64 binary literal.
    format!("CAST(0b{} AS INT64)", bits)
}
