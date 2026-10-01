mod base;
mod builtin;
pub mod inner;

// TODO: rewrite GenericDialect again

pub use base::PySqlDialect;
pub use builtin::PyBigQuery;
pub use builtin::PyDatabricks;
pub use builtin::PyDuckDb;
pub use builtin::PyHive;
pub use builtin::PyMsSql;
pub use builtin::PyMySql;
pub use builtin::PyOracle;
pub use builtin::PyPostgreSql;
pub use builtin::PyRedshift;
pub use builtin::PySnowflake;
pub use builtin::PySpark;
pub use builtin::PySqlite;
pub use builtin::PyTeradata;

#[pyo3::pymodule(name = "dialect", gil_used = false)]
pub mod dialect_mod {
    #[pymodule_export]
    use super::PyBigQuery;
    #[pymodule_export]
    use super::PyDatabricks;
    #[pymodule_export]
    use super::PyDuckDb;
    #[pymodule_export]
    use super::PyHive;
    #[pymodule_export]
    use super::PyMsSql;
    #[pymodule_export]
    use super::PyMySql;
    #[pymodule_export]
    use super::PyOracle;
    #[pymodule_export]
    use super::PyPostgreSql;
    #[pymodule_export]
    use super::PyRedshift;
    #[pymodule_export]
    use super::PySnowflake;
    #[pymodule_export]
    use super::PySpark;
    #[pymodule_export]
    use super::PySqlDialect;
    #[pymodule_export]
    use super::PySqlite;
    #[pymodule_export]
    use super::PyTeradata;
}
