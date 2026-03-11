crate::implement_pyclass! {
    /// Subclass of schema statements.
    ///
    /// @alias _BackendName = typing.Literal["sqlite", "postgresql", "postgres", "mysql"]
    /// @signature (self)
    #[derive(Debug, Clone, Copy)]
    [subclass] PySchemaStatement as "SchemaStatement";
}

#[pyo3::pymethods]
impl PySchemaStatement {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(
        args: &pyo3::Bound<'_, pyo3::types::PyTuple>,
        kwds: Option<&pyo3::Bound<'_, pyo3::types::PyDict>>,
    ) -> Self {
        Self
    }

    fn __init__(&self) {}

    /// Build a SQL string representation.
    ///
    /// @signature (self, backend: _BackendName, /) -> str
    #[pyo3(signature = (backend, /))]
    #[allow(unused_variables)]
    #[allow(clippy::wrong_self_convention)]
    fn to_sql(&self, backend: String) -> pyo3::PyResult<String> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(()))
    }
}
