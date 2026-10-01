use std::hash::Hash;
use std::hash::Hasher;

use crate::internal::utils;

use super::inner::Dialect;

/// Abstract SQL dialect.
///
/// Concrete dialects are frozen subclasses (`PostgreSql`, `DuckDb`, ...).
/// Pass an instance wherever a dialect is required.
/// This base class cannot be instantiated.
///
/// Instances of the same dialect compare equal and hash together.
#[rapidquery_macros::pyclass("rapidquery.dialect.SqlDialect", subclass, skip_from_py_object)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PySqlDialect(pub Dialect);

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for PySqlDialect {
    type Error = pyo3::PyErr;

    fn extract(
        obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>,
    ) -> Result<Self, Self::Error> {
        unsafe {
            // SAFETY: `obj` is a borrowed `&Bound<PyAny>`, so `as_ptr()` is a
            // live `PyObject*`.
            if crate::typeref::check(obj.as_ptr(), crate::typeref::SQL_DIALECT_TYPE) {
                let cast = obj.cast_unchecked::<Self>();
                return Ok(Self(cast.get().0));
            }

            rapidquery_macros::pyerr!(
                Err(PyTypeError),
                "expected a SqlDialect, got {}",
                utils::get_type_name(obj.py(), obj.as_ptr())
            )
        }
    }
}

#[pyo3::pymethods]
impl PySqlDialect {
    // Bound<Self> so only instances match; the class object is a type.
    // PostgreSql() == PostgreSql is false; PostgreSql() == PostgreSql() is true.
    fn __eq__(&self, other: &pyo3::Bound<'_, Self>) -> bool {
        self.0 == other.get().0
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = std::hash::DefaultHasher::default();
        self.0.hash(&mut hasher);
        hasher.finish()
    }
}
