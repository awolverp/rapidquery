use crate::sqltypes::abstracts::{PySQLTypeAbstract, SQLTypeTrait};

crate::implement_pyclass! {
    /// Binary large object column type (BLOB).
    ///
    /// Stores large binary data such as images, documents, audio files, or
    /// any binary content. Size limits vary by database system.
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyBlobType as "Blob";
}
crate::implement_pyclass! {
    /// Fixed-length binary data column type (BINARY).
    ///
    /// Stores binary data of a fixed length. Values shorter than the specified
    /// length are padded. Useful for storing hashes, keys, or other binary
    /// data with consistent length.
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyBinaryType as "Binary" (pub u32);
}
crate::implement_pyclass! {
    /// Variable-length binary data column type (VARBINARY).
    ///
    /// Stores binary data of variable length up to a specified maximum.
    /// More storage-efficient than BINARY for binary data of varying lengths.
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyVarBinaryType as "VarBinary" (pub Option<u32>);
}
crate::implement_pyclass! {
    /// Fixed-length bit string column type (BIT).
    ///
    /// Stores a fixed number of bits. Useful for storing boolean flags efficiently
    /// or binary data where individual bits have meaning.
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyBitType as "Bit" (pub Option<u32>);
}
crate::implement_pyclass! {
    /// Variable-length bit string column type (VARBIT).
    ///
    /// Stores a variable number of bits up to a specified maximum. More flexible
    /// than fixed BIT type for bit strings of varying lengths.
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyVarBitType as "VarBit" (pub u32);

}

#[inline]
unsafe fn _serialize_function(
    object: *mut pyo3::ffi::PyObject,
) -> pyo3::PyResult<sea_query::Value> {
    let buffer = pyo3::ffi::PyBytes_AsString(object) as *const u8;
    let size = pyo3::ffi::PyBytes_Size(object) as usize;

    debug_assert!(!buffer.is_null());

    let val = std::slice::from_raw_parts(buffer, size);

    Ok(sea_query::Value::Bytes(Some(Box::new(val.to_vec()))))
}

#[inline]
unsafe fn _deserialize_function(
    py: pyo3::Python,
    value: &[u8],
) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
    let pyptr = pyo3::ffi::PyBytes_FromStringAndSize(std::ptr::null(), value.len() as isize);

    if pyptr.is_null() {
        return Err(pyo3::PyErr::fetch(py));
    }

    let buffer = pyo3::ffi::PyBytes_AsString(pyptr) as *mut u8;
    debug_assert!(!buffer.is_null());

    let mutable = std::slice::from_raw_parts_mut(buffer, value.len());
    mutable.copy_from_slice(value);

    Ok(pyptr)
}

impl SQLTypeTrait for PyBlobType {
    #[inline(always)]
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::Blob
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        if pyo3::ffi::PyBytes_CheckExact(ptr) != 1 {
            crate::new_error!(
                PyTypeError,
                "expected bytes for {} serialization, got {}",
                self.to_sql_type_name(),
                crate::internal::get_type_name(py, ptr)
            )
        } else {
            Ok(())
        }
    }

    unsafe fn serialize(
        &self,
        _py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<sea_query::Value> {
        _serialize_function(ptr)
    }

    unsafe fn deserialize(
        &self,
        py: pyo3::Python,
        value: &sea_query::Value,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        match value {
            sea_query::Value::Bytes(Some(x)) => _deserialize_function(py, x),
            sea_query::Value::Bytes(None) => Ok(pyo3::ffi::Py_None()),
            _ => crate::new_error!(
                PyTypeError,
                "expected bytes for {} deserialization, got {:?}",
                self.to_sql_type_name(),
                value
            ),
        }
    }
}

impl SQLTypeTrait for PyBinaryType {
    #[inline(always)]
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::Binary(self.0)
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        if pyo3::ffi::PyBytes_CheckExact(ptr) != 1 {
            crate::new_error!(
                PyTypeError,
                "expected bytes for {} serialization, got {}",
                self.to_sql_type_name(),
                crate::internal::get_type_name(py, ptr)
            )
        } else {
            Ok(())
        }
    }

    unsafe fn serialize(
        &self,
        _py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<sea_query::Value> {
        _serialize_function(ptr)
    }

    unsafe fn deserialize(
        &self,
        py: pyo3::Python,
        value: &sea_query::Value,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        match value {
            sea_query::Value::Bytes(Some(x)) => _deserialize_function(py, x),
            sea_query::Value::Bytes(None) => Ok(pyo3::ffi::Py_None()),
            _ => crate::new_error!(
                PyTypeError,
                "expected bytes for {} deserialization, got {:?}",
                self.to_sql_type_name(),
                value
            ),
        }
    }
}

impl SQLTypeTrait for PyVarBinaryType {
    #[inline(always)]
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::VarBinary(
            self.0
                .map_or(sea_query::StringLen::None, sea_query::StringLen::N),
        )
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        if pyo3::ffi::PyBytes_CheckExact(ptr) != 1 {
            crate::new_error!(
                PyTypeError,
                "expected bytes for {} serialization, got {}",
                self.to_sql_type_name(),
                crate::internal::get_type_name(py, ptr)
            )
        } else {
            Ok(())
        }
    }

    unsafe fn serialize(
        &self,
        _py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<sea_query::Value> {
        _serialize_function(ptr)
    }

    unsafe fn deserialize(
        &self,
        py: pyo3::Python,
        value: &sea_query::Value,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        match value {
            sea_query::Value::Bytes(Some(x)) => _deserialize_function(py, x),
            sea_query::Value::Bytes(None) => Ok(pyo3::ffi::Py_None()),
            _ => crate::new_error!(
                PyTypeError,
                "expected bytes for {} deserialization, got {:?}",
                self.to_sql_type_name(),
                value
            ),
        }
    }
}

impl SQLTypeTrait for PyBitType {
    #[inline(always)]
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::Bit(self.0)
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        if pyo3::ffi::PyBytes_CheckExact(ptr) != 1 {
            crate::new_error!(
                PyTypeError,
                "expected bytes for {} serialization, got {}",
                self.to_sql_type_name(),
                crate::internal::get_type_name(py, ptr)
            )
        } else {
            Ok(())
        }
    }

    unsafe fn serialize(
        &self,
        _py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<sea_query::Value> {
        _serialize_function(ptr)
    }

    unsafe fn deserialize(
        &self,
        py: pyo3::Python,
        value: &sea_query::Value,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        match value {
            sea_query::Value::Bytes(Some(x)) => _deserialize_function(py, x),
            sea_query::Value::Bytes(None) => Ok(pyo3::ffi::Py_None()),
            _ => crate::new_error!(
                PyTypeError,
                "expected bytes for {} deserialization, got {:?}",
                self.to_sql_type_name(),
                value
            ),
        }
    }
}

impl SQLTypeTrait for PyVarBitType {
    #[inline(always)]
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::VarBit(self.0)
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        if pyo3::ffi::PyBytes_CheckExact(ptr) != 1 {
            crate::new_error!(
                PyTypeError,
                "expected bytes for {} serialization, got {}",
                self.to_sql_type_name(),
                crate::internal::get_type_name(py, ptr)
            )
        } else {
            Ok(())
        }
    }

    unsafe fn serialize(
        &self,
        _py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<sea_query::Value> {
        _serialize_function(ptr)
    }

    unsafe fn deserialize(
        &self,
        py: pyo3::Python,
        value: &sea_query::Value,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        match value {
            sea_query::Value::Bytes(Some(x)) => _deserialize_function(py, x),
            sea_query::Value::Bytes(None) => Ok(pyo3::ffi::Py_None()),
            _ => crate::new_error!(
                PyTypeError,
                "expected bytes for {} deserialization, got {:?}",
                self.to_sql_type_name(),
                value
            ),
        }
    }
}

super::abstracts::implement_sqltype_pymethods!(PyBlobType);
super::abstracts::implement_sqltype_pymethods!(
    PyBinaryType,
    init(|length: u32| Self(length)),
    "int",
    signature(length = 255)
);
super::abstracts::implement_sqltype_pymethods!(
    PyVarBinaryType,
    init(|length: Option<u32>| Self(length)),
    "int",
    signature(length = None)
);
super::abstracts::implement_sqltype_pymethods!(
    PyBitType,
    init(|length: Option<u32>| Self(length)),
    "int",
    signature(length = None)
);
super::abstracts::implement_sqltype_pymethods!(
    PyVarBitType,
    init(|length: u32| Self(length)),
    "int",
    signature(length = 255)
);
