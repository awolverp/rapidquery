//! Transaction Control Language

/// Base of every TCL statement. Construct a subclass, not this class.
#[rapidquery_macros::pyclass("rapidquery.tcl.TransactionControlLanguage", subclass, skip_from_py_object)]
#[derive(Debug, Clone, Copy)]
pub struct PyTransactionControlLanguage;

#[pyo3::pymodule(name = "tcl", gil_used = false)]
pub mod tcl_mod {
    #[pymodule_export]
    use super::PyTransactionControlLanguage;
}
