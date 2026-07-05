/// Represents that how a boolean should be serialized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BoolFormat {
    /// `TRUE` / `FALSE`
    Keyword,
    /// `1` / `0`
    Int,
}

impl BoolFormat {
    pub fn format(self, value: bool) -> &'static str {
        match (self, value) {
            (Self::Keyword, true) => "TRUE",
            (Self::Keyword, false) => "FALSE",
            (Self::Int, true) => "1",
            (Self::Int, false) => "0",
        }
    }
}

/// Represents the various identifier quote styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IdentifierQuote {
    SingleQuote,
    DoubleQuote,
    Backtick,
    SquareBracket,
}

/// Represents the various placeholder syntaxes used for parameterized queries
/// and prepared statements across different SQL dialects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceholderSyntax {
    /// At-Sign Named Parameter syntax (`@param_name`).
    AtSign,

    /// Colon Named Parameter syntax (`:param_name`).
    Colon,

    /// Dollar-Sign Named Parameter syntax (`$param_name`).
    DollarNamed,
}

pub trait Dialect {
    /// Dialect name.
    const NAME: &'static str;

    /// How a boolean should be serialized.
    const BOOL_FORMAT: BoolFormat = BoolFormat::Keyword;

    /// The identifier quote style.
    const IDENTIFIER_QUOTE: Option<IdentifierQuote>;

    /// The placeholder syntax.
    const PLACEHOLDER_SYNTAX: PlaceholderSyntax;

    /// Escapes string literals.
    fn escape_string(&self, string: &str) -> String;

    /// Quotes string literals.
    ///
    /// You should call [`Dialect::escape`] before this method.
    fn quote_string(&self, string: &str) -> String;

    /// Render a byte slice as a literal in the configured form.
    fn encode_bytes(&self, bytes: &[u8]) -> String;

    /// Render a byte slice as a literal in the configured form.
    ///
    /// The `bits` is always validated.
    fn encode_bits(&self, bits: &str) -> String;
}

/// Converts a byte slice to an uppercase hex string.
#[inline]
pub(super) fn bytes_to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        write!(out, "{:02X}", b).unwrap();
    }
    out
}

/// Standard single-quote escape: `'` → `''`.
/// Used by most SQL dialects.
#[inline]
pub(super) fn escape_single_quote(s: &str) -> String {
    s.replace('\'', "''")
}

/// Wraps a (pre-escaped) string in single quotes.
#[inline]
pub(super) fn wrap_single_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    out.push_str(s);
    out.push('\'');
    out
}

#[macro_export]
macro_rules! implement_dialect_pymethods {
    ($name:ident) => {
        #[pyo3::pymethods]
        impl $name {
            #[new]
            fn __new__() -> pyo3::PyClassInitializer<Self> {
                pyo3::PyClassInitializer::from(super::base::PyBaseDialect).add_subclass(Self)
            }
        }
    };
}
pub(super) use implement_dialect_pymethods;

mod base;
pub use base::PyBaseDialect;

mod bigquery;
pub use bigquery::PyBigQueryDialect;

mod databricks;
pub use databricks::PyDatabricksDialect;

mod duckdb;
pub use duckdb::PyDuckDbDialect;

mod hive;
pub use hive::PyHiveDialect;

mod mssql;
pub use mssql::PyMsSqlDialect;

mod mysql;
pub use mysql::PyMySqlDialect;

mod oracle;
pub use oracle::PyOracleDialect;

mod postgresql;
pub use postgresql::PyPostgreSqlDialect;

mod redshift;
pub use redshift::PyRedshiftDialect;

mod snowflake;
pub use snowflake::PySnowflakeDialect;

mod spark;
pub use spark::PySparkDialect;

mod sqlite;
pub use sqlite::PySQLiteDialect;

mod teradata;
pub use teradata::PyTeradataDialect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DialectUnion {
    BigQuery(PyBigQueryDialect),
    Databricks(PyDatabricksDialect),
    DuckDb(PyDuckDbDialect),
    Hive(PyHiveDialect),
    MsSql(PyMsSqlDialect),
    MySql(PyMySqlDialect),
    Oracle(PyOracleDialect),
    PostgreSql(PyPostgreSqlDialect),
    Redshift(PyRedshiftDialect),
    Snowflake(PySnowflakeDialect),
    Spark(PySparkDialect),
    Sqlite(PySQLiteDialect),
    Teradata(PyTeradataDialect),
}

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for DialectUnion {
    type Error = pyo3::PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: pyo3::inspect::PyStaticExpr = type_hint_union!(
        // as object
        type_hint_identifier!("rapidquery._lib.dialect", "BaseDialect"),
        // as type
        type_hint_subscript!(
            type_hint_identifier!("type"),
            type_hint_identifier!("rapidquery._lib.dialect", "BaseDialect")
        )
    );

    fn extract(
        obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>,
    ) -> Result<Self, Self::Error> {
        unsafe {
            let ptr = obj.as_ptr();

            let type_ptr = {
                if pyo3::ffi::PyType_CheckExact(ptr) == 1 {
                    ptr as *mut pyo3::ffi::PyTypeObject
                } else {
                    pyo3::ffi::Py_TYPE(ptr)
                }
            };

            if type_ptr == crate::typeref::BIGQUERY_DIALECT_TYPE {
                return Ok(Self::BigQuery(PyBigQueryDialect));
            }
            if type_ptr == crate::typeref::DATABRICKS_DIALECT_TYPE {
                return Ok(Self::Databricks(PyDatabricksDialect));
            }
            if type_ptr == crate::typeref::DUCKDB_DIALECT_TYPE {
                return Ok(Self::DuckDb(PyDuckDbDialect));
            }
            if type_ptr == crate::typeref::HIVE_DIALECT_TYPE {
                return Ok(Self::Hive(PyHiveDialect));
            }
            if type_ptr == crate::typeref::MS_SQL_DIALECT_TYPE {
                return Ok(Self::MsSql(PyMsSqlDialect));
            }
            if type_ptr == crate::typeref::MYSQL_DIALECT_TYPE {
                return Ok(Self::MySql(PyMySqlDialect));
            }
            if type_ptr == crate::typeref::ORACLE_DIALECT_TYPE {
                return Ok(Self::Oracle(PyOracleDialect));
            }
            if type_ptr == crate::typeref::POSTGRESQL_DIALECT_TYPE {
                return Ok(Self::PostgreSql(PyPostgreSqlDialect));
            }
            if type_ptr == crate::typeref::REDSHIFT_DIALECT_TYPE {
                return Ok(Self::Redshift(PyRedshiftDialect));
            }
            if type_ptr == crate::typeref::SNOWFLAKE_DIALECT_TYPE {
                return Ok(Self::Snowflake(PySnowflakeDialect));
            }
            if type_ptr == crate::typeref::SPARK_DIALECT_TYPE {
                return Ok(Self::Spark(PySparkDialect));
            }
            if type_ptr == crate::typeref::SQLITE_DIALECT_TYPE {
                return Ok(Self::Sqlite(PySQLiteDialect));
            }
            if type_ptr == crate::typeref::TERADATA_DIALECT_TYPE {
                return Ok(Self::Teradata(PyTeradataDialect));
            }

            Err(new_py_error!(
                PyTypeError,
                "Unknown dialect: {}",
                crate::internal::utils::get_type_name(obj.py(), ptr)
            ))
        }
    }
}

macro_rules! impl_fields {
    ($self:expr, $dialect:ident => $expr:expr) => {
        match $self {
            Self::BigQuery($dialect) => $expr,
            Self::Databricks($dialect) => $expr,
            Self::DuckDb($dialect) => $expr,
            Self::Hive($dialect) => $expr,
            Self::MsSql($dialect) => $expr,
            Self::MySql($dialect) => $expr,
            Self::Oracle($dialect) => $expr,
            Self::PostgreSql($dialect) => $expr,
            Self::Redshift($dialect) => $expr,
            Self::Snowflake($dialect) => $expr,
            Self::Spark($dialect) => $expr,
            Self::Sqlite($dialect) => $expr,
            Self::Teradata($dialect) => $expr,
        }
    };

    ($self:expr, $variable:ident) => {
        match $self {
            DialectUnion::BigQuery(_) => <PyBigQueryDialect as Dialect>::$variable,
            DialectUnion::Databricks(_) => <PyDatabricksDialect as Dialect>::$variable,
            DialectUnion::DuckDb(_) => <PyDuckDbDialect as Dialect>::$variable,
            DialectUnion::Hive(_) => <PyHiveDialect as Dialect>::$variable,
            DialectUnion::MsSql(_) => <PyMsSqlDialect as Dialect>::$variable,
            DialectUnion::MySql(_) => <PyMySqlDialect as Dialect>::$variable,
            DialectUnion::Oracle(_) => <PyOracleDialect as Dialect>::$variable,
            DialectUnion::PostgreSql(_) => <PyPostgreSqlDialect as Dialect>::$variable,
            DialectUnion::Redshift(_) => <PyRedshiftDialect as Dialect>::$variable,
            DialectUnion::Snowflake(_) => <PySnowflakeDialect as Dialect>::$variable,
            DialectUnion::Spark(_) => <PySparkDialect as Dialect>::$variable,
            DialectUnion::Sqlite(_) => <PySQLiteDialect as Dialect>::$variable,
            DialectUnion::Teradata(_) => <PyTeradataDialect as Dialect>::$variable,
        }
    };
}

impl DialectUnion {
    pub fn name(&self) -> &'static str {
        impl_fields!(self, NAME)
    }

    pub fn bool_format(&self) -> BoolFormat {
        impl_fields!(self, BOOL_FORMAT)
    }

    pub fn identifier_quote(&self) -> Option<IdentifierQuote> {
        impl_fields!(self, IDENTIFIER_QUOTE)
    }

    pub fn placeholder_syntax(&self) -> PlaceholderSyntax {
        impl_fields!(self, PLACEHOLDER_SYNTAX)
    }

    pub fn escape_string(&self, string: &str) -> String {
        impl_fields!(self, s => s.escape_string(string))
    }

    pub fn quote_string(&self, string: &str) -> String {
        impl_fields!(self, s => s.quote_string(string))
    }

    pub fn encode_bits(&self, bits: &str) -> String {
        impl_fields!(self, s => s.encode_bits(bits))
    }

    pub fn encode_bytes(&self, bytes: &[u8]) -> String {
        impl_fields!(self, s => s.encode_bytes(bytes))
    }
}

#[pyo3::pymodule(name = "dialect")]
pub mod dialect_module {
    #[pymodule_export]
    pub use super::{
        PyBaseDialect, PyBigQueryDialect, PyDatabricksDialect, PyDuckDbDialect, PyHiveDialect,
        PyMsSqlDialect, PyMySqlDialect, PyOracleDialect, PyPostgreSqlDialect, PyRedshiftDialect,
        PySQLiteDialect, PySnowflakeDialect, PySparkDialect, PyTeradataDialect,
    };
}
