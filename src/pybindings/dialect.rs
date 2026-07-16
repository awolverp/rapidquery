use crate::internal::utils::get_type_name;
use crate::native;

#[rapidquery_macros::pyclass_shortcut("SqlDialect", subclass, skip_from_py_object)]
#[derive(Debug, Clone)]
pub struct PySqlDialect(pub native::dialect::SqlDialect);

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for PySqlDialect {
    type Error = pyo3::PyErr;

    fn extract(
        obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>,
    ) -> Result<Self, Self::Error> {
        macro_rules! map {
            (
                $ptr:expr,
                $($name:ident => $native:ident,)+
            ) => {
                $(
                    if $ptr == crate::typeref::$name {
                        return Ok(PySqlDialect(native::dialect::SqlDialect::$native));
                    }
                )+
            };
        }

        unsafe {
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

            new_pyerr!(
                Err(PyTypeError),
                "expected a dialect, got {}",
                get_type_name(obj.py(), obj.as_ptr())
            )
        }
    }
}

#[pyo3::pymethods]
impl PySqlDialect {
    #[getter]
    #[inline]
    fn name(&self) -> &'static str {
        self.0.name()
    }
}

macro_rules! impl_dialects {
    ($($native:ident => $name:ident => $pyname:literal,)+) => {
        $(
            #[rapidquery_macros::pyclass_shortcut($pyname, extends=PySqlDialect)]
            pub struct $name;

            #[pyo3::pymethods]
            impl $name {
                #[new]
                #[inline]
                fn __new__() -> pyo3::PyClassInitializer<Self> {
                    pyo3::PyClassInitializer::from(PySqlDialect(native::dialect::SqlDialect::$native))
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
    DuckDb => PyDuckDb => "DuckDb",
    Hive => PyHive => "Hive",
    MsSql => PyMsSql => "MsSql",
    MySql => PyMySql => "MySql",
    Oracle => PyOracle => "Oracle",
    PostgreSql => PyPostgreSql => "PostgreSql",
    Redshift => PyRedshift => "Redshift",
    Snowflake => PySnowflake => "Snowflake",
    Spark => PySpark => "Spark",
    Sqlite => PySqlite => "Sqlite",
    Teradata => PyTeradata => "Teradata",
}
