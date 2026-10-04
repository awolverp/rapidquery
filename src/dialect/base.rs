use crate::internal::utils;

use super::flags;

/// SQL dialect.
///
/// Instances of the same dialect compare equal and hash together.
#[rapidquery_macros::pyclass("rapidquery.dialect.Dialect", skip_from_py_object)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyDialect(pub flags::DialectFlags);

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for PyDialect {
    type Error = pyo3::PyErr;

    fn extract(obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>) -> Result<Self, Self::Error> {
        unsafe {
            // SAFETY: `obj` is a borrowed `&Bound<PyAny>`, so `as_ptr()` is a
            // live `PyObject*`.
            if crate::typeref::check_exact(obj.as_ptr(), crate::typeref::DIALECT_TYPE) {
                let cast = obj.cast_unchecked::<Self>();
                return Ok(Self(cast.get().0));
            }

            rapidquery_macros::pyerr!(
                Err(PyTypeError),
                "expected Dialect, got {}",
                utils::get_type_name(obj.py(), obj.as_ptr())
            )
        }
    }
}

#[pyo3::pymethods]
impl PyDialect {
    fn __eq__(&self, other: &pyo3::Bound<'_, Self>) -> bool {
        self.0 == other.get().0
    }

    fn __hash__(&self) -> u64 {
        use std::hash::Hash;
        use std::hash::Hasher;

        let mut hasher = std::hash::DefaultHasher::default();
        self.0.hash(&mut hasher);
        hasher.finish()
    }
}
