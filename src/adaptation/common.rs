use pyo3::types::PyAnyMethods;

/// Import json module only once
#[inline]
pub fn import_json_module(py: pyo3::Python<'_>) -> pyo3::PyResult<&pyo3::Bound<'_, pyo3::types::PyModule>> {
    static JSON_CLS: once_cell::sync::OnceCell<pyo3::Py<pyo3::types::PyModule>> =
        once_cell::sync::OnceCell::new();

    let json = JSON_CLS.get_or_try_init(|| py.import("json").map(|x| x.unbind()));
    json.map(|x| x.bind(py))
}

/// Serialize pyobject with Python `json` module
/// 
/// Note: `ptr` should be borrowed
#[inline]
pub fn _serialize_object_with_pyjson(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
    let json = import_json_module(py)?;
    let dumps_func = json.getattr("dumps")?;

    unsafe {
        dumps_func
            .call1((pyo3::Py::<pyo3::PyAny>::from_borrowed_ptr(py, ptr),))
            .map(|x| x.into_ptr())
    }
}

/// Deserialize pyobject with Python `json` module
/// 
/// Note: `ptr` should be borrowed
#[inline]
pub fn _deserialize_object_with_pyjson(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
    let json = import_json_module(py)?;
    let loads_func = json.getattr("loads")?;

    unsafe {
        loads_func
            .call1((pyo3::Py::<pyo3::PyAny>::from_borrowed_ptr(py, ptr),))
            .map(|x| x.into_ptr())
    }
}

/// Try to serialize pyobject to validate pyobject is JSON-serializable
#[inline]
pub fn _validate_json_object(py: pyo3::Python<'_>, ptr: *mut pyo3::ffi::PyObject) -> pyo3::PyResult<()> {
    unsafe {
        // Fast path
        if (pyo3::ffi::PyLong_CheckExact(ptr) == 1)
            || (pyo3::ffi::PyUnicode_CheckExact(ptr) == 1)
            || (pyo3::ffi::PyFloat_CheckExact(ptr) == 1)
            || (pyo3::ffi::Py_IsNone(ptr) == 1)
        {
            return Ok(());
        }
    }

    _serialize_object_with_pyjson(py, ptr)?;
    Ok(())
}

/// Import decimal module only once
#[inline]
pub fn import_decimal_module(py: pyo3::Python<'_>) -> pyo3::PyResult<&pyo3::Bound<'_, pyo3::types::PyModule>> {
    static DECIMAL_CLS: once_cell::sync::OnceCell<pyo3::Py<pyo3::types::PyModule>> =
        once_cell::sync::OnceCell::new();

    let decimal = DECIMAL_CLS.get_or_try_init(|| py.import("decimal").map(|x| x.unbind()));
    decimal.map(|x| x.bind(py))
}

/// Call `decimal.Decimal(ptr)`
/// 
/// Note: `ptr` should be owned
#[inline]
#[allow(non_snake_case)]
pub fn _new_decimal_Decimal(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
    let decimal = import_decimal_module(py)?;
    let decimal_class = decimal.getattr("Decimal")?;

    unsafe {
        decimal_class
            .call1((pyo3::Py::<pyo3::PyAny>::from_owned_ptr(py, ptr),))
            .map(|x| x.into_ptr())
    }
}
