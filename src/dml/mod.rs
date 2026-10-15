//! Data Manipulation Language

/// Base of every DML statement. Construct a subclass, not this class.
#[rapidquery_macros::pyclass("rapidquery.dml.DataManipulationLanguage", subclass, skip_from_py_object)]
#[derive(Debug, Clone, Copy)]
pub struct PyDataManipulationLanguage;

#[pyo3::pymodule(name = "dml", gil_used = false)]
pub mod dml_mod {
    #[pymodule_export]
    use super::PyDataManipulationLanguage;
}
