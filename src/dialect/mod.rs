mod base;
mod builtin;
mod convert;
mod generic;
pub mod inner;

pub use base::PySqlDialect;
pub use builtin::{
    PyBigQuery, PyDatabricks, PyDuckDb, PyHive, PyMsSql, PyMySql, PyOracle, PyPostgreSql,
    PyRedshift, PySnowflake, PySpark, PySqlite, PyTeradata,
};
pub use generic::PyGenericDialect;

#[pyo3::pymodule(name = "dialect", gil_used = false)]
pub mod dialect_mod {
    #[pymodule_export]
    use super::{
        PyBigQuery, PyDatabricks, PyDuckDb, PyGenericDialect, PyHive, PyMsSql, PyMySql, PyOracle,
        PyPostgreSql, PyRedshift, PySnowflake, PySpark, PySqlDialect, PySqlite, PyTeradata,
    };
}
