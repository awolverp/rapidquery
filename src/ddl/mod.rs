//! Data Definition Language

mod schema;
pub use schema::DropSchemaBehavior;
pub use schema::PyCreateSchema;
pub use schema::PyDropSchema;

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
}
