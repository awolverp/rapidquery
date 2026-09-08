mod strings;
mod submodule_ext;

pub use strings::*;
pub use submodule_ext::*;

/// Fully-qualified type name of `obj` (`int`, `rapidquery._core.dialect.PostgreSQL`, …).
///
/// Returns `"<unknown>"` if `Py_TYPE` is null or the name lookup fails.
///
/// # Safety
///
/// `obj` must be a valid, non-null `PyObject*` (a borrowed `Bound<PyAny>`
/// pointer is fine). The GIL must be held (`py`).
#[inline]
pub unsafe fn get_type_name<'a>(py: pyo3::Python<'a>, obj: *mut pyo3::ffi::PyObject) -> String {
    use pyo3::types::{PyStringMethods, PyTypeMethods};

    let type_ = pyo3::ffi::Py_TYPE(obj);

    if type_.is_null() {
        String::from("<unknown>")
    } else {
        let obj = pyo3::types::PyType::from_borrowed_type_ptr(py, type_);

        obj.fully_qualified_name()
            .map(|x| x.to_string_lossy().into_owned())
            .unwrap_or_else(|_| String::from("<unknown>"))
    }
}

/// Helper enum to accept either an `isize` index or a `slice` in `__getitem__`.
#[derive(Debug, pyo3::FromPyObject)]
pub enum IndexOrSlice {
    Index(isize),
    Slice(pyo3::Py<pyo3::types::PySlice>),
}
