//! All SQL types with their serialization and deserialization methods.
//!
//! With support for custom types for Python.

mod abstracts;
mod binary;
mod datetimes;
mod json;
mod others;
mod primitives;
mod vector;

pub use abstracts::*;
pub use binary::*;
pub use datetimes::*;
pub use json::*;
pub use others::*;
pub use primitives::*;
pub use vector::*;

/// Type engine is an enum which can control validations, serializations,
/// and deserializations of column types.
///
/// [`NativeSQLType`] is implemented for this type.
pub struct TypeEngine(
    std::sync::Arc<dyn NativeSQLType + Send + Sync>,
    std::ptr::NonNull<pyo3::ffi::PyObject>,
);

/// SAFETY: PyObjects are safe between threads.
unsafe impl Send for TypeEngine {}
unsafe impl Sync for TypeEngine {}

impl Clone for TypeEngine {
    fn clone(&self) -> Self {
        unsafe {
            pyo3::ffi::Py_INCREF(self.1.as_ptr());
        }

        Self(self.0.clone(), self.1)
    }
}

impl Drop for TypeEngine {
    fn drop(&mut self) {
        unsafe {
            pyo3::ffi::Py_DECREF(self.1.as_ptr());
        }
    }
}

macro_rules! wrap_typeengine {
    ($result:expr, $ptr:expr) => {
        unsafe {
            pyo3::ffi::Py_INCREF($ptr);
            Self(
                std::sync::Arc::new($result),
                std::ptr::NonNull::new_unchecked($ptr),
            )
        }
    };

    ($py:expr, $result:expr, $initializer:expr) => {
        unsafe {
            let object_ptr = pyo3::Py::new($py, ($initializer, PySQLTypeAbstract))
                .unwrap()
                .into_ptr();

            Self(
                std::sync::Arc::new($result),
                std::ptr::NonNull::new_unchecked(object_ptr),
            )
        }
    };
}

macro_rules! check_native_column_types {
    (
        $object:expr,
        $type_ptr:expr,
        $($type:ty => $constant:ident,)*
    ) => {
        $(
            if $type_ptr == crate::typeref::$constant {
                let val = $object.cast_unchecked::<$type>();
                return Ok(wrap_typeengine!(val.get().clone(), $object.as_ptr()));
            }
        )*
    };
}

macro_rules! infer_rules {
    (
        $py:expr,
        $object_ptr:expr,
        $type_ptr:expr,
        $(($cond:expr) => $result:expr,)*
    ) => {
        $(
            if $cond {
                return Ok(wrap_typeengine!($py, $result, $result));
            }
        )*
    };
}

impl TypeEngine {
    /// Creates a new [`TypeEngine`].
    pub fn new(object: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        unsafe {
            let typ = pyo3::ffi::Py_TYPE(object.as_ptr());

            check_native_column_types!(
                object,
                typ,
                PyBlobType => BLOB_COLUMN_TYPE,
                PyBinaryType => BINARY_COLUMN_TYPE,
                PyVarBinaryType => VAR_BINARY_COLUMN_TYPE,
                PyBitType => BIT_COLUMN_TYPE,
                PyVarBitType => VAR_BIT_COLUMN_TYPE,
                PyDateTimeType => DATETIME_COLUMN_TYPE,
                PyTimestampType => TIMESTAMP_COLUMN_TYPE,
                PyTimeType => TIME_COLUMN_TYPE,
                PyDateType => DATE_COLUMN_TYPE,
                PyJSONType => JSON_COLUMN_TYPE,
                PyJSONBinaryType => JSON_BINARY_COLUMN_TYPE,
                PyDecimalType => DECIMAL_COLUMN_TYPE,
                PyUUIDType => UUID_COLUMN_TYPE,
                PyINETType => INET_COLUMN_TYPE,
                PyMacAddressType => MAC_ADDRESS_COLUMN_TYPE,
                PyBooleanType => BOOLEAN_COLUMN_TYPE,
                PyTinyIntegerType => TINY_INTEGER_COLUMN_TYPE,
                PySmallIntegerType => SMALL_INTEGER_COLUMN_TYPE,
                PyIntegerType => INTEGER_COLUMN_TYPE,
                PyBigIntegerType => BIG_INTEGER_COLUMN_TYPE,
                PyTinyUnsignedType => TINY_UNSIGNED_COLUMN_TYPE,
                PySmallUnsignedType => SMALL_UNSIGNED_COLUMN_TYPE,
                PyUnsignedType => UNSIGNED_COLUMN_TYPE,
                PyBigUnsignedType => BIG_UNSIGNED_COLUMN_TYPE,
                PyFloatType => FLOAT_COLUMN_TYPE,
                PyDoubleType => DOUBLE_COLUMN_TYPE,
                PyTextType => TEXT_COLUMN_TYPE,
                PyCharType => CHAR_COLUMN_TYPE,
                PyStringType => STRING_COLUMN_TYPE,
                PyVectorType => VECTOR_COLUMN_TYPE,
                PyArrayType => ARRAY_COLUMN_TYPE,
            );

            Err(typeerror!(
                "expected SQLTypeAbstract, got {:?}",
                object.py(),
                object.as_ptr(),
            ))
        }
    }

    /// Tries to guess a native column type depends on type of the `object`.
    #[cfg_attr(feature = "optimize", optimize(speed))]
    pub fn infer_pyobject(object: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        unsafe {
            let object_ptr = object.as_ptr();
            let object_type_ptr = pyo3::ffi::Py_TYPE(object_ptr);

            if pyo3::ffi::PyLong_CheckExact(object_ptr) == 1 {
                return Ok(wrap_typeengine!(object.py(), PyBigIntegerType, PyBigIntegerType));
            }

            infer_rules!(
                object.py(),
                object_ptr,
                object_type_ptr,

                (pyo3::ffi::Py_IsNone(object_ptr) == 1) => PyBooleanType,
                (pyo3::ffi::PyBool_Check(object_ptr) == 1) => PyBooleanType,
                (pyo3::ffi::PyFloat_CheckExact(object_ptr) == 1) => PyDoubleType,
                (pyo3::ffi::PyUnicode_CheckExact(object_ptr) == 1) => PyTextType,
                (pyo3::ffi::PyBytes_CheckExact(object_ptr) == 1) => PyBlobType,

                (object_type_ptr == crate::typeref::STD_DATETIME_TYPE) => PyDateTimeType,
                (object_type_ptr == crate::typeref::STD_DATE_TYPE) => PyDateType,
                (object_type_ptr == crate::typeref::STD_TIME_TYPE) => PyTimeType,
                (object_type_ptr == crate::typeref::STD_DECIMAL_TYPE) => PyDecimalType(None),
                (object_type_ptr == crate::typeref::STD_UUID_TYPE) => PyUUIDType,
            );

            if (pyo3::ffi::PyDict_CheckExact(object_ptr) == 1
                || pyo3::ffi::PyList_CheckExact(object_ptr) == 1)
                && _validate_json_object(object.py(), object_ptr).is_ok()
            {
                return Ok(wrap_typeengine!(object.py(), PyJSONType, PyJSONType));
            }

            Err(typeerror!(
                "Could not infer column/sql type for {:?}",
                object.py(),
                object_ptr,
            ))
        }
    }

    /// Tries to guess a native column type depends on type of the `object`.
    #[cfg_attr(feature = "optimize", optimize(speed))]
    pub fn infer_value(py: pyo3::Python<'_>, object: &sea_query::Value) -> Self {
        match object {
            sea_query::Value::Bool(_) => wrap_typeengine!(py, PyBooleanType, PyBooleanType),
            sea_query::Value::TinyInt(_) => wrap_typeengine!(py, PyTinyIntegerType, PyTinyIntegerType),
            sea_query::Value::SmallInt(_) => wrap_typeengine!(py, PySmallIntegerType, PySmallIntegerType),
            sea_query::Value::Int(_) => wrap_typeengine!(py, PyIntegerType, PyIntegerType),
            sea_query::Value::BigInt(_) => wrap_typeengine!(py, PyBigIntegerType, PyBigIntegerType),
            sea_query::Value::TinyUnsigned(_) => wrap_typeengine!(py, PyTinyUnsignedType, PyTinyUnsignedType),
            sea_query::Value::SmallUnsigned(_) => {
                wrap_typeengine!(py, PySmallUnsignedType, PySmallUnsignedType)
            }
            sea_query::Value::Unsigned(_) => wrap_typeengine!(py, PyUnsignedType, PyUnsignedType),
            sea_query::Value::BigUnsigned(_) => wrap_typeengine!(py, PyBigUnsignedType, PyBigUnsignedType),
            sea_query::Value::Float(_) => wrap_typeengine!(py, PyFloatType, PyFloatType),
            sea_query::Value::Double(_) => wrap_typeengine!(py, PyDoubleType, PyDoubleType),
            sea_query::Value::String(_) => wrap_typeengine!(py, PyStringType(None), PyStringType(None)),
            sea_query::Value::Char(_) => wrap_typeengine!(py, PyCharType(None), PyCharType(None)),
            sea_query::Value::Bytes(_) => wrap_typeengine!(py, PyBlobType, PyBlobType),
            sea_query::Value::Json(_) => wrap_typeengine!(py, PyJSONType, PyJSONType),
            sea_query::Value::ChronoDate(_) => wrap_typeengine!(py, PyDateType, PyDateType),
            sea_query::Value::ChronoTime(_) => wrap_typeengine!(py, PyTimeType, PyTimeType),
            sea_query::Value::ChronoDateTime(_) => wrap_typeengine!(py, PyDateTimeType, PyDateTimeType),
            sea_query::Value::ChronoDateTimeUtc(_) => wrap_typeengine!(py, PyDateTimeType, PyDateTimeType),
            sea_query::Value::ChronoDateTimeLocal(_) => wrap_typeengine!(py, PyDateTimeType, PyDateTimeType),
            sea_query::Value::ChronoDateTimeWithTimeZone(_) => {
                wrap_typeengine!(py, PyDateTimeType, PyDateTimeType)
            }
            sea_query::Value::Uuid(_) => wrap_typeengine!(py, PyUUIDType, PyUUIDType),
            sea_query::Value::Decimal(_) => wrap_typeengine!(py, PyDecimalType(None), PyDecimalType(None)),
            sea_query::Value::Array(_, Some(nested)) => {
                let nested_type_engine = {
                    if nested.len() == 0 {
                        // Vec is empty so the nested type is not important
                        wrap_typeengine!(py, PyBooleanType, PyBooleanType)
                    } else {
                        Self::infer_value(py, &nested[0])
                    }
                };

                wrap_typeengine!(
                    py,
                    PyArrayType(nested_type_engine),
                    PyArrayType(nested_type_engine.clone())
                )
            }
            sea_query::Value::Array(_, None) => {
                // Vec is None so the nested type is not important
                let nested_type_engine = wrap_typeengine!(py, PyBooleanType, PyBooleanType);
                wrap_typeengine!(
                    py,
                    PyArrayType(nested_type_engine),
                    PyArrayType(nested_type_engine.clone())
                )
            }
            sea_query::Value::Vector(_) => wrap_typeengine!(py, PyVectorType(None), PyVectorType(None)),
            sea_query::Value::IpNetwork(_) => wrap_typeengine!(py, PyINETType, PyINETType),
            sea_query::Value::MacAddress(_) => wrap_typeengine!(py, PyMacAddressType, PyMacAddressType),
        }
    }

    pub fn as_pyobject<'py>(&self, py: pyo3::Python<'py>) -> pyo3::Bound<'py, pyo3::PyAny> {
        unsafe { pyo3::Bound::from_borrowed_ptr(py, self.1.as_ptr()) }
    }
}

impl NativeSQLType for TypeEngine {
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        self.0.to_sea_query_column_type()
    }

    unsafe fn validate(&self, py: pyo3::Python, ptr: *mut pyo3::ffi::PyObject) -> pyo3::PyResult<()> {
        self.0.validate(py, ptr)
    }

    unsafe fn deserialize(
        &self,
        py: pyo3::Python,
        value: &sea_query::Value,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        self.0.deserialize(py, value)
    }

    fn to_sql_type_name(&self) -> String {
        self.0.to_sql_type_name()
    }

    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<sea_query::Value> {
        self.0.serialize(py, ptr)
    }
}

impl std::fmt::Display for TypeEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            pyo3::Python::attach_unchecked(|py| {
                let object = self.as_pyobject(py);
                std::fmt::Display::fmt(&object, f)
            })
        }
    }
}

impl std::fmt::Debug for TypeEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            pyo3::Python::attach_unchecked(|py| {
                let object = self.as_pyobject(py);
                std::fmt::Debug::fmt(&object, f)
            })
        }
    }
}
