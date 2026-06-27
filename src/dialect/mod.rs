#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

pub trait Dialect {
    /// Dialect name
    fn name(&self) -> &'static str;

    /// How a boolean should be rendered.
    fn bool_format(&self) -> BoolFormat {
        BoolFormat::Int
    }

    /// The start/end characters used to quote identifiers.
    fn identifier_quotes(&self) -> Option<(char, char)> {
        None
    }

    /// Escapes string literals.
    fn escape_string(&self, string: &str) -> String;

    /// (zero-copy) Quotes string literals.
    ///
    /// You should call [`Dialect::escape`] before this method.
    fn quote_string(&self, string: &mut String);

    /// Calls [`Self::escape_string`] and [`Self::quote_string`] in order.
    fn escape_and_quote_string(&self, string: &str) -> String {
        let mut result = self.escape_string(string);
        self.quote_string(&mut result);
        result
    }

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
pub(super) fn wrap_single_quote(s: &mut String) {
    s.insert(0, '\'');
    s.push('\'');
}

#[macro_export]
macro_rules! implement_dialect_pymethods {
    ($name:ident) => {
        #[pyo3::pymethods]
        impl $name {
            #[new]
            fn __new__() -> Self {
                Self
            }

            fn get_name(&self) -> &'static str {
                super::Dialect::name(self)
            }

            fn get_identifier_quotes(&self) -> Option<(char, char)> {
                super::Dialect::identifier_quotes(self)
            }

            fn escape_string(&self, s: String) -> String {
                super::Dialect::escape_string(self, &s)
            }

            fn quote_string(&self, mut s: String) -> String {
                super::Dialect::quote_string(self, &mut s);
                s
            }

            fn encode_bytes(&self, bytes: Vec<u8>) -> String {
                super::Dialect::encode_bytes(self, &bytes)
            }

            fn encode_bits(&self, bits: &str) -> pyo3::PyResult<String> {
                if !bits.chars().all(|c| c == '0' || c == '1') {
                    return Err(new_py_error!(PyValueError, "invalid input: {}", bits));
                }

                Ok(super::Dialect::encode_bits(self, bits))
            }
        }
    };
}
pub(super) use implement_dialect_pymethods;

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
pub use sqlite::PySqliteDialect;

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
    Sqlite(PySqliteDialect),
    Teradata(PyTeradataDialect),
}

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for DialectUnion {
    type Error = pyo3::PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: pyo3::inspect::PyStaticExpr = type_hint_union!(
        // as object
        type_hint_identifier!("rapidquery._lib.dialect", "BigQueryDialect"),
        type_hint_identifier!("rapidquery._lib.dialect", "DatabricksDialect"),
        type_hint_identifier!("rapidquery._lib.dialect", "DuckDbDialect"),
        type_hint_identifier!("rapidquery._lib.dialect", "HiveDialect"),
        type_hint_identifier!("rapidquery._lib.dialect", "MsSqlDialect"),
        type_hint_identifier!("rapidquery._lib.dialect", "MySqlDialect"),
        type_hint_identifier!("rapidquery._lib.dialect", "OracleDialect"),
        type_hint_identifier!("rapidquery._lib.dialect", "PostgreSqlDialect"),
        type_hint_identifier!("rapidquery._lib.dialect", "RedshiftDialect"),
        type_hint_identifier!("rapidquery._lib.dialect", "SnowflakeDialect"),
        type_hint_identifier!("rapidquery._lib.dialect", "SparkDialect"),
        type_hint_identifier!("rapidquery._lib.dialect", "SqliteDialect"),
        type_hint_identifier!("rapidquery._lib.dialect", "TeradataDialect"),
        // as type
        type_hint_subscript!(
            type_hint_identifier!("type"),
            type_hint_identifier!("rapidquery._lib.dialect", "BigQueryDialect")
        ),
        type_hint_subscript!(
            type_hint_identifier!("type"),
            type_hint_identifier!("rapidquery._lib.dialect", "DatabricksDialect")
        ),
        type_hint_subscript!(
            type_hint_identifier!("type"),
            type_hint_identifier!("rapidquery._lib.dialect", "DuckDbDialect")
        ),
        type_hint_subscript!(
            type_hint_identifier!("type"),
            type_hint_identifier!("rapidquery._lib.dialect", "HiveDialect")
        ),
        type_hint_subscript!(
            type_hint_identifier!("type"),
            type_hint_identifier!("rapidquery._lib.dialect", "MsSqlDialect")
        ),
        type_hint_subscript!(
            type_hint_identifier!("type"),
            type_hint_identifier!("rapidquery._lib.dialect", "MySqlDialect")
        ),
        type_hint_subscript!(
            type_hint_identifier!("type"),
            type_hint_identifier!("rapidquery._lib.dialect", "OracleDialect")
        ),
        type_hint_subscript!(
            type_hint_identifier!("type"),
            type_hint_identifier!("rapidquery._lib.dialect", "PostgreSqlDialect")
        ),
        type_hint_subscript!(
            type_hint_identifier!("type"),
            type_hint_identifier!("rapidquery._lib.dialect", "RedshiftDialect")
        ),
        type_hint_subscript!(
            type_hint_identifier!("type"),
            type_hint_identifier!("rapidquery._lib.dialect", "SnowflakeDialect")
        ),
        type_hint_subscript!(
            type_hint_identifier!("type"),
            type_hint_identifier!("rapidquery._lib.dialect", "SparkDialect")
        ),
        type_hint_subscript!(
            type_hint_identifier!("type"),
            type_hint_identifier!("rapidquery._lib.dialect", "SqliteDialect")
        ),
        type_hint_subscript!(
            type_hint_identifier!("type"),
            type_hint_identifier!("rapidquery._lib.dialect", "TeradataDialect")
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
                return Ok(Self::Sqlite(PySqliteDialect));
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

macro_rules! impl_for_all_fields {
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
}

impl Dialect for DialectUnion {
    fn name(&self) -> &'static str {
        impl_for_all_fields!(self, s => s.name())
    }

    fn bool_format(&self) -> BoolFormat {
        impl_for_all_fields!(self, s => s.bool_format())
    }

    fn identifier_quotes(&self) -> Option<(char, char)> {
        impl_for_all_fields!(self, s => s.identifier_quotes())
    }

    fn escape_string(&self, string: &str) -> String {
        impl_for_all_fields!(self, s => s.escape_string(string))
    }

    fn quote_string(&self, string: &mut String) {
        impl_for_all_fields!(self, s => s.quote_string(string))
    }

    fn encode_bits(&self, bits: &str) -> String {
        impl_for_all_fields!(self, s => s.encode_bits(bits))
    }

    fn encode_bytes(&self, bytes: &[u8]) -> String {
        impl_for_all_fields!(self, s => s.encode_bytes(bytes))
    }
}

#[pyo3::pymodule(name = "dialect")]
pub mod dialect_module {
    #[pymodule_export]
    pub use super::{
        PyBigQueryDialect, PyDatabricksDialect, PyDuckDbDialect, PyHiveDialect, PyMsSqlDialect,
        PyMySqlDialect, PyOracleDialect, PyPostgreSqlDialect, PyRedshiftDialect,
        PySnowflakeDialect, PySparkDialect, PySqliteDialect, PyTeradataDialect,
    };
}
