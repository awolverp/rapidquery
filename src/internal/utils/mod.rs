mod strings;
mod submodule_ext;

pub use strings::*;
pub use submodule_ext::*;

/// Borrow the cached UTF-8 buffer of a Python `str`.
///
/// # Safety
///
/// `ptr` must be a live unicode object (`PyUnicode_Check` / exact). The
/// returned slice is valid until that object is mutated or the GIL is
/// released. The GIL must be held (`py`).
#[inline]
pub unsafe fn borrow_py_str_buffer<'a>(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
) -> pyo3::PyResult<&'a str> {
    let mut size: pyo3::ffi::Py_ssize_t = 0;
    let data = pyo3::ffi::PyUnicode_AsUTF8AndSize(ptr, &mut size);
    if data.is_null() || size < 0 {
        Err(pyo3::PyErr::fetch(py))
    } else {
        // SAFETY: `PyUnicode_AsUTF8AndSize` returns UTF-8; `size` is the byte
        // length. The pointer is non-null on the success path.
        Ok(std::str::from_utf8_unchecked(std::slice::from_raw_parts(
            data as *const u8,
            size as usize,
        )))
    }
}

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

/// Gets the Python builtin value `None` as [`pyo3::Bound<'_, pyo3::PyAny>`].
#[inline]
pub fn new_pynone<'py>(py: pyo3::Python<'py>) -> pyo3::Bound<'py, pyo3::PyAny> {
    unsafe { pyo3::Bound::from_borrowed_ptr(py, pyo3::ffi::Py_None()) }
}

/// Helper enum to accept either an `isize` index or a `slice` in `__getitem__`.
#[derive(Debug, pyo3::FromPyObject)]
pub enum IndexOrSlice {
    Index(isize),
    Slice(pyo3::Py<pyo3::types::PySlice>),
}
