use crate::sqltypes::abstracts::{PySQLTypeAbstract, SQLTypeTrait};

crate::implement_pyclass! {
    /// Boolean column type (BOOLEAN).
    ///
    /// Stores true/false values. The standard way to store boolean data,
    /// though implementation varies by database (some use TINYINT(1) or
    /// similar representations).
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyBooleanType as "Boolean";
}
crate::implement_pyclass! {
    /// Very small integer column type (TINYINT).
    ///
    /// Typically stores integers in the range -128 to 127 (signed) or 0 to 255
    /// (unsigned). Useful for flags, small counters, or enumerated values.
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyTinyIntegerType as "TinyInteger";
}
crate::implement_pyclass! {
    /// Small integer column type (SMALLINT).
    ///
    /// Typically stores integers in the range -32,768 to 32,767 (signed) or
    /// 0 to 65,535 (unsigned). Good for moderate-sized counters or numeric codes.
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PySmallIntegerType as "SmallInteger";
}
crate::implement_pyclass! {
    /// Standard integer column type (INTEGER/INT).
    ///
    /// The most common integer type, typically storing 32-bit integers in the
    /// range -2,147,483,648 to 2,147,483,647 (signed). Suitable for most
    /// numeric data including IDs, quantities, and counters.
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyIntegerType as "Integer";
}
crate::implement_pyclass! {
    /// Large integer column type (BIGINT).
    ///
    /// Stores 64-bit integers for very large numeric values. Essential for
    /// high-volume systems, timestamps, large counters, or when integer
    /// overflow is a concern.
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyBigIntegerType as "BigInteger";
}
crate::implement_pyclass! {
    /// Unsigned tiny integer column type.
    ///
    /// Stores small positive integers only, typically 0 to 255. Useful for
    /// small counters, percentages, or enumerated values that are always positive.
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyTinyUnsignedType as "TinyUnsigned";
}
crate::implement_pyclass! {
    /// Unsigned small integer column type.
    ///
    /// Stores moderate positive integers only, typically 0 to 65,535. Good for
    /// larger counters or numeric codes that are always positive.
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PySmallUnsignedType as "SmallUnsigned";
}
crate::implement_pyclass! {
    /// Unsigned integer column type.
    ///
    /// Stores positive integers only, typically 0 to 4,294,967,295. Doubles the
    /// positive range compared to signed integers, useful for IDs and counters
    /// that will never be negative.
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyUnsignedType as "Unsigned";
}
crate::implement_pyclass! {
    /// Unsigned big integer column type.
    ///
    /// Stores very large positive integers only. Provides the maximum positive
    /// integer range for high-volume systems or when very large positive
    /// values are required.
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyBigUnsignedType as "BigUnsigned";
}
crate::implement_pyclass! {
    /// Single-precision floating point column type (FLOAT).
    ///
    /// Stores approximate numeric values with single precision. Suitable for
    /// scientific calculations, measurements, or any numeric data where some
    /// precision loss is acceptable in exchange for storage efficiency.
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyFloatType as "Float";
}
crate::implement_pyclass! {
    /// Double-precision floating point column type (DOUBLE).
    ///
    /// Stores approximate numeric values with double precision. Provides higher
    /// precision than FLOAT for scientific calculations or when more accuracy
    /// is required in floating-point operations.
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyDoubleType as "Double";
}
crate::implement_pyclass! {
    /// Large text column type (TEXT).
    ///
    /// Represents a large text field capable of storing long strings without
    /// a predefined length limit. Suitable for storing articles, comments,
    /// descriptions, or any text content that may be very long.
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyTextType as "Text";
}
crate::implement_pyclass! {
    /// Fixed-length character string column type (CHAR).
    ///
    /// Represents a fixed-length character string. Values shorter than the
    /// specified length are padded with spaces. Suitable for storing data
    /// with consistent, known lengths like country codes or status flags.
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyCharType as "Char" (pub Option<u32>);
}
crate::implement_pyclass! {
    /// Variable-length character string column type (VARCHAR).
    ///
    /// Represents a variable-length character string with a maximum length limit.
    /// This is the most common string type for storing text data of varying lengths
    /// like names, descriptions, or user input.
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyStringType as "String" (pub Option<u32>);
}

impl SQLTypeTrait for PyBooleanType {
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::Boolean
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        if pyo3::ffi::PyBool_Check(ptr) != 1 {
            crate::new_error!(
                PyTypeError,
                "expected bool for {} serialization, got {}",
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
        if pyo3::ffi::Py_True() == ptr {
            Ok(sea_query::Value::Bool(Some(true)))
        } else {
            Ok(sea_query::Value::Bool(Some(false)))
        }
    }

    unsafe fn deserialize(
        &self,
        _py: pyo3::Python,
        value: &sea_query::Value,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        match value {
            sea_query::Value::Bool(Some(x)) if *x => Ok(pyo3::ffi::Py_True()),
            sea_query::Value::Bool(Some(x)) if !*x => Ok(pyo3::ffi::Py_False()),
            sea_query::Value::Bool(None) => Ok(pyo3::ffi::Py_None()),
            _ => crate::new_error!(
                PyTypeError,
                "expected bool for {} deserialization, got {:?}",
                self.to_sql_type_name(),
                value
            ),
        }
    }
}

macro_rules! implement_numeric_SQLTypeTrait {
    (
        $name:ident,
        $type:ty,
        $python_type_name:literal,
        $column_type:ident,
        $value_type:ident,
        $convertfunction:ident,
        $($checkfunction:ident,)+
    ) => {
        impl SQLTypeTrait for $name {
            fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
                sea_query::ColumnType::$column_type
            }

            unsafe fn validate(&self, py: pyo3::Python, ptr: *mut pyo3::ffi::PyObject) -> pyo3::PyResult<()> {
                self.serialize(py, ptr)?;
                Ok(())
            }

            unsafe fn serialize(
                &self,
                py: pyo3::Python,
                ptr: *mut pyo3::ffi::PyObject,
            ) -> pyo3::PyResult<sea_query::Value> {
                $(
                    if pyo3::ffi::$checkfunction(ptr) == 1 {
                        let val = pyo3::ffi::$convertfunction(ptr);

                        if !pyo3::ffi::PyErr_Occurred().is_null() {
                            return Err(pyo3::PyErr::fetch(py));
                        }

                        return TryInto::<$type>::try_into(val)
                            .map_err(|_| pyo3::exceptions::PyOverflowError::new_err("out of range"))
                            .map(|x| sea_query::Value::$value_type(Some(x)));
                    }
                )+

                crate::new_error!(
                    PyTypeError,
                    concat!("expected ", $python_type_name, " for {} serialization, got {}"),
                    self.to_sql_type_name(),
                    crate::internal::get_type_name(py, ptr)
                )
            }

            unsafe fn deserialize(
                &self,
                py: pyo3::Python,
                value: &sea_query::Value,
            ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
                let val = match value {
                    sea_query::Value::BigInt(Some(x)) => pyo3::ffi::PyLong_FromLongLong(*x),
                    sea_query::Value::Int(Some(x)) => pyo3::ffi::PyLong_FromLong((*x) as _),
                    sea_query::Value::SmallInt(Some(x)) => pyo3::ffi::PyLong_FromLong((*x) as _),
                    sea_query::Value::TinyInt(Some(x)) => pyo3::ffi::PyLong_FromLong((*x) as _),
                    sea_query::Value::BigUnsigned(Some(x)) => pyo3::ffi::PyLong_FromUnsignedLongLong(*x),
                    sea_query::Value::Unsigned(Some(x)) => pyo3::ffi::PyLong_FromUnsignedLong((*x) as _),
                    sea_query::Value::SmallUnsigned(Some(x)) => {
                        pyo3::ffi::PyLong_FromUnsignedLong((*x) as _)
                    }
                    sea_query::Value::TinyUnsigned(Some(x)) => {
                        pyo3::ffi::PyLong_FromUnsignedLong((*x) as _)
                    }
                    sea_query::Value::Double(Some(x)) => pyo3::ffi::PyFloat_FromDouble(*x),
                    sea_query::Value::Float(Some(x)) => pyo3::ffi::PyFloat_FromDouble((*x) as _),
                    sea_query::Value::BigInt(None)
                    | sea_query::Value::Int(None)
                    | sea_query::Value::SmallInt(None)
                    | sea_query::Value::TinyInt(None)
                    | sea_query::Value::BigUnsigned(None)
                    | sea_query::Value::Unsigned(None)
                    | sea_query::Value::SmallUnsigned(None)
                    | sea_query::Value::TinyUnsigned(None) => pyo3::ffi::Py_None(),
                    _ => return crate::new_error!(
                        PyTypeError,
                        "expected int for {} deserialization, got {:?}",
                        self.to_sql_type_name(),
                        value
                    ),
                };

                if val.is_null() {
                    Err(pyo3::PyErr::fetch(py))
                } else {
                    Ok(val)
                }
            }
        }
    };
}

implement_numeric_SQLTypeTrait!(
    PyBigIntegerType,
    i64,
    "int",
    BigInteger,
    BigInt,
    PyLong_AsLongLong,
    PyLong_CheckExact,
);
implement_numeric_SQLTypeTrait!(
    PyIntegerType,
    i32,
    "int",
    Integer,
    Int,
    PyLong_AsLongLong,
    PyLong_CheckExact,
);
implement_numeric_SQLTypeTrait!(
    PySmallIntegerType,
    i16,
    "int",
    SmallInteger,
    SmallInt,
    PyLong_AsLongLong,
    PyLong_CheckExact,
);
implement_numeric_SQLTypeTrait!(
    PyTinyIntegerType,
    i8,
    "int",
    TinyInteger,
    TinyInt,
    PyLong_AsLongLong,
    PyLong_CheckExact,
);
implement_numeric_SQLTypeTrait!(
    PyBigUnsignedType,
    u64,
    "int",
    BigUnsigned,
    BigUnsigned,
    PyLong_AsUnsignedLongLong,
    PyLong_CheckExact,
);
implement_numeric_SQLTypeTrait!(
    PyUnsignedType,
    u32,
    "int",
    Unsigned,
    Unsigned,
    PyLong_AsUnsignedLong,
    PyLong_CheckExact,
);
implement_numeric_SQLTypeTrait!(
    PySmallUnsignedType,
    u16,
    "int",
    SmallUnsigned,
    SmallUnsigned,
    PyLong_AsUnsignedLong,
    PyLong_CheckExact,
);
implement_numeric_SQLTypeTrait!(
    PyTinyUnsignedType,
    u8,
    "int",
    TinyUnsigned,
    TinyUnsigned,
    PyLong_AsUnsignedLong,
    PyLong_CheckExact,
);
implement_numeric_SQLTypeTrait!(
    PyFloatType,
    // We're using f64 instead of f32 'cause we don't have f32::try_from(f64)
    f64,
    "float",
    Float,
    Double,
    PyFloat_AsDouble,
    PyFloat_CheckExact,
    PyLong_CheckExact,
);
implement_numeric_SQLTypeTrait!(
    PyDoubleType,
    f64,
    "float",
    Float,
    Double,
    PyFloat_AsDouble,
    PyFloat_CheckExact,
    PyLong_CheckExact,
);

#[inline]
pub(super) unsafe fn _serialize_string(
    py: pyo3::Python,
    object: *mut pyo3::ffi::PyObject,
) -> pyo3::PyResult<String> {
    let mut size: pyo3::ffi::Py_ssize_t = 0;
    let c_str = pyo3::ffi::PyUnicode_AsUTF8AndSize(object, &mut size);

    if c_str.is_null() || size < 0 {
        Err(pyo3::PyErr::fetch(py))
    } else {
        let val = std::ffi::CStr::from_ptr(c_str);
        Ok(std::str::from_utf8_unchecked(val.to_bytes()).into())
    }
}

#[inline]
pub(super) unsafe fn _deserialize_string(
    py: pyo3::Python,
    value: &str,
) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
    let val = pyo3::types::PyString::intern(py, std::str::from_utf8_unchecked(value.as_bytes()));
    Ok(val.into_ptr())
}

impl SQLTypeTrait for PyTextType {
    #[inline(always)]
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::Text
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        if pyo3::ffi::PyUnicode_CheckExact(ptr) == 0 {
            crate::new_error!(
                PyTypeError,
                "expected str for {} serialization, got {}",
                self.to_sql_type_name(),
                crate::internal::get_type_name(py, ptr)
            )
        } else {
            Ok(())
        }
    }

    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        object: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<sea_query::Value> {
        _serialize_string(py, object).map(|x| sea_query::Value::String(Some(Box::new(x))))
    }

    unsafe fn deserialize(
        &self,
        py: pyo3::Python,
        value: &sea_query::Value,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        match value {
            sea_query::Value::String(Some(x)) => _deserialize_string(py, x),
            sea_query::Value::String(None) => Ok(pyo3::ffi::Py_None()),
            _ => crate::new_error!(
                PyTypeError,
                "expected str for {} deserialization, got {:?}",
                self.to_sql_type_name(),
                value
            ),
        }
    }
}

impl SQLTypeTrait for PyCharType {
    #[inline(always)]
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::Char(self.0)
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        if pyo3::ffi::PyUnicode_CheckExact(ptr) == 0 {
            crate::new_error!(
                PyTypeError,
                "expected str for {} serialization, got {}",
                self.to_sql_type_name(),
                crate::internal::get_type_name(py, ptr)
            )
        } else {
            Ok(())
        }
    }

    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        object: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<sea_query::Value> {
        _serialize_string(py, object).map(|x| sea_query::Value::String(Some(Box::new(x))))
    }

    unsafe fn deserialize(
        &self,
        py: pyo3::Python,
        value: &sea_query::Value,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        match value {
            sea_query::Value::String(Some(x)) => _deserialize_string(py, x),
            sea_query::Value::String(None) => Ok(pyo3::ffi::Py_None()),
            _ => crate::new_error!(
                PyTypeError,
                "expected str for {} deserialization, got {:?}",
                self.to_sql_type_name(),
                value
            ),
        }
    }
}

impl SQLTypeTrait for PyStringType {
    #[inline(always)]
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::String(
            self.0
                .map_or(sea_query::StringLen::None, sea_query::StringLen::N),
        )
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        if pyo3::ffi::PyUnicode_CheckExact(ptr) == 0 {
            crate::new_error!(
                PyTypeError,
                "expected str for {} serialization, got {}",
                self.to_sql_type_name(),
                crate::internal::get_type_name(py, ptr)
            )
        } else {
            Ok(())
        }
    }

    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        object: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<sea_query::Value> {
        _serialize_string(py, object).map(|x| sea_query::Value::String(Some(Box::new(x))))
    }

    unsafe fn deserialize(
        &self,
        py: pyo3::Python,
        value: &sea_query::Value,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        match value {
            sea_query::Value::String(Some(x)) => _deserialize_string(py, x),
            sea_query::Value::String(None) => Ok(pyo3::ffi::Py_None()),
            _ => crate::new_error!(
                PyTypeError,
                "expected str for {} deserialization, got {:?}",
                self.to_sql_type_name(),
                value
            ),
        }
    }
}

super::abstracts::implement_sqltype_pymethods!(PyBooleanType);
super::abstracts::implement_sqltype_pymethods!(PyBigIntegerType);
super::abstracts::implement_sqltype_pymethods!(PyIntegerType);
super::abstracts::implement_sqltype_pymethods!(PySmallIntegerType);
super::abstracts::implement_sqltype_pymethods!(PyTinyIntegerType);
super::abstracts::implement_sqltype_pymethods!(PyBigUnsignedType);
super::abstracts::implement_sqltype_pymethods!(PyUnsignedType);
super::abstracts::implement_sqltype_pymethods!(PySmallUnsignedType);
super::abstracts::implement_sqltype_pymethods!(PyTinyUnsignedType);
super::abstracts::implement_sqltype_pymethods!(PyFloatType);
super::abstracts::implement_sqltype_pymethods!(PyDoubleType);
super::abstracts::implement_sqltype_pymethods!(PyTextType);
super::abstracts::implement_sqltype_pymethods!(
    PyCharType,
    init(|length: Option<u32>| Self(length)),
    "int | None",
    signature(length = None)
);
super::abstracts::implement_sqltype_pymethods!(
    PyStringType,
    init(|length: Option<u32>| Self(length)),
    "int | None",
    signature(length = None)
);
