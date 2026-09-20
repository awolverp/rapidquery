//! SQL dialect definitions.
//!
//! A dialect encodes how one database engine quotes identifiers, escapes
//! string/byte/bit literals, formats booleans, and spells named placeholders.
//!
//! Dialects are selected at runtime through the [`SqlDialect`] enum. Each
//! variant is a marker; behavior lives in the matching private module.
//!
//! # Placeholders
//!
//! [`PlaceholderSyntax`] is the *named* form (`@param_N`, `:param_N`,
//! `$param_N`). Several engines have no native named placeholders (Hive,
//! MySQL, Postgres `$1`, SQLite `?`); those dialects still pick a named
//! style that common Python clients rewrite client-side.
//!
//! # Quoting contract
//!
//! `escape_*` writes the **body** only (special characters neutralized).
//! `quote_*` wraps an **already-escaped** body in the dialect's delimiters.
//! Call `escape` then `quote`, or use `quote_identifier` which does both
//! for identifiers.

use std::fmt::{self, Write};

use crate::internal::utils::{replace_ascii_to, replace_to};

mod bigquery;
mod databricks;
mod duckdb;
mod hive;
mod mssql;
mod mysql;
mod oracle;
mod postgresql;
mod redshift;
mod snowflake;
mod spark;
mod sqlite;
mod teradata;
pub mod py;

#[cfg(test)]
mod tests;

pub use py::dialect_mod;

/// How a boolean literal is spelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BoolFormat {
    /// `TRUE` / `FALSE`
    Keyword,
    /// `1` / `0`
    Int,
}

impl BoolFormat {
    fn write<W: Write>(self, writer: &mut W, value: bool) -> fmt::Result {
        match (self, value) {
            (Self::Keyword, true) => writer.write_str("TRUE"),
            (Self::Keyword, false) => writer.write_str("FALSE"),
            (Self::Int, true) => writer.write_char('1'),
            (Self::Int, false) => writer.write_char('0'),
        }
    }
}

/// Delimiters wrapped around a quoted identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IdentifierQuote {
    /// `'name'` (Teradata).
    SingleQuote,
    /// `"name"` (Postgres, SQLite, Oracle, …).
    DoubleQuote,
    /// `` `name` `` (MySQL, BigQuery, Spark, …).
    Backtick,
    /// `[name]` (SQL Server).
    SquareBracket,
}

impl IdentifierQuote {
    /// Opening and closing delimiter characters.
    #[inline]
    pub const fn delimiters(self) -> (char, char) {
        match self {
            Self::SingleQuote => ('\'', '\''),
            Self::DoubleQuote => ('"', '"'),
            Self::Backtick => ('`', '`'),
            Self::SquareBracket => ('[', ']'),
        }
    }
}

/// Named-placeholder spelling used when building parameterized SQL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PlaceholderSyntax {
    /// `@param_name` (SQL Server, BigQuery).
    AtSign,
    /// `:param_name` (Oracle, and many client-side rewrites).
    Colon,
    /// `$param_name` (DuckDB; not Postgres positional `$1`).
    DollarNamed,
}

impl PlaceholderSyntax {
    /// Character written immediately before the parameter name.
    #[inline]
    pub const fn sigil(self) -> char {
        match self {
            Self::AtSign => '@',
            Self::Colon => ':',
            Self::DollarNamed => '$',
        }
    }
}

// ---------------------------------------------------------------------------
// Shared write helpers (no heap allocation beyond the caller's `Write`).
// ---------------------------------------------------------------------------

const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

/// Write `byte` as two uppercase hex digits.
#[inline]
pub(super) fn write_hex2<W: Write>(writer: &mut W, byte: u8) -> fmt::Result {
    let buf = [
        HEX_DIGITS[(byte >> 4) as usize],
        HEX_DIGITS[(byte & 0x0F) as usize],
    ];
    // SAFETY: HEX_DIGITS is ASCII, so `buf` is valid UTF-8.
    writer.write_str(unsafe { std::str::from_utf8_unchecked(&buf) })
}

/// Write `bytes` as a contiguous uppercase hex run (`DEADBEEF`).
#[inline]
pub(super) fn bytes_to_hex<W: Write>(writer: &mut W, bytes: &[u8]) -> fmt::Result {
    for &b in bytes {
        write_hex2(writer, b)?;
    }
    Ok(())
}

/// Write `\xNN` for a single byte (uppercase hex).
#[inline]
pub(super) fn write_byte_hex_escape<W: Write>(writer: &mut W, byte: u8) -> fmt::Result {
    writer.write_str("\\x")?;
    write_hex2(writer, byte)
}

/// Write `\uNNNN` for a codepoint that fits in 16 bits.
#[inline]
pub(super) fn write_unicode4_escape<W: Write>(writer: &mut W, code: u32) -> fmt::Result {
    debug_assert!(code <= 0xFFFF);
    writer.write_str("\\u")?;
    write_hex2(writer, (code >> 8) as u8)?;
    write_hex2(writer, code as u8)
}

/// `X'DEADBEEF'` blob literal, used by MySQL, SQLite, Spark, Snowflake, …
#[inline]
pub(super) fn write_x_hex_literal<W: Write>(writer: &mut W, bytes: &[u8]) -> fmt::Result {
    writer.write_str("X'")?;
    bytes_to_hex(writer, bytes)?;
    writer.write_char('\'')
}

/// ANSI string-body escape: `'` → `''`. Does not strip NUL.
#[inline]
pub(super) fn escape_single_quote<W: Write>(writer: &mut W, string: &str) -> fmt::Result {
    replace_to(string, '\'', "''", writer)
}

/// ANSI string-body escape plus NUL stripping (`'` → `''`, `\0` deleted).
///
/// Used by engines whose string types cannot hold NUL (T-SQL, Oracle,
/// Redshift, SQLite, Teradata).
#[inline]
pub(super) fn escape_quote_and_strip_nul<W: Write>(writer: &mut W, string: &str) -> fmt::Result {
    const REPLACEMENTS: [(u8, &str); 2] = [(b'\'', "''"), (b'\0', "")];
    replace_ascii_to(string, &REPLACEMENTS, writer)
}

/// Wrap an already-escaped body in single quotes (`'…'`).
#[inline]
pub(super) fn wrap_single_quote<W: Write>(writer: &mut W, string: &str) -> fmt::Result {
    writer.write_char('\'')?;
    writer.write_str(string)?;
    writer.write_char('\'')
}

/// Debug-only check that `bits` is a `'0'`/`'1'` string.
#[inline]
pub(super) fn debug_assert_bits(bits: &str) {
    debug_assert!(
        bits.chars().all(|c| c == '0' || c == '1'),
        "encode_bits: invalid input: {bits}"
    );
}

/// Parse `bits` as a base-2 `i64`.
///
/// Empty strings and values that overflow `i64` (including bit strings
/// longer than 64) return `0`. That fallback is the current contract until
/// a later layer can surface a real error. Callers must pass only `'0'`/`'1'`.
#[inline]
pub(super) fn bits_as_i64(bits: &str) -> i64 {
    debug_assert_bits(bits);
    i64::from_str_radix(bits, 2).unwrap_or(0)
}

/// Run `write` into a new `String`. `fmt::Write` for `String` is infallible.
#[inline]
fn to_string(write: impl FnOnce(&mut String) -> fmt::Result) -> String {
    let mut buf = String::new();
    let _ = write(&mut buf);
    buf
}

/// Dispatches to the per-dialect free function of the same name, forwarding
/// any extra arguments. Keeps [`SqlDialect`]'s methods free of repetitive
/// match boilerplate as new dialects/methods are added.
///
/// Use `dispatch!(self, fn METHOD, arg1, arg2, ...)` for functions and
/// `dispatch!(self, const NAME)` for associated consts (no call parens).
macro_rules! dispatch {
    ($self:expr, fn $method:ident $(, $arg:expr)*) => {
        match $self {
            SqlDialect::BigQuery => bigquery::$method($($arg),*),
            SqlDialect::Databricks => databricks::$method($($arg),*),
            SqlDialect::DuckDb => duckdb::$method($($arg),*),
            SqlDialect::Hive => hive::$method($($arg),*),
            SqlDialect::MsSql => mssql::$method($($arg),*),
            SqlDialect::MySql => mysql::$method($($arg),*),
            SqlDialect::Oracle => oracle::$method($($arg),*),
            SqlDialect::PostgreSql => postgresql::$method($($arg),*),
            SqlDialect::Redshift => redshift::$method($($arg),*),
            SqlDialect::Snowflake => snowflake::$method($($arg),*),
            SqlDialect::Spark => spark::$method($($arg),*),
            SqlDialect::Sqlite => sqlite::$method($($arg),*),
            SqlDialect::Teradata => teradata::$method($($arg),*),
        }
    };
    ($self:expr, const $item:ident) => {
        match $self {
            SqlDialect::BigQuery => bigquery::$item,
            SqlDialect::Databricks => databricks::$item,
            SqlDialect::DuckDb => duckdb::$item,
            SqlDialect::Hive => hive::$item,
            SqlDialect::MsSql => mssql::$item,
            SqlDialect::MySql => mysql::$item,
            SqlDialect::Oracle => oracle::$item,
            SqlDialect::PostgreSql => postgresql::$item,
            SqlDialect::Redshift => redshift::$item,
            SqlDialect::Snowflake => snowflake::$item,
            SqlDialect::Spark => spark::$item,
            SqlDialect::Sqlite => sqlite::$item,
            SqlDialect::Teradata => teradata::$item,
        }
    };
}

/// The SQL dialects supported by this crate, selectable at runtime.
///
/// Each variant carries no data; dialect-specific behavior lives in the
/// corresponding private module and is reached through the methods below.
///
/// [`SqlDialect::name`] is the Python class name (`DuckDB`, `PostgreSQL`,
/// `SQLite`, …). Rust variant identifiers stay idiomatic (`DuckDb`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SqlDialect {
    /// [Google BigQuery](https://cloud.google.com/bigquery/)
    BigQuery,
    /// [Databricks SQL](https://www.databricks.com/)
    Databricks,
    /// [DuckDB](https://duckdb.org/)
    DuckDb,
    /// [Apache Hive](https://hive.apache.org/)
    Hive,
    /// [Microsoft SQL Server](https://www.microsoft.com/en-us/sql-server/)
    MsSql,
    /// [MySQL](https://www.mysql.com/)
    MySql,
    /// [Oracle Database](https://docs.oracle.com/en/database/oracle/oracle-database/)
    Oracle,
    /// [PostgreSQL](https://www.postgresql.org/)
    PostgreSql,
    /// [Amazon Redshift](https://aws.amazon.com/redshift/)
    Redshift,
    /// [Snowflake](https://www.snowflake.com/)
    Snowflake,
    /// [Apache Spark SQL](https://spark.apache.org/docs/latest/sql-ref.html)
    Spark,
    /// [SQLite](https://www.sqlite.org/)
    Sqlite,
    /// [Teradata](https://docs.teradata.com/)
    Teradata,
}

impl SqlDialect {
    /// Display name, identical to the Python class name (e.g. `"PostgreSQL"`, `"DuckDB"`).
    #[inline]
    pub fn name(&self) -> &'static str {
        dispatch!(self, const NAME)
    }

    /// Named-placeholder style for parameterized SQL.
    #[inline]
    pub fn placeholder_syntax(&self) -> PlaceholderSyntax {
        dispatch!(self, const PLACEHOLDER_SYNTAX)
    }

    /// Native bit-string literals (`B'…'`, `b'…'`, `'…'B`).
    #[inline]
    pub fn supports_bit(self) -> bool {
        matches!(
            self,
            Self::PostgreSql | Self::DuckDb | Self::MySql | Self::Teradata
        )
    }

    /// `ARRAY[…]` constructor syntax.
    #[inline]
    pub fn supports_array(self) -> bool {
        matches!(self, Self::PostgreSql | Self::DuckDb | Self::BigQuery)
    }

    /// Native `INET` / `CIDR` types.
    #[inline]
    pub fn supports_inet(self) -> bool {
        matches!(self, Self::PostgreSql | Self::DuckDb)
    }

    /// Native `MACADDR` type.
    #[inline]
    pub fn supports_mac_address(self) -> bool {
        matches!(self, Self::PostgreSql)
    }

    /// `{IGNORE|RESPECT} NULLS` inside the argument list
    /// (`FIRST_VALUE(x IGNORE NULLS)`), as documented for BigQuery and DuckDB.
    /// Other dialects emit it after the closing parenthesis.
    #[inline]
    pub fn null_treatment_inside_parens(self) -> bool {
        matches!(self, Self::BigQuery | Self::DuckDb)
    }

    /// Write a boolean literal (`TRUE`/`FALSE` or `1`/`0`).
    #[inline]
    pub fn format_boolean<W: Write>(&self, writer: &mut W, value: bool) -> fmt::Result {
        dispatch!(self, const BOOL_FORMAT).write(writer, value)
    }

    /// [`format_boolean`](Self::format_boolean) into an owned `String`.
    #[inline]
    pub fn format_boolean_as_string(&self, value: bool) -> String {
        let mut buf = String::with_capacity(5);
        let _ = self.format_boolean(&mut buf, value);
        buf
    }

    /// Write `ident` with delimiter characters doubled/escaped, **without**
    /// wrapping it. Use [`quote_identifier`](Self::quote_identifier) to also
    /// add delimiters.
    #[inline]
    pub fn escape_identifier<W: Write>(&self, writer: &mut W, ident: &str) -> fmt::Result {
        dispatch!(self, fn escape_identifier, writer, ident)
    }

    /// [`escape_identifier`](Self::escape_identifier) into an owned `String`.
    #[inline]
    pub fn escape_identifier_as_string(&self, ident: &str) -> String {
        to_string(|buf| self.escape_identifier(buf, ident))
    }

    /// Escape `ident` and wrap it in this dialect's identifier delimiters.
    #[inline]
    pub fn quote_identifier<W: Write>(&self, writer: &mut W, ident: &str) -> fmt::Result {
        let (start, end) = dispatch!(self, const IDENTIFIER_QUOTE).delimiters();
        writer.write_char(start)?;
        self.escape_identifier(writer, ident)?;
        writer.write_char(end)
    }

    /// [`quote_identifier`](Self::quote_identifier) into an owned `String`.
    #[inline]
    pub fn quote_identifier_as_string(&self, ident: &str) -> String {
        to_string(|buf| self.quote_identifier(buf, ident))
    }

    /// Write `string` with special characters neutralized, **without** wrapping
    /// it in quotes. Call [`quote_string`](Self::quote_string) on the result
    /// to produce a complete literal.
    #[inline]
    pub fn escape_string<W: Write>(&self, writer: &mut W, string: &str) -> fmt::Result {
        dispatch!(self, fn escape_string, writer, string)
    }

    /// [`escape_string`](Self::escape_string) into an owned `String`.
    #[inline]
    pub fn escape_string_as_string(&self, string: &str) -> String {
        to_string(|buf| self.escape_string(buf, string))
    }

    /// Wrap an **already-escaped** string in this dialect's literal quoting
    /// (`'…'`, `E'…'`, `N'…'`).
    #[inline]
    pub fn quote_string<W: Write>(&self, writer: &mut W, string: &str) -> fmt::Result {
        dispatch!(self, fn quote_string, writer, string)
    }

    /// [`quote_string`](Self::quote_string) into an owned `String`.
    #[inline]
    pub fn quote_string_as_string(&self, string: &str) -> String {
        to_string(|buf| self.quote_string(buf, string))
    }

    /// Escape `raw` and wrap it as a complete string literal.
    ///
    /// Equivalent to `quote_string(escape_string(raw))` with no intermediate
    /// `String`: writes the dialect prefix (`'` / `E'` / `N'`), then the
    /// escaped body, then `'`.
    #[inline]
    pub fn write_string_literal<W: Write>(&self, writer: &mut W, raw: &str) -> fmt::Result {
        match self {
            Self::PostgreSql => writer.write_str("E'")?,
            Self::MsSql => writer.write_str("N'")?,
            _ => writer.write_char('\'')?,
        }
        self.escape_string(writer, raw)?;
        writer.write_char('\'')
    }

    /// [`write_string_literal`](Self::write_string_literal) into an owned `String`.
    #[inline]
    pub fn write_string_literal_as_string(&self, raw: &str) -> String {
        to_string(|buf| self.write_string_literal(buf, raw))
    }

    /// Write `bytes` as a native blob/bytea/varbyte literal.
    #[inline]
    pub fn encode_bytes<W: Write>(&self, writer: &mut W, bytes: &[u8]) -> fmt::Result {
        dispatch!(self, fn encode_bytes, writer, bytes)
    }

    /// [`encode_bytes`](Self::encode_bytes) into an owned `String`.
    #[inline]
    pub fn encode_bytes_as_string(&self, bytes: &[u8]) -> String {
        to_string(|buf| self.encode_bytes(buf, bytes))
    }

    /// Write a bit-string (`'0'`/`'1'` characters) as a native literal.
    ///
    /// Callers must pass only `'0'` and `'1'`; that is checked with
    /// `debug_assert!`. Dialects that fold bits into an `i64` return `0` on
    /// overflow (see [`bits_as_i64`]).
    #[inline]
    pub fn encode_bits<W: Write>(&self, writer: &mut W, bits: &str) -> fmt::Result {
        dispatch!(self, fn encode_bits, writer, bits)
    }

    /// [`encode_bits`](Self::encode_bits) into an owned `String`.
    #[inline]
    pub fn encode_bits_as_string(&self, bits: &str) -> String {
        to_string(|buf| self.encode_bits(buf, bits))
    }
}
