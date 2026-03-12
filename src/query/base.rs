use crate::internal::{BoundArgs, BoundKwargs, BoundObject};

crate::implement_pyclass! {
    /// Subclass of query statements.
    ///
    /// @alias _BackendName = typing.Literal["sqlite", "postgresql", "postgres", "mysql"]
    /// @signature (self)
    #[derive(Debug, Clone, Copy)]
    [subclass] PyQueryStatement as "QueryStatement";
}

#[pyo3::pymethods]
impl PyQueryStatement {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(args: BoundArgs<'_>, kwds: Option<BoundKwargs<'_>>) -> Self {
        Self
    }

    fn __init__(&self) {}

    /// Build a SQL string representation.
    ///
    /// **This method is unsafe and can cause SQL injection.** use `.build()` method instead.
    ///
    /// @signature (self, backend: _BackendName, /) -> str
    #[pyo3(signature = (backend, /))]
    #[allow(unused_variables)]
    #[allow(clippy::wrong_self_convention)]
    fn to_sql(&self, py: pyo3::Python<'_>, backend: String) -> pyo3::PyResult<String> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(()))
    }

    /// Build the SQL statement with parameter values.
    ///
    /// @signature (self, backend: _BackendName, /) -> tuple[str, tuple[Value, ...]]
    #[pyo3(signature = (backend, /))]
    #[allow(unused_variables)]
    fn build<'a>(
        &self,
        py: pyo3::Python<'a>,
        backend: String,
    ) -> pyo3::PyResult<(String, BoundObject<'a>)> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(()))
    }
}
