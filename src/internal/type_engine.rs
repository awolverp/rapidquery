use crate::sqltypes::SQLTypeTrait;

/// Type engine is an enum which can control validations, serializations,
/// and deserializations of column types.
///
/// [`SQLTypeTrait`] is implemented for this type.
pub struct TypeEngine(
    pub std::sync::Arc<dyn SQLTypeTrait + Send + Sync>,
    pub std::ptr::NonNull<pyo3::ffi::PyObject>,
);

// SAFETY: PyObjects are safe between threads.
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
            let object_ptr = pyo3::Py::new($py, ($initializer, crate::sqltypes::PySQLTypeAbstract))
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
                crate::sqltypes::PyBlobType => BLOB_COLUMN_TYPE,
                crate::sqltypes::PyBinaryType => BINARY_COLUMN_TYPE,
                crate::sqltypes::PyVarBinaryType => VAR_BINARY_COLUMN_TYPE,
                crate::sqltypes::PyBitType => BIT_COLUMN_TYPE,
                crate::sqltypes::PyVarBitType => VAR_BIT_COLUMN_TYPE,
                crate::sqltypes::PyDateTimeType => DATETIME_COLUMN_TYPE,
                crate::sqltypes::PyTimestampType => TIMESTAMP_COLUMN_TYPE,
                crate::sqltypes::PyTimeType => TIME_COLUMN_TYPE,
                crate::sqltypes::PyDateType => DATE_COLUMN_TYPE,
                crate::sqltypes::PyJSONType => JSON_COLUMN_TYPE,
                crate::sqltypes::PyJSONBinaryType => JSON_BINARY_COLUMN_TYPE,
                crate::sqltypes::PyDecimalType => DECIMAL_COLUMN_TYPE,
                crate::sqltypes::PyUUIDType => UUID_COLUMN_TYPE,
                crate::sqltypes::PyINETType => INET_COLUMN_TYPE,
                crate::sqltypes::PyMacAddressType => MAC_ADDRESS_COLUMN_TYPE,
                crate::sqltypes::PyBooleanType => BOOLEAN_COLUMN_TYPE,
                crate::sqltypes::PyTinyIntegerType => TINY_INTEGER_COLUMN_TYPE,
                crate::sqltypes::PySmallIntegerType => SMALL_INTEGER_COLUMN_TYPE,
                crate::sqltypes::PyIntegerType => INTEGER_COLUMN_TYPE,
                crate::sqltypes::PyBigIntegerType => BIG_INTEGER_COLUMN_TYPE,
                crate::sqltypes::PyTinyUnsignedType => TINY_UNSIGNED_COLUMN_TYPE,
                crate::sqltypes::PySmallUnsignedType => SMALL_UNSIGNED_COLUMN_TYPE,
                crate::sqltypes::PyUnsignedType => UNSIGNED_COLUMN_TYPE,
                crate::sqltypes::PyBigUnsignedType => BIG_UNSIGNED_COLUMN_TYPE,
                crate::sqltypes::PyFloatType => FLOAT_COLUMN_TYPE,
                crate::sqltypes::PyDoubleType => DOUBLE_COLUMN_TYPE,
                crate::sqltypes::PyTextType => TEXT_COLUMN_TYPE,
                crate::sqltypes::PyCharType => CHAR_COLUMN_TYPE,
                crate::sqltypes::PyStringType => STRING_COLUMN_TYPE,
                crate::sqltypes::PyVectorType => VECTOR_COLUMN_TYPE,
                crate::sqltypes::PyArrayType => ARRAY_COLUMN_TYPE,
                crate::sqltypes::PyEnumType => ENUM_COLUMN_TYPE,
            );

            crate::new_error!(
                PyTypeError,
                "expected SQLTypeAbstract, got {:?}",
                crate::internal::get_type_name(object.py(), object.as_ptr())
            )
        }
    }

    /// Tries to guess a native column type depends on type of the `object`.
    #[cfg_attr(feature = "optimize", optimize(speed))]
    pub fn infer_pyobject(object: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        unsafe {
            let object_ptr = object.as_ptr();
            let object_type_ptr = pyo3::ffi::Py_TYPE(object_ptr);

            if pyo3::ffi::PyLong_CheckExact(object_ptr) == 1 {
                return Ok(wrap_typeengine!(
                    object.py(),
                    crate::sqltypes::PyBigIntegerType,
                    crate::sqltypes::PyBigIntegerType
                ));
            }

            infer_rules!(
                object.py(),
                object_ptr,
                object_type_ptr,

                (pyo3::ffi::Py_IsNone(object_ptr) == 1) => crate::sqltypes::PyBooleanType,
                (pyo3::ffi::PyBool_Check(object_ptr) == 1) => crate::sqltypes::PyBooleanType,
                (pyo3::ffi::PyFloat_CheckExact(object_ptr) == 1) => crate::sqltypes::PyDoubleType,
                (pyo3::ffi::PyUnicode_CheckExact(object_ptr) == 1) => crate::sqltypes::PyTextType,
                (pyo3::ffi::PyBytes_CheckExact(object_ptr) == 1) => crate::sqltypes::PyBlobType,

                (object_type_ptr == crate::typeref::STD_DATETIME_TYPE) => crate::sqltypes::PyDateTimeType,
                (object_type_ptr == crate::typeref::STD_DATE_TYPE) => crate::sqltypes::PyDateType,
                (object_type_ptr == crate::typeref::STD_TIME_TYPE) => crate::sqltypes::PyTimeType,
                (object_type_ptr == crate::typeref::STD_DECIMAL_TYPE) => crate::sqltypes::PyDecimalType(None),
                (object_type_ptr == crate::typeref::STD_UUID_TYPE) => crate::sqltypes::PyUUIDType,
            );

            if (pyo3::ffi::PyDict_CheckExact(object_ptr) == 1
                || pyo3::ffi::PyList_CheckExact(object_ptr) == 1)
                && crate::sqltypes::_validate_json_object(object.py(), object_ptr).is_ok()
            {
                return Ok(wrap_typeengine!(
                    object.py(),
                    crate::sqltypes::PyJSONType,
                    crate::sqltypes::PyJSONType
                ));
            }

            crate::new_error!(
                PyTypeError,
                "Could not infer column/sql type for {:?}",
                crate::internal::get_type_name(object.py(), object.as_ptr())
            )
        }
    }

    /// Tries to guess a native column type depends on type of the `object`.
    #[cfg_attr(feature = "optimize", optimize(speed))]
    pub fn infer_value(py: pyo3::Python<'_>, object: &sea_query::Value) -> Self {
        match object {
            sea_query::Value::Bool(_) => wrap_typeengine!(
                py,
                crate::sqltypes::PyBooleanType,
                crate::sqltypes::PyBooleanType
            ),
            sea_query::Value::TinyInt(_) => {
                wrap_typeengine!(
                    py,
                    crate::sqltypes::PyTinyIntegerType,
                    crate::sqltypes::PyTinyIntegerType
                )
            }
            sea_query::Value::SmallInt(_) => {
                wrap_typeengine!(
                    py,
                    crate::sqltypes::PySmallIntegerType,
                    crate::sqltypes::PySmallIntegerType
                )
            }
            sea_query::Value::Int(_) => wrap_typeengine!(
                py,
                crate::sqltypes::PyIntegerType,
                crate::sqltypes::PyIntegerType
            ),
            sea_query::Value::BigInt(_) => wrap_typeengine!(
                py,
                crate::sqltypes::PyBigIntegerType,
                crate::sqltypes::PyBigIntegerType
            ),
            sea_query::Value::TinyUnsigned(_) => {
                wrap_typeengine!(
                    py,
                    crate::sqltypes::PyTinyUnsignedType,
                    crate::sqltypes::PyTinyUnsignedType
                )
            }
            sea_query::Value::SmallUnsigned(_) => {
                wrap_typeengine!(
                    py,
                    crate::sqltypes::PySmallUnsignedType,
                    crate::sqltypes::PySmallUnsignedType
                )
            }
            sea_query::Value::Unsigned(_) => wrap_typeengine!(
                py,
                crate::sqltypes::PyUnsignedType,
                crate::sqltypes::PyUnsignedType
            ),
            sea_query::Value::BigUnsigned(_) => {
                wrap_typeengine!(
                    py,
                    crate::sqltypes::PyBigUnsignedType,
                    crate::sqltypes::PyBigUnsignedType
                )
            }
            sea_query::Value::Float(_) => wrap_typeengine!(
                py,
                crate::sqltypes::PyFloatType,
                crate::sqltypes::PyFloatType
            ),
            sea_query::Value::Double(_) => wrap_typeengine!(
                py,
                crate::sqltypes::PyDoubleType,
                crate::sqltypes::PyDoubleType
            ),
            sea_query::Value::String(_) => {
                wrap_typeengine!(
                    py,
                    crate::sqltypes::PyStringType(None),
                    crate::sqltypes::PyStringType(None)
                )
            }
            sea_query::Value::Char(_) => wrap_typeengine!(
                py,
                crate::sqltypes::PyCharType(None),
                crate::sqltypes::PyCharType(None)
            ),
            sea_query::Value::Bytes(_) => {
                wrap_typeengine!(py, crate::sqltypes::PyBlobType, crate::sqltypes::PyBlobType)
            }
            sea_query::Value::Json(_) => {
                wrap_typeengine!(py, crate::sqltypes::PyJSONType, crate::sqltypes::PyJSONType)
            }
            sea_query::Value::ChronoDate(_) => {
                wrap_typeengine!(py, crate::sqltypes::PyDateType, crate::sqltypes::PyDateType)
            }
            sea_query::Value::ChronoTime(_) => {
                wrap_typeengine!(py, crate::sqltypes::PyTimeType, crate::sqltypes::PyTimeType)
            }
            sea_query::Value::ChronoDateTime(_) => {
                wrap_typeengine!(
                    py,
                    crate::sqltypes::PyDateTimeType,
                    crate::sqltypes::PyDateTimeType
                )
            }
            sea_query::Value::ChronoDateTimeUtc(_) => {
                wrap_typeengine!(
                    py,
                    crate::sqltypes::PyDateTimeType,
                    crate::sqltypes::PyDateTimeType
                )
            }
            sea_query::Value::ChronoDateTimeLocal(_) => {
                wrap_typeengine!(
                    py,
                    crate::sqltypes::PyDateTimeType,
                    crate::sqltypes::PyDateTimeType
                )
            }
            sea_query::Value::ChronoDateTimeWithTimeZone(_) => {
                wrap_typeengine!(
                    py,
                    crate::sqltypes::PyDateTimeType,
                    crate::sqltypes::PyDateTimeType
                )
            }
            sea_query::Value::Uuid(_) => {
                wrap_typeengine!(py, crate::sqltypes::PyUUIDType, crate::sqltypes::PyUUIDType)
            }
            sea_query::Value::Decimal(_) => {
                wrap_typeengine!(
                    py,
                    crate::sqltypes::PyDecimalType(None),
                    crate::sqltypes::PyDecimalType(None)
                )
            }
            sea_query::Value::Array(_, Some(nested)) => {
                let nested_type_engine = {
                    if nested.is_empty() {
                        // Vec is empty so the nested type is not important
                        wrap_typeengine!(
                            py,
                            crate::sqltypes::PyBooleanType,
                            crate::sqltypes::PyBooleanType
                        )
                    } else {
                        Self::infer_value(py, &nested[0])
                    }
                };

                wrap_typeengine!(
                    py,
                    crate::sqltypes::PyArrayType(nested_type_engine),
                    crate::sqltypes::PyArrayType(nested_type_engine.clone())
                )
            }
            sea_query::Value::Array(_, None) => {
                // Vec is None so the nested type is not important
                let nested_type_engine = wrap_typeengine!(
                    py,
                    crate::sqltypes::PyBooleanType,
                    crate::sqltypes::PyBooleanType
                );
                wrap_typeengine!(
                    py,
                    crate::sqltypes::PyArrayType(nested_type_engine),
                    crate::sqltypes::PyArrayType(nested_type_engine.clone())
                )
            }
            sea_query::Value::Vector(_) => {
                wrap_typeengine!(
                    py,
                    crate::sqltypes::PyVectorType(None),
                    crate::sqltypes::PyVectorType(None)
                )
            }
            sea_query::Value::IpNetwork(_) => {
                wrap_typeengine!(py, crate::sqltypes::PyINETType, crate::sqltypes::PyINETType)
            }
            sea_query::Value::MacAddress(_) => {
                wrap_typeengine!(
                    py,
                    crate::sqltypes::PyMacAddressType,
                    crate::sqltypes::PyMacAddressType
                )
            }
        }
    }

    pub fn as_pyobject<'py>(&self, py: pyo3::Python<'py>) -> pyo3::Bound<'py, pyo3::PyAny> {
        unsafe { pyo3::Bound::from_borrowed_ptr(py, self.1.as_ptr()) }
    }
}

impl SQLTypeTrait for TypeEngine {
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        self.0.to_sea_query_column_type()
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
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
