use crate::sqltypes::abstracts::PySQLTypeAbstract;
use crate::sqltypes::abstracts::SQLTypeTrait;

use pyo3::types::PyAnyMethods;

crate::implement_pyclass! {
    /// JSON data column type (JSON).
    ///
    /// Stores JSON documents with validation and indexing capabilities.
    /// Allows for flexible schema design and complex nested data structures
    /// while maintaining some query capabilities.
    ///
    /// @extends SQLTypeAbstract[typing.Any]
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyJSONType as "JSON";
}
crate::implement_pyclass! {
    /// Binary JSON column type (JSONB).
    ///
    /// Stores JSON documents in a binary format for improved performance.
    /// Provides faster query and manipulation operations compared to text-based
    /// JSON storage, with additional indexing capabilities.
    ///
    /// @extends SQLTypeAbstract[typing.Any]
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyJSONBinaryType as "JSONBinary";
}

/// Import json module only once
#[inline(always)]
pub fn import_json_module(
    py: pyo3::Python<'_>,
) -> pyo3::PyResult<&pyo3::Bound<'_, pyo3::types::PyModule>> {
    static JSON_CLS: std::sync::OnceLock<pyo3::Py<pyo3::types::PyModule>> =
        std::sync::OnceLock::new();

    let json = JSON_CLS.get_or_try_init(|| py.import("json").map(|x| x.unbind()));
    json.map(|x| x.bind(py))
}

/// Serialize pyobject with Python `json` module
///
/// Note: `ptr` should be borrowed
#[inline(always)]
pub fn _serialize_object_with_pyjson(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
    let json = import_json_module(py)?;
    let dumps_func = json.getattr("dumps")?;

    unsafe {
        let arg1 = pyo3::Bound::from_borrowed_ptr(py, ptr);
        dumps_func.call1((arg1,)).map(|x| x.into_ptr())
    }
}

/// Deserialize pyobject with Python `json` module
///
/// Note: `ptr` should be borrowed
#[inline(always)]
pub fn _deserialize_object_with_pyjson(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
    let json = import_json_module(py)?;
    let loads_func = json.getattr("loads")?;

    unsafe {
        let arg1 = pyo3::Bound::from_borrowed_ptr(py, ptr);
        loads_func.call1((arg1,)).map(|x| x.into_ptr())
    }
}

/// Try to serialize pyobject to validate pyobject is JSON-serializable
#[inline]
#[cfg_attr(feature = "optimize", optimize(speed))]
pub fn _validate_json_object(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
) -> pyo3::PyResult<()> {
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

#[inline]
#[cfg_attr(feature = "optimize", optimize(speed))]
unsafe fn _serialize_function(
    py: pyo3::Python,
    ptr: *mut pyo3::ffi::PyObject,
) -> pyo3::PyResult<sea_query::Value> {
    let serialized = _serialize_object_with_pyjson(py, ptr)?;

    let mut size: pyo3::ffi::Py_ssize_t = 0;
    let c_str = pyo3::ffi::PyUnicode_AsUTF8AndSize(serialized, &mut size);

    if c_str.is_null() || size < 0 {
        pyo3::ffi::Py_DECREF(serialized);
        Err(pyo3::PyErr::fetch(py))
    } else {
        let val = std::ffi::CStr::from_ptr(c_str);
        let val = serde_json::from_slice::<serde_json::Value>(val.to_bytes());

        pyo3::ffi::Py_DECREF(serialized);

        let val =
            val.map_err(|x| pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(x.to_string()))?;

        Ok(sea_query::Value::Json(Some(Box::new(val))))
    }
}

#[inline]
#[cfg_attr(feature = "optimize", optimize(speed))]
unsafe fn _deserialize_function(
    py: pyo3::Python,
    value: &serde_json::Value,
) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
    let encoded = serde_json::to_vec(value)
        .map_err(|x| pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(x.to_string()))?;

    let val = pyo3::types::PyString::intern(py, std::str::from_utf8_unchecked(&encoded));

    let val = _deserialize_object_with_pyjson(py, val.as_ptr())?;
    Ok(val)
}

impl SQLTypeTrait for PyJSONType {
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::Json
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        _validate_json_object(py, ptr)
    }

    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<sea_query::Value> {
        _serialize_function(py, ptr)
    }

    unsafe fn deserialize(
        &self,
        py: pyo3::Python,
        value: &sea_query::Value,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        match value {
            sea_query::Value::Json(Some(x)) => _deserialize_function(py, x),
            sea_query::Value::Json(None) => Ok(pyo3::ffi::Py_None()),
            _ => crate::new_error!(
                PyTypeError,
                "expected json for {} deserialization, got {:?}",
                self.to_sql_type_name(),
                value
            ),
        }
    }
}

impl SQLTypeTrait for PyJSONBinaryType {
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::JsonBinary
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        _validate_json_object(py, ptr)
    }

    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<sea_query::Value> {
        _serialize_function(py, ptr)
    }

    unsafe fn deserialize(
        &self,
        py: pyo3::Python,
        value: &sea_query::Value,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        match value {
            sea_query::Value::Json(Some(x)) => _deserialize_function(py, x),
            sea_query::Value::Json(None) => Ok(pyo3::ffi::Py_None()),
            _ => crate::new_error!(
                PyTypeError,
                "expected json for {} deserialization, got {:?}",
                self.to_sql_type_name(),
                value
            ),
        }
    }
}

super::abstracts::implement_sqltype_pymethods!(PyJSONType);
super::abstracts::implement_sqltype_pymethods!(PyJSONBinaryType);
