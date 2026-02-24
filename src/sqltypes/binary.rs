use crate::sqltypes::abstracts::NativeSQLType;
use crate::sqltypes::abstracts::PySQLTypeAbstract;

implement_pyclass! {
    (
        /// Binary large object column type (BLOB).
        ///
        /// Stores large binary data such as images, documents, audio files, or
        /// any binary content. Size limits vary by database system.
        ///
        /// @extends SQLTypeAbstract[bytes,bytes]
        #[derive(Debug, Clone, Copy)]
        pub struct [extends=PySQLTypeAbstract] PyBlobType as "BlobType";
    )
    (
        /// Fixed-length binary data column type (BINARY).
        ///
        /// Stores binary data of a fixed length. Values shorter than the specified
        /// length are padded. Useful for storing hashes, keys, or other binary
        /// data with consistent length.
        ///
        /// @extends SQLTypeAbstract[bytes,bytes]
        /// @signature (length: int = 255)
        #[derive(Debug, Clone, Copy)]
        pub struct [extends=PySQLTypeAbstract] PyBinaryType as "BinaryType" (pub u32);
    )
    (
        /// Variable-length binary data column type (VARBINARY).
        ///
        /// Stores binary data of variable length up to a specified maximum.
        /// More storage-efficient than BINARY for binary data of varying lengths.
        ///
        /// @extends SQLTypeAbstract[bytes,bytes]
        #[derive(Debug, Clone, Copy)]
        pub struct [extends=PySQLTypeAbstract] PyVarBinaryType as "VarBinaryType" (pub Option<u32>);
    )
    (
        /// Fixed-length bit string column type (BIT).
        ///
        /// Stores a fixed number of bits. Useful for storing boolean flags efficiently
        /// or binary data where individual bits have meaning.
        ///
        /// @extends SQLTypeAbstract[bytes,bytes]
        /// @signature (length: int)
        #[derive(Debug, Clone, Copy)]
        pub struct [extends=PySQLTypeAbstract] PyBitType as "BitType" (pub Option<u32>);
    )
    (
        /// Variable-length bit string column type (VARBIT).
        ///
        /// Stores a variable number of bits up to a specified maximum. More flexible
        /// than fixed BIT type for bit strings of varying lengths.
        ///
        /// @extends SQLTypeAbstract[bytes,bytes]
        #[derive(Debug, Clone, Copy)]
        pub struct [extends=PySQLTypeAbstract] PyVarBitType as "VarBitType" (pub u32);
    )
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

impl NativeSQLType for PyBlobType {
    #[inline(always)]
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::Blob
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        if pyo3::ffi::PyBytes_CheckExact(ptr) != 0 {
            Err(typeerror!("expected bytes, got {:?}", py, ptr))
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
            _ => invalid_value_for_deserialize!("bytes", value),
        }
    }
}

impl NativeSQLType for PyBinaryType {
    #[inline(always)]
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::Binary(self.0)
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        if pyo3::ffi::PyBytes_CheckExact(ptr) != 0 {
            Err(typeerror!("expected bytes, got {:?}", py, ptr))
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
            _ => invalid_value_for_deserialize!("bytes", value),
        }
    }
}

impl NativeSQLType for PyVarBinaryType {
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
        if pyo3::ffi::PyBytes_CheckExact(ptr) != 0 {
            Err(typeerror!("expected bytes, got {:?}", py, ptr))
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
            _ => invalid_value_for_deserialize!("bytes", value),
        }
    }
}

impl NativeSQLType for PyBitType {
    #[inline(always)]
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::Bit(self.0)
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        if pyo3::ffi::PyBytes_CheckExact(ptr) != 0 {
            Err(typeerror!("expected bytes, got {:?}", py, ptr))
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
            _ => invalid_value_for_deserialize!("bytes", value),
        }
    }
}

impl NativeSQLType for PyVarBitType {
    #[inline(always)]
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::VarBit(self.0)
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        if pyo3::ffi::PyBytes_CheckExact(ptr) != 0 {
            Err(typeerror!("expected bytes, got {:?}", py, ptr))
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
            _ => invalid_value_for_deserialize!("bytes", value),
        }
    }
}

super::abstracts::implement_native_pymethods!(PyBlobType);
super::abstracts::implement_native_pymethods!(
    PyBinaryType,
    init(|length: u32| Self(length)),
    "int",
    signature(length = 255)
);
super::abstracts::implement_native_pymethods!(
    PyVarBinaryType,
    init(|length: Option<u32>| Self(length)),
    "int",
    signature(length = None)
);
super::abstracts::implement_native_pymethods!(
    PyBitType,
    init(|length: Option<u32>| Self(length)),
    "int",
    signature(length = None)
);
super::abstracts::implement_native_pymethods!(
    PyVarBitType,
    init(|length: u32| Self(length)),
    "int",
    signature(length = 255)
);
