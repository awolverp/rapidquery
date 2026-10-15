//! Data Definition Language

/// Base of every DDL statement. Construct a subclass, not this class.
#[rapidquery_macros::pyclass("rapidquery.ddl.DataDefinitionLanguage", subclass, skip_from_py_object)]
#[derive(Debug, Clone, Copy)]
pub struct PyDataDefinitionLanguage;

#[pyo3::pymodule(name = "ddl", gil_used = false)]
pub mod ddl_mod {
    #[pymodule_export]
    use super::PyDataDefinitionLanguage;
}
