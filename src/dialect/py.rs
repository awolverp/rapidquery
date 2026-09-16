//! Python class hierarchy around [`super::SqlDialect`].
//!
//! The engine selects a dialect with a `Copy` enum. Python sees a
//! class hierarchy instead:
//!
//! * [`PySqlDialect`] — `rapidquery._core.dialect.SqlDialect`, the ABC
//! * one frozen subclass per variant (`PostgreSQL`, `DuckDB`, …)
//!
//! Extraction ([`FromPyObject`] for [`PySqlDialect`]) accepts **either** the
//! class (`PostgreSQL`) **or** an instance (`PostgreSQL()`). Matching is by
//! interned [`PyTypeObject`] pointer (see [`crate::typeref`]), so only the
//! 13 built-in classes convert — a Python subclass of `SqlDialect` does not.
//!
//! Instances compare and hash by native variant (`PostgreSQL() == PostgreSQL()`)
//! and `repr` as `PostgreSQL()`.

use std::hash::{Hash, Hasher};

use crate::internal::utils::get_type_name;

use super::SqlDialect;

/// Abstract SQL dialect.
///
/// Concrete dialects are frozen subclasses (`PostgreSQL`, `DuckDB`, ...).
/// Pass either the class or an instance wherever a dialect is required.
/// This base class cannot be instantiated.
///
/// Instances of the same dialect compare equal and hash together.
#[rapidquery_macros::pyclass(
    "SqlDialect",
    subclass,
    skip_from_py_object,
    module = "rapidquery._core.dialect"
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PySqlDialect(pub SqlDialect);

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for PySqlDialect {
    type Error = pyo3::PyErr;

    fn extract(
        obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>,
    ) -> Result<Self, Self::Error> {
        // Pointer-identity against the 13 interned type objects. Cheaper than
        // `isinstance` and rejects unknown subclasses by construction.
        macro_rules! map {
            (
                $ptr:expr,
                $($name:ident => $native:ident,)+
            ) => {
                $(
                    if $ptr == crate::typeref::$name {
                        return Ok(PySqlDialect(SqlDialect::$native));
                    }
                )+
            };
        }

        unsafe {
            // SAFETY: `obj` is a borrowed `&Bound<PyAny>`, so `as_ptr()` is a
            // live `PyObject*`. `PyType_CheckExact` is true when the caller
            // passed the class itself (`PostgreSQL`); otherwise we take
            // `Py_TYPE` of an instance (`PostgreSQL()`). Both pointers are
            // interned `PyTypeObject*` values — we only compare addresses
            // against `typeref`, we do not dereference them here.
            // `get_type_name` requires a valid object pointer on the error
            // path, which we have.
            let type_ptr = if pyo3::ffi::PyType_CheckExact(obj.as_ptr()) == 1 {
                obj.as_ptr() as *mut pyo3::ffi::PyTypeObject
            } else {
                pyo3::ffi::Py_TYPE(obj.as_ptr())
            };

            map!(
                type_ptr,
                BIGQUERY_DIALECT_TYPE => BigQuery,
                DATABRICKS_DIALECT_TYPE => Databricks,
                DUCKDB_DIALECT_TYPE => DuckDb,
                HIVE_DIALECT_TYPE => Hive,
                MSSQL_DIALECT_TYPE => MsSql,
                MYSQL_DIALECT_TYPE => MySql,
                ORACLE_DIALECT_TYPE => Oracle,
                POSTGRESQL_DIALECT_TYPE => PostgreSql,
                REDSHIFT_DIALECT_TYPE => Redshift,
                SNOWFLAKE_DIALECT_TYPE => Snowflake,
                SPARK_DIALECT_TYPE => Spark,
                SQLITE_DIALECT_TYPE => Sqlite,
                TERADATA_DIALECT_TYPE => Teradata,
            );

            rapidquery_macros::pyerr!(
                Err(PyTypeError),
                "expected a dialect, got {}",
                get_type_name(obj.py(), obj.as_ptr())
            )
        }
    }
}

#[pyo3::pymethods]
impl PySqlDialect {
    /// Dialect identifier, identical to the Python class name (`"PostgreSQL"`).
    #[getter]
    #[inline]
    fn name(&self) -> &'static str {
        self.0.name()
    }

    fn __repr__(&self) -> String {
        let name = self.0.name();
        let mut buf = String::with_capacity(name.len() + 2);
        buf.push_str(name);
        buf.push_str("()");
        buf
    }

    // Bound<Self> so only instances match; the class object is a type.
    // PostgreSQL() == PostgreSQL is false; PostgreSQL() == PostgreSQL() is true.
    fn __eq__(&self, other: &pyo3::Bound<'_, Self>) -> bool {
        self.0 == other.get().0
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.0.hash(&mut hasher);
        hasher.finish()
    }
}

/// Generate one frozen Python class per native [`SqlDialect`] variant and
/// export them on the `dialect` submodule.
///
/// Each class is an empty `extends=PySqlDialect` subclass. `__new__` stores
/// the matching native variant on the base. `immutable_type` (from the
/// `pyclass` macro) makes the concrete classes non-subclassable from Python.
macro_rules! impl_dialects {
    ($($native:ident => $name:ident => $pyname:literal,)+) => {
        $(
            #[doc = concat!($pyname, " SQL dialect.")]
            #[rapidquery_macros::pyclass(
                $pyname,
                extends = PySqlDialect,
                module = "rapidquery._core.dialect"
            )]
            pub struct $name;

            #[pyo3::pymethods]
            impl $name {
                // Marker instance. Passing the class itself is equivalent
                // at extract time; instances compare equal by variant.
                #[new]
                #[inline]
                fn __new__() -> pyo3::PyClassInitializer<Self> {
                    pyo3::PyClassInitializer::from(PySqlDialect(SqlDialect::$native))
                        .add_subclass(Self)
                }
            }
        )+

        #[pyo3::pymodule(name="dialect", gil_used = false)]
        pub mod dialect_mod {
            #[pymodule_export]
            use super::{
                PySqlDialect,
                $($name,)+
            };
        }
    };
}

impl_dialects! {
    BigQuery => PyBigQuery => "BigQuery",
    Databricks => PyDatabricks => "Databricks",
    DuckDb => PyDuckDb => "DuckDB",
    Hive => PyHive => "Hive",
    MsSql => PyMsSql => "MSSQL",
    MySql => PyMySql => "MySql",
    Oracle => PyOracle => "Oracle",
    PostgreSql => PyPostgreSql => "PostgreSQL",
    Redshift => PyRedshift => "Redshift",
    Snowflake => PySnowflake => "Snowflake",
    Spark => PySpark => "Spark",
    Sqlite => PySqlite => "SQLite",
    Teradata => PyTeradata => "Teradata",
}
