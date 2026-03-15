use crate::internal::{BoundArgs, BoundKwargs};

crate::implement_pyclass! {
    /// Subclass of schema statements.
    #[derive(Debug, Clone, Copy)]
    [subclass] PySchemaStatement as "SchemaStatement";
}

#[pyo3::pymethods]
impl PySchemaStatement {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(args: BoundArgs<'_>, kwds: Option<BoundKwargs<'_>>) -> Self {
        Self
    }

    fn __init__(&self) {}

    /// Build a SQL string representation.
    #[pyo3(signature = (backend, /))]
    #[allow(unused_variables)]
    #[allow(clippy::wrong_self_convention)]
    fn to_sql(&self, backend: String) -> pyo3::PyResult<String> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(()))
    }
}
