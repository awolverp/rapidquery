/// Returns the type name of a [`pyo3::ffi::PyObject`].
///
/// Returns `"<unknown>"` on failure.
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
