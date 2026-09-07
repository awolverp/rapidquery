//! [Microsoft SQL Server](https://www.microsoft.com/en-us/sql-server/) (T-SQL).
//!
//! • Strings: `N'…'` (NVARCHAR). Only `'` is special (`''`); NUL is stripped
//!   because T-SQL string literals cannot hold U+0000.
//! • Identifiers: `[name]`; `]` doubled as `]]`.
//! • Bytes: `0xDEADBEEF` (no quotes).
//! • Bits: T-SQL `BIT` is boolean-like; multi-bit values become
//!   `CAST(<i64> AS BIGINT)`.
//! • Bool: `1` / `0` (no `TRUE`/`FALSE` keyword).
//! • Placeholders: `@name` (native).
//!
//! Ref: <https://learn.microsoft.com/en-us/sql/t-sql/data-types/constants-transact-sql>

use crate::internal::utils::replace_to;

use super::{BoolFormat, IdentifierQuote, PlaceholderSyntax};

pub(super) const NAME: &str = "MsSql";

pub(super) const BOOL_FORMAT: BoolFormat = BoolFormat::Int;

pub(super) const IDENTIFIER_QUOTE: IdentifierQuote = IdentifierQuote::SquareBracket;

pub(super) const PLACEHOLDER_SYNTAX: PlaceholderSyntax = PlaceholderSyntax::AtSign;

pub(super) fn escape_string<W: std::fmt::Write>(writer: &mut W, string: &str) -> std::fmt::Result {
    super::escape_quote_and_strip_nul(writer, string)
}

pub(super) fn quote_string<W: std::fmt::Write>(writer: &mut W, string: &str) -> std::fmt::Result {
    writer.write_str("N'")?;
    writer.write_str(string)?;
    writer.write_char('\'')
}

pub(super) fn escape_identifier<W: std::fmt::Write>(
    writer: &mut W,
    ident: &str,
) -> std::fmt::Result {
    replace_to(ident, ']', "]]", writer)
}

pub(super) fn encode_bytes<W: std::fmt::Write>(writer: &mut W, bytes: &[u8]) -> std::fmt::Result {
    writer.write_str("0x")?;
    super::bytes_to_hex(writer, bytes)
}

pub(super) fn encode_bits<W: std::fmt::Write>(writer: &mut W, bits: &str) -> std::fmt::Result {
    write!(writer, "CAST({} AS BIGINT)", super::bits_as_i64(bits))
}
