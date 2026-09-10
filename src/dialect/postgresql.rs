//! [PostgreSQL](https://www.postgresql.org/).
//!
//! • Strings: always `E'…'` escape-string syntax so `\n` / `\xNN` are
//!   interpreted. Standard `'…'` with `''` doubling would not honor those
//!   sequences (and `standard_conforming_strings` is on by default).
//! • Identifiers: double-quote quoted; `"` doubled.
//! • Bytes: `'\xDEADBEEF'::bytea` (hex format, 9.0+).
//! • Bits: `B'10110101'` bit-string literal.
//! • Bool: `TRUE` / `FALSE`.
//! • Placeholders: `:name` (not native; Postgres uses `$1`. Common clients
//!   rewrite named params).
//!
//! Ref: <https://www.postgresql.org/docs/current/sql-syntax-lexical.html>
//!      <https://www.postgresql.org/docs/current/datatype-binary.html>
//!      <https://www.postgresql.org/docs/current/datatype-bit.html>

use crate::internal::utils::replace_to;

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "PostgreSQL";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Keyword;

pub(super) const IDENTIFIER_QUOTE: IdentifierQuote = IdentifierQuote::DoubleQuote;

pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::Colon;

#[cfg_attr(feature = "nightly", optimize(speed))]
pub(super) fn escape_string<W: std::fmt::Write>(writer: &mut W, string: &str) -> std::fmt::Result {
    for ch in string.chars() {
        match ch {
            '\\' => writer.write_str("\\\\")?,
            '\'' => writer.write_str("\\'")?,
            '\n' => writer.write_str("\\n")?,
            '\r' => writer.write_str("\\r")?,
            '\t' => writer.write_str("\\t")?,
            '\0' => writer.write_str("\\x00")?,
            c if (c as u32) < 32 || c as u32 == 127 => {
                super::write_byte_hex_escape(writer, c as u8)?;
            }
            c => writer.write_char(c)?,
        }
    }
    Ok(())
}

pub(super) fn quote_string<W: std::fmt::Write>(writer: &mut W, string: &str) -> std::fmt::Result {
    writer.write_str("E'")?;
    writer.write_str(string)?;
    writer.write_char('\'')
}

pub(super) fn escape_identifier<W: std::fmt::Write>(
    writer: &mut W,
    ident: &str,
) -> std::fmt::Result {
    replace_to(ident, '"', "\"\"", writer)
}

pub(super) fn encode_bytes<W: std::fmt::Write>(writer: &mut W, bytes: &[u8]) -> std::fmt::Result {
    writer.write_str("'\\x")?;
    super::bytes_to_hex(writer, bytes)?;
    writer.write_str("'::bytea")
}

pub(super) fn encode_bits<W: std::fmt::Write>(writer: &mut W, bits: &str) -> std::fmt::Result {
    super::debug_assert_bits(bits);
    write!(writer, "B'{bits}'")
}
