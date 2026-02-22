use crate::sqltypes::abstracts::NativeSQLType;
use crate::sqltypes::abstracts::PySQLTypeAbstract;

implement_pyclass! {
    (
        /// Boolean column type (BOOLEAN).
        ///
        /// Stores true/false values. The standard way to store boolean data,
        /// though implementation varies by database (some use TINYINT(1) or
        /// similar representations).
        ///
        /// @extends SQLTypeAbstract[bool,bool]
        #[derive(Debug, Clone, Copy)]
        pub struct [extends=PySQLTypeAbstract] PyBooleanType as "BooleanType";
    )
    (
        /// Very small integer column type (TINYINT).
        ///
        /// Typically stores integers in the range -128 to 127 (signed) or 0 to 255
        /// (unsigned). Useful for flags, small counters, or enumerated values.
        ///
        /// @extends SQLTypeAbstract[int,int]
        #[derive(Debug, Clone, Copy)]
        pub struct [extends=PySQLTypeAbstract] PyTinyIntegerType as "TinyIntegerType";
    )
    (
        /// Small integer column type (SMALLINT).
        ///
        /// Typically stores integers in the range -32,768 to 32,767 (signed) or
        /// 0 to 65,535 (unsigned). Good for moderate-sized counters or numeric codes.
        ///
        /// @extends SQLTypeAbstract[int,int]
        #[derive(Debug, Clone, Copy)]
        pub struct [extends=PySQLTypeAbstract] PySmallIntegerType as "SmallIntegerType";
    )
    (
        /// Standard integer column type (INTEGER/INT).
        ///
        /// The most common integer type, typically storing 32-bit integers in the
        /// range -2,147,483,648 to 2,147,483,647 (signed). Suitable for most
        /// numeric data including IDs, quantities, and counters.
        ///
        /// @extends SQLTypeAbstract[int,int]
        #[derive(Debug, Clone, Copy)]
        pub struct [extends=PySQLTypeAbstract] PyIntegerType as "IntegerType";
    )
    (
        /// Large integer column type (BIGINT).
        ///
        /// Stores 64-bit integers for very large numeric values. Essential for
        /// high-volume systems, timestamps, large counters, or when integer
        /// overflow is a concern.
        ///
        /// @extends SQLTypeAbstract[int,int]
        #[derive(Debug, Clone, Copy)]
        pub struct [extends=PySQLTypeAbstract] PyBigIntegerType as "BigIntegerType";
    )
    (
        /// Unsigned tiny integer column type.
        ///
        /// Stores small positive integers only, typically 0 to 255. Useful for
        /// small counters, percentages, or enumerated values that are always positive.
        ///
        /// @extends SQLTypeAbstract[int,int]
        #[derive(Debug, Clone, Copy)]
        pub struct [extends=PySQLTypeAbstract] PyTinyUnsignedType as "TinyUnsignedType";
    )
    (
        /// Unsigned small integer column type.
        ///
        /// Stores moderate positive integers only, typically 0 to 65,535. Good for
        /// larger counters or numeric codes that are always positive.
        ///
        /// @extends SQLTypeAbstract[int,int]
        #[derive(Debug, Clone, Copy)]
        pub struct [extends=PySQLTypeAbstract] PySmallUnsignedType as "SmallUnsignedType";
    )
    (
        /// Unsigned integer column type.
        ///
        /// Stores positive integers only, typically 0 to 4,294,967,295. Doubles the
        /// positive range compared to signed integers, useful for IDs and counters
        /// that will never be negative.
        ///
        /// @extends SQLTypeAbstract[int,int]
        #[derive(Debug, Clone, Copy)]
        pub struct [extends=PySQLTypeAbstract] PyUnsignedType as "UnsignedType";
    )
    (
        /// Unsigned big integer column type.
        ///
        /// Stores very large positive integers only. Provides the maximum positive
        /// integer range for high-volume systems or when very large positive
        /// values are required.
        ///
        /// @extends SQLTypeAbstract[int,int]
        #[derive(Debug, Clone, Copy)]
        pub struct [extends=PySQLTypeAbstract] PyBigUnsignedType as "BigUnsignedType";
    )
    (
        /// Single-precision floating point column type (FLOAT).
        ///
        /// Stores approximate numeric values with single precision. Suitable for
        /// scientific calculations, measurements, or any numeric data where some
        /// precision loss is acceptable in exchange for storage efficiency.
        ///
        /// @extends SQLTypeAbstract[float | int,float]
        #[derive(Debug, Clone, Copy)]
        pub struct [extends=PySQLTypeAbstract] PyFloatType as "FloatType";
    )
    (
        /// Double-precision floating point column type (DOUBLE).
        ///
        /// Stores approximate numeric values with double precision. Provides higher
        /// precision than FLOAT for scientific calculations or when more accuracy
        /// is required in floating-point operations.
        ///
        /// @extends SQLTypeAbstract[float | int,float]
        #[derive(Debug, Clone, Copy)]
        pub struct [extends=PySQLTypeAbstract] PyDoubleType as "DoubleType";
    )
    (
        /// Large text column type (TEXT).
        ///
        /// Represents a large text field capable of storing long strings without
        /// a predefined length limit. Suitable for storing articles, comments,
        /// descriptions, or any text content that may be very long.
        ///
        /// @extends SQLTypeAbstract[float | int,float]
        #[derive(Debug, Clone, Copy)]
        pub struct [extends=PySQLTypeAbstract] PyTextType as "TextType";
    )
    (
        /// Fixed-length character string column type (CHAR).
        ///
        /// Represents a fixed-length character string. Values shorter than the
        /// specified length are padded with spaces. Suitable for storing data
        /// with consistent, known lengths like country codes or status flags.
        ///
        /// @extends SQLTypeAbstract[str,str]
        /// @signature (length: int | None = ...)
        #[derive(Debug, Clone, Copy)]
        pub struct [extends=PySQLTypeAbstract] PyCharType as "CharType" (pub Option<u32>);
    )
    (
        /// Variable-length character string column type (VARCHAR).
        ///
        /// Represents a variable-length character string with a maximum length limit.
        /// This is the most common string type for storing text data of varying lengths
        /// like names, descriptions, or user input.
        ///
        /// @extends SQLTypeAbstract[str,str]
        /// @signature (length: int | None = ...)
        #[derive(Debug, Clone, Copy)]
        pub struct [extends=PySQLTypeAbstract] PyStringType as "StringType" (pub Option<u32>);
    )
}

impl NativeSQLType for PyBooleanType {
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::Boolean
    }

    unsafe fn validate(&self, py: pyo3::Python, ptr: *mut pyo3::ffi::PyObject) -> pyo3::PyResult<()> {
        if pyo3::ffi::PyBool_Check(ptr) != 1 {
            Err(typeerror!("expected bool, got {:?}", py, ptr))
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
            _ => invalid_value_for_deserialize!("bool", value),
        }
    }
}

macro_rules! implement_numeric_NativeSQLType {
    (
        $name:ident,
        $type:ty,
        $python_type_name:literal,
        $column_type:ident,
        $value_type:ident,
        $convertfunction:ident,
        $($checkfunction:ident,)+
    ) => {
        impl NativeSQLType for $name {
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

                Err(typeerror!(
                    concat!("expected ", $python_type_name, ", got {:?}"),
                    py,
                    ptr
                ))
            }

            unsafe fn deserialize(
                &self,
                py: pyo3::Python,
                value: &sea_query::Value,
            ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
                let val = match value {
                    sea_query::Value::BigInt(Some(x)) => pyo3::ffi::PyLong_FromLongLong(*x),
                    sea_query::Value::Int(Some(x)) => pyo3::ffi::PyLong_FromLong((*x) as i64),
                    sea_query::Value::SmallInt(Some(x)) => pyo3::ffi::PyLong_FromLong((*x) as i64),
                    sea_query::Value::TinyInt(Some(x)) => pyo3::ffi::PyLong_FromLong((*x) as i64),
                    sea_query::Value::BigUnsigned(Some(x)) => pyo3::ffi::PyLong_FromUnsignedLongLong(*x),
                    sea_query::Value::Unsigned(Some(x)) => pyo3::ffi::PyLong_FromUnsignedLong((*x) as u64),
                    sea_query::Value::SmallUnsigned(Some(x)) => {
                        pyo3::ffi::PyLong_FromUnsignedLong((*x) as u64)
                    }
                    sea_query::Value::TinyUnsigned(Some(x)) => {
                        pyo3::ffi::PyLong_FromUnsignedLong((*x) as u64)
                    }
                    sea_query::Value::Double(Some(x)) => pyo3::ffi::PyFloat_FromDouble(*x),
                    sea_query::Value::Float(Some(x)) => pyo3::ffi::PyFloat_FromDouble((*x) as f64),
                    sea_query::Value::BigInt(None)
                    | sea_query::Value::Int(None)
                    | sea_query::Value::SmallInt(None)
                    | sea_query::Value::TinyInt(None)
                    | sea_query::Value::BigUnsigned(None)
                    | sea_query::Value::Unsigned(None)
                    | sea_query::Value::SmallUnsigned(None)
                    | sea_query::Value::TinyUnsigned(None) => pyo3::ffi::Py_None(),
                    _ => return invalid_value_for_deserialize!("int", value),
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

implement_numeric_NativeSQLType!(
    PyBigIntegerType,
    i64,
    "int",
    BigInteger,
    BigInt,
    PyLong_CheckExact,
    PyLong_AsLongLong,
    PyFloat_CheckExact,
);
implement_numeric_NativeSQLType!(
    PyIntegerType,
    i32,
    "int",
    Integer,
    Int,
    PyLong_AsLongLong,
    PyLong_CheckExact,
    PyFloat_CheckExact,
);
implement_numeric_NativeSQLType!(
    PySmallIntegerType,
    i16,
    "int",
    SmallInteger,
    SmallInt,
    PyLong_AsLongLong,
    PyLong_CheckExact,
    PyFloat_CheckExact,
);
implement_numeric_NativeSQLType!(
    PyTinyIntegerType,
    i8,
    "int",
    TinyInteger,
    TinyInt,
    PyLong_AsLongLong,
    PyLong_CheckExact,
    PyFloat_CheckExact,
);
implement_numeric_NativeSQLType!(
    PyBigUnsignedType,
    u64,
    "int",
    BigUnsigned,
    BigUnsigned,
    PyLong_AsLongLong,
    PyLong_CheckExact,
    PyFloat_CheckExact,
);
implement_numeric_NativeSQLType!(
    PyUnsignedType,
    u32,
    "int",
    Unsigned,
    Unsigned,
    PyLong_AsLongLong,
    PyLong_CheckExact,
    PyFloat_CheckExact,
);
implement_numeric_NativeSQLType!(
    PySmallUnsignedType,
    u16,
    "int",
    SmallUnsigned,
    SmallUnsigned,
    PyLong_AsLongLong,
    PyLong_CheckExact,
    PyFloat_CheckExact,
);
implement_numeric_NativeSQLType!(
    PyTinyUnsignedType,
    u8,
    "int",
    TinyUnsigned,
    TinyUnsigned,
    PyLong_AsLongLong,
    PyLong_CheckExact,
    PyFloat_CheckExact,
);
implement_numeric_NativeSQLType!(
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
implement_numeric_NativeSQLType!(
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

impl NativeSQLType for PyTextType {
    #[inline(always)]
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::Text
    }

    unsafe fn validate(&self, py: pyo3::Python, ptr: *mut pyo3::ffi::PyObject) -> pyo3::PyResult<()> {
        if pyo3::ffi::PyUnicode_CheckExact(ptr) == 0 {
            Err(typeerror!("expected str, got {:?}", py, ptr))
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
            _ => invalid_value_for_deserialize!("str", value),
        }
    }
}

impl NativeSQLType for PyCharType {
    #[inline(always)]
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::Char(self.0)
    }

    unsafe fn validate(&self, py: pyo3::Python, ptr: *mut pyo3::ffi::PyObject) -> pyo3::PyResult<()> {
        if pyo3::ffi::PyUnicode_CheckExact(ptr) == 0 {
            Err(typeerror!("expected str, got {:?}", py, ptr))
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
            _ => invalid_value_for_deserialize!("str", value),
        }
    }
}

impl NativeSQLType for PyStringType {
    #[inline(always)]
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::String(
            self.0
                .map_or(sea_query::StringLen::None, |x| sea_query::StringLen::N(x)),
        )
    }

    unsafe fn validate(&self, py: pyo3::Python, ptr: *mut pyo3::ffi::PyObject) -> pyo3::PyResult<()> {
        if pyo3::ffi::PyUnicode_CheckExact(ptr) == 0 {
            Err(typeerror!("expected str, got {:?}", py, ptr))
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
            _ => invalid_value_for_deserialize!("str", value),
        }
    }
}

super::abstracts::implement_native_pymethods!(PyBooleanType);
super::abstracts::implement_native_pymethods!(PyBigIntegerType);
super::abstracts::implement_native_pymethods!(PyIntegerType);
super::abstracts::implement_native_pymethods!(PySmallIntegerType);
super::abstracts::implement_native_pymethods!(PyTinyIntegerType);
super::abstracts::implement_native_pymethods!(PyBigUnsignedType);
super::abstracts::implement_native_pymethods!(PyUnsignedType);
super::abstracts::implement_native_pymethods!(PySmallUnsignedType);
super::abstracts::implement_native_pymethods!(PyTinyUnsignedType);
super::abstracts::implement_native_pymethods!(PyFloatType);
super::abstracts::implement_native_pymethods!(PyDoubleType);
super::abstracts::implement_native_pymethods!(PyTextType);
super::abstracts::implement_native_pymethods!(
    PyCharType,
    init(|length: Option<u32>| Self(length)),
    "int | None",
    signature(length = None)
);
super::abstracts::implement_native_pymethods!(
    PyStringType,
    init(|length: Option<u32>| Self(length)),
    "int | None",
    signature(length = None)
);
