//! Data Definition Language

mod schema;
pub use schema::DropSchemaBehavior;
pub use schema::PyCreateSchema;
pub use schema::PyDropSchema;

mod index;
pub use index::IndexColumnExtractor;
pub use index::PyCreateIndex;
pub use index::PyDropIndex;
pub use index::PyIndexColumn;

/// Base of every DDL statement. Construct a subclass, not this class.
#[rapidquery_macros::pyclass("rapidquery.ddl.DataDefinitionLanguage", subclass)]
#[derive(Debug, Clone, Copy)]
pub struct PyDataDefinitionLanguage;

#[pyo3::pymodule(name = "ddl", gil_used = false)]
pub mod ddl_mod {
    #[pymodule_export]
    use super::PyDataDefinitionLanguage;

    #[pymodule_export]
    use super::PyCreateSchema;
    #[pymodule_export]
    use super::PyDropSchema;

    #[pymodule_export]
    use super::PyCreateIndex;
    #[pymodule_export]
    use super::PyDropIndex;
    #[pymodule_export]
    use super::PyIndexColumn;
}
