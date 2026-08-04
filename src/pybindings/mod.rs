pub mod dialect;
pub use dialect::dialect_mod;

pub mod data_type;
pub use data_type::data_type_mod;

use crate::native;

#[rapidquery_macros::pyclass_shortcut("Ident", subclass, skip_from_py_object)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PyIdent(pub native::Ident);

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for PyIdent {
    type Error = pyo3::PyErr;

    fn extract(
        obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>,
    ) -> Result<Self, Self::Error> {
        unsafe {
            if crate::typeref::check(obj.as_ptr(), crate::typeref::IDENT_TYPE) {
                let ident = obj.cast_unchecked::<Self>();
                return Ok(ident.get().clone());
            }
        }

        if let Ok(string) = obj.extract::<String>() {
            return Ok(Self(string.into()));
        }

        new_pyerr!(
            Err(PyTypeError),
            "expected Ident or Asterisk or str, got {}",
            unsafe { crate::internal::utils::get_type_name(obj.py(), obj.as_ptr()) }
        )
    }
}

#[pyo3::pymethods]
impl PyIdent {
    #[new]
    #[inline]
    #[pyo3(signature = (content, /))]
    fn __new__(content: &str) -> Self {
        Self(native::Ident::Content(content.into()))
    }

    // TODO: add similar methods to `str`
}

#[rapidquery_macros::pyclass_shortcut("Asterisk", extends=PyIdent, skip_from_py_object)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PyAsterisk;

#[pyo3::pymethods]
impl PyAsterisk {
    #[new]
    #[inline]
    fn __new__() -> pyo3::PyClassInitializer<Self> {
        pyo3::PyClassInitializer::from(PyIdent(native::Ident::Asterisk)).add_subclass(Self)
    }
}
