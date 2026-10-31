//! Data Control Language

/// Base of every DCL statement. Construct a subclass, not this class.
#[rapidquery_macros::pyclass("rapidquery.dcl.DataControlLanguage", subclass)]
#[derive(Debug, Clone, Copy)]
pub struct PyDataControlLanguage;

#[pyo3::pymodule(name = "dcl", gil_used = false)]
pub mod dcl_mod {
    #[pymodule_export]
    use super::PyDataControlLanguage;
}
