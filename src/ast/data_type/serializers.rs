use pyo3::types::PyAnyMethods;
use pyo3::IntoPyObjectExt;

use crate::internal::alias;
use std::sync::Arc;

/// A value serializer: `PyObject -> String` in SQL-literal form.
pub trait Serialize {
    /// Convert a *borrowed* Python object into a SQL literal string.
    ///
    /// ### Safety
    /// - `ptr` must be a valid borrowed pointer.
    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
        _dialect: &crate::dialect::DialectUnion,
    ) -> pyo3::PyResult<String>;
}

/// SQL data-type value serializers.
///
/// One variant per *distinct serialization behaviour* - many SQL data types
/// collapse onto the same serializer.
#[derive(Debug)]
pub enum DataTypeSerializerUnion {
    /// Every signed/unsigned integer width.
    ///
    /// Supports:
    /// - `TinyInt`
    /// - `TinyIntUnsigned`
    /// - `UTinyInt`
    /// - `Int2`
    /// - `Int2Unsigned`
    /// - `SmallInt`
    /// - `SmallIntUnsigned`
    /// - `USmallInt`
    /// - `MediumInt`
    /// - `MediumIntUnsigned`
    /// - `Int`
    /// - `Int4`
    /// - `Int8`
    /// - `Integer`
    /// - `IntUnsigned`
    /// - `Int4Unsigned`
    /// - `BigInt`
    /// - `BigIntUnsigned`
    /// - `Int8Unsigned`
    /// - `UInt8`
    /// - `UInt16`
    /// - `UInt32`
    /// - `UInt64`
    /// - `UInt128`
    /// - `UInt256`
    /// - `Int128`
    /// - `Int256`
    /// - `UBigInt`
    /// - `UInt`
    Integer(PyIntegerSerializer),
    /// Every float **and** decimal/numeric type (FLOAT, DOUBLE, DECIMAL, …).
    ///
    /// Supports:
    /// - `Numeric`
    /// - `Decimal`
    /// - `DecimalUnsigned`
    /// - `BigNumeric`
    /// - `BigDecimal`
    /// - `Dec`
    /// - `DecUnsigned`
    /// - `Float`
    /// - `FloatUnsigned`
    /// - `Real`
    /// - `RealUnsigned`
    /// - `Float4`
    /// - `Float8`
    /// - `Double`
    /// - `DoubleUnsigned`
    /// - `DoublePrecision`
    /// - `DoublePrecisionUnsigned`
    Float(PyFloatSerializer),
    /// BOOLEAN / BOOL.
    ///
    /// Supports:
    /// - `Bool`
    /// - `Boolean`
    Boolean(PyBooleanSerializer),
    /// Any character/text type (CHAR, VARCHAR, TEXT, CLOB, ENUM, SET, …).
    ///
    /// Supports:
    /// - `Character`
    /// - `Char`
    /// - `CharacterVarying`
    /// - `CharVarying`
    /// - `Varchar`
    /// - `Nvarchar`
    /// - `CharacterLargeObject`
    /// - `CharLargeObject`
    /// - `Clob`
    /// - `String`
    /// - `FixedString`
    /// - `Regclass`
    /// - `Enum`
    /// - `Set`
    String(PyStringSerializer),
    /// UUID.
    ///
    /// Supports:
    /// - `UUID`
    Uuid(PyUuidSerializer),
    /// Binary blobs (BINARY, VARBINARY, BLOB, BYTEA, BYTES, …).
    ///
    /// Supports:
    /// - `Binary`
    /// - `Varbinary`
    /// - `Blob`
    /// - `LongBlob`
    /// - `Bytes`
    /// - `Bytea`
    Bytes(PyBytesSerializer),
    /// Bit strings (BIT, BIT VARYING, VARBIT).
    ///
    /// Supports:
    /// - `Bit`
    /// - `BitVarying`
    /// - `VarBit`
    Bit(PyBitSerializer),
    /// JSON / JSONB.
    ///
    /// Supports:
    /// - `JSON`
    /// - `JSONB`
    JSON(PyJSONSerializer),
    /// DATE.
    ///
    /// Supports:
    /// - `Date`
    /// - `Date32`
    Date(PyDateSerializer),
    /// TIME.
    ///
    /// Supports:
    /// - `Time`
    Time(PyTimeSerializer),
    /// DATETIME / TIMESTAMP / DATETIME64.
    ///
    /// Supports:
    /// - `Datetime`
    /// - `Datetime64`
    /// - `Timestamp`
    /// - `TimestampNtz`
    /// - `TimestampWithTz`
    Datetime(PyDatetimeSerializer),
    /// INET.
    ///
    /// Supports:
    /// - `Inet`
    Inet(PyInetSerializer),
    /// MACADDR.
    ///
    /// Supports:
    /// - `MacAddr`
    MacAddress(PyMacAddressSerializer),
    /// ARRAY of a single element type.
    ///
    /// Supports:
    /// - `Array`
    Array(PyArraySerializer),
    /// Custom serializer
    Custom(alias::PyObject),
    /// Unspecified serializer.
    ///
    /// This is also useful when user doesn't specify the serializer,
    /// and also we couldn't detect the type from datatype.
    Unspecified(PyUnspecifiedSerializer),
}

impl DataTypeSerializerUnion {
    pub fn infer_from_name(value: &super::name::DataTypeName) -> Self {
        use super::name::DataTypeGroup;

        match value.group() {
            DataTypeGroup::STRING => Self::String(PyStringSerializer),
            DataTypeGroup::UUID => Self::Uuid(PyUuidSerializer),
            DataTypeGroup::BYTES => Self::Bytes(PyBytesSerializer),
            DataTypeGroup::BIT => Self::Bit(PyBitSerializer),
            DataTypeGroup::INTEGER => Self::Integer(PyIntegerSerializer),
            DataTypeGroup::FLOAT => Self::Float(PyFloatSerializer),
            DataTypeGroup::BOOLEAN => Self::Boolean(PyBooleanSerializer),
            DataTypeGroup::DATE => Self::Date(PyDateSerializer),
            DataTypeGroup::TIME => Self::Time(PyTimeSerializer),
            DataTypeGroup::DATETIME => Self::Datetime(PyDatetimeSerializer),
            DataTypeGroup::JSON | DataTypeGroup::JSONB => Self::JSON(PyJSONSerializer),
            DataTypeGroup::INET => Self::Inet(PyInetSerializer),
            DataTypeGroup::MACADDRESS => Self::MacAddress(PyMacAddressSerializer),
            DataTypeGroup::CUSTOM => Self::Unspecified(PyUnspecifiedSerializer),
        }
    }

    pub fn to_pyobject<'py>(
        &self,
        py: pyo3::Python<'py>,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        match self {
            Self::Integer(s) => s.into_bound_py_any(py),
            Self::Float(s) => s.into_bound_py_any(py),
            Self::Boolean(s) => s.into_bound_py_any(py),
            Self::String(s) => s.into_bound_py_any(py),
            Self::Uuid(s) => s.into_bound_py_any(py),
            Self::Bytes(s) => s.into_bound_py_any(py),
            Self::Bit(s) => s.into_bound_py_any(py),
            Self::JSON(s) => s.into_bound_py_any(py),
            Self::Date(s) => s.into_bound_py_any(py),
            Self::Time(s) => s.into_bound_py_any(py),
            Self::Datetime(s) => s.into_bound_py_any(py),
            Self::Inet(s) => s.into_bound_py_any(py),
            Self::MacAddress(s) => s.into_bound_py_any(py),
            Self::Array(s) => s.clone().into_bound_py_any(py),
            Self::Custom(s) => Ok(s.bind(py).clone()),
            Self::Unspecified(s) => s.into_bound_py_any(py),
        }
    }
}

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for DataTypeSerializerUnion {
    type Error = pyo3::PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: pyo3::inspect::PyStaticExpr = type_hint_union!(
        type_hint_identifier!("'ArraySerializer'"),
        type_hint_identifier!("'BitSerializer'"),
        type_hint_identifier!("'BooleanSerializer'"),
        type_hint_identifier!("'BytesSerializer'"),
        type_hint_identifier!("'DateSerializer'"),
        type_hint_identifier!("'DatetimeSerializer'"),
        type_hint_identifier!("'FloatSerializer'"),
        type_hint_identifier!("'InetSerializer'"),
        type_hint_identifier!("'IntegerSerializer'"),
        type_hint_identifier!("'JSONSerializer'"),
        type_hint_identifier!("'MacAddressSerializer'"),
        type_hint_identifier!("'StringSerializer'"),
        type_hint_identifier!("'TimeSerializer'"),
        type_hint_identifier!("'UnspecifiedSerializer'"),
        type_hint_identifier!("'UuidSerializer'"),
        type_hint_subscript!(
            type_hint_identifier!("typing", "Callable"),
            pyo3::inspect::PyStaticExpr::List {
                elts: &[
                    type_hint_union!(
                        type_hint_identifier!("rapidquery._lib.dialect", "BigQueryDialect"),
                        type_hint_identifier!("rapidquery._lib.dialect", "DatabricksDialect"),
                        type_hint_identifier!("rapidquery._lib.dialect", "DuckDbDialect"),
                        type_hint_identifier!("rapidquery._lib.dialect", "HiveDialect"),
                        type_hint_identifier!("rapidquery._lib.dialect", "MsSqlDialect"),
                        type_hint_identifier!("rapidquery._lib.dialect", "MySqlDialect"),
                        type_hint_identifier!("rapidquery._lib.dialect", "OracleDialect"),
                        type_hint_identifier!("rapidquery._lib.dialect", "PostgreSqlDialect"),
                        type_hint_identifier!("rapidquery._lib.dialect", "RedshiftDialect"),
                        type_hint_identifier!("rapidquery._lib.dialect", "SnowflakeDialect"),
                        type_hint_identifier!("rapidquery._lib.dialect", "SparkDialect"),
                        type_hint_identifier!("rapidquery._lib.dialect", "SQLiteDialect"),
                        type_hint_identifier!("rapidquery._lib.dialect", "TeradataDialect")
                    ),
                    type_hint_identifier!("str")
                ]
            },
            type_hint_identifier!("str")
        )
    );

    fn extract(
        obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>,
    ) -> Result<Self, Self::Error> {
        unsafe {
            let ptr = obj.as_ptr();
            let type_ptr = pyo3::ffi::Py_TYPE(ptr);

            if type_ptr == crate::typeref::INTEGER_SERIALIZER_TYPE {
                return Ok(Self::Integer(PyIntegerSerializer));
            }
            if type_ptr == crate::typeref::FLOAT_SERIALIZER_TYPE {
                return Ok(Self::Float(PyFloatSerializer));
            }
            if type_ptr == crate::typeref::BOOLEAN_SERIALIZER_TYPE {
                return Ok(Self::Boolean(PyBooleanSerializer));
            }
            if type_ptr == crate::typeref::STRING_SERIALIZER_TYPE {
                return Ok(Self::String(PyStringSerializer));
            }
            if type_ptr == crate::typeref::UUID_SERIALIZER_TYPE {
                return Ok(Self::Uuid(PyUuidSerializer));
            }
            if type_ptr == crate::typeref::BYTES_SERIALIZER_TYPE {
                return Ok(Self::Bytes(PyBytesSerializer));
            }
            if type_ptr == crate::typeref::BIT_SERIALIZER_TYPE {
                return Ok(Self::Bit(PyBitSerializer));
            }
            if type_ptr == crate::typeref::JSON_SERIALIZER_TYPE {
                return Ok(Self::JSON(PyJSONSerializer));
            }
            if type_ptr == crate::typeref::DATE_SERIALIZER_TYPE {
                return Ok(Self::Date(PyDateSerializer));
            }
            if type_ptr == crate::typeref::TIME_SERIALIZER_TYPE {
                return Ok(Self::Time(PyTimeSerializer));
            }
            if type_ptr == crate::typeref::DATETIME_SERIALIZER_TYPE {
                return Ok(Self::Datetime(PyDatetimeSerializer));
            }
            if type_ptr == crate::typeref::INET_SERIALIZER_TYPE {
                return Ok(Self::Inet(PyInetSerializer));
            }
            if type_ptr == crate::typeref::MAC_ADDRESS_SERIALIZER_TYPE {
                return Ok(Self::MacAddress(PyMacAddressSerializer));
            }
            if type_ptr == crate::typeref::ARRAY_SERIALIZER_TYPE {
                let array_ref = obj.cast_unchecked::<PyArraySerializer>();
                let element = array_ref.get().element.clone();

                return Ok(Self::Array(PyArraySerializer { element }));
            }
            if type_ptr == crate::typeref::UNSPECIFIED_SERIALIZER_TYPE {
                return Ok(Self::Unspecified(PyUnspecifiedSerializer));
            }

            // Fall through: treat as a custom callable serializer.
            let callable = obj.extract::<alias::PyObject>()?;
            Ok(Self::Custom(callable))
        }
    }
}

impl Serialize for DataTypeSerializerUnion {
    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
        dialect: &crate::dialect::DialectUnion,
    ) -> pyo3::PyResult<String> {
        match self {
            Self::Integer(s) => s.serialize(py, ptr, dialect),
            Self::Float(s) => s.serialize(py, ptr, dialect),
            Self::Boolean(s) => s.serialize(py, ptr, dialect),
            Self::String(s) => s.serialize(py, ptr, dialect),
            Self::Uuid(s) => s.serialize(py, ptr, dialect),
            Self::Bytes(s) => s.serialize(py, ptr, dialect),
            Self::Bit(s) => s.serialize(py, ptr, dialect),
            Self::JSON(s) => s.serialize(py, ptr, dialect),
            Self::Date(s) => s.serialize(py, ptr, dialect),
            Self::Time(s) => s.serialize(py, ptr, dialect),
            Self::Datetime(s) => s.serialize(py, ptr, dialect),
            Self::Inet(s) => s.serialize(py, ptr, dialect),
            Self::MacAddress(s) => s.serialize(py, ptr, dialect),
            Self::Array(s) => s.serialize(py, ptr, dialect),
            Self::Custom(s) => {
                let object_arg = pyo3::Bound::from_borrowed_ptr(py, ptr);
                let dialect_arg = dialect.name();
                let output = s.call1(py, (object_arg, dialect_arg))?;
                _serialize_string(py, output.as_ptr())
            }
            Self::Unspecified(s) => s.serialize(py, ptr, dialect),
        }
    }
}

implement_pyclass! {
    /// Serializes any Python `int` as a bare decimal literal.
    ///
    /// Uses `str(int)` so arbitrary precision (beyond i64/u64) is preserved - one
    /// serializer genuinely covers TINYINT through BIGINT and the unsigned variants.
    #[derive(Debug, Clone, Copy, Default)]
    [skip_from_py_object] PyIntegerSerializer as "IntegerSerializer";
}

#[pyo3::pymethods]
impl PyIntegerSerializer {
    #[new]
    fn __new__() -> Self {
        Self
    }
}

impl Serialize for PyIntegerSerializer {
    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
        _dialect: &crate::dialect::DialectUnion,
    ) -> pyo3::PyResult<String> {
        if pyo3::ffi::PyLong_CheckExact(ptr) == 1 {
            // bool is a subclass of int in Python, but PyLong_CheckExact rejects
            // it, so True/False never sneaks through here.
            py_to_string(py, ptr)
        } else {
            Err(new_py_error!(
                PyTypeError,
                "expected int for integer serialization, got {}",
                crate::internal::utils::get_type_name(py, ptr)
            ))
        }
    }
}

implement_pyclass! {
    /// Serializes Python `float`, `int`, `decimal.Decimal`, or numeric `str` as a
    /// bare numeric literal.
    ///
    /// `Decimal`/`str`/`int` go through their exact text form (no precision loss);
    /// only a real `float` is formatted from the f64. Non-finite floats are rejected.
    #[derive(Debug, Clone, Copy, Default)]
    [skip_from_py_object] PyFloatSerializer as "FloatSerializer";
}

#[pyo3::pymethods]
impl PyFloatSerializer {
    #[new]
    fn __new__() -> Self {
        Self
    }
}

impl Serialize for PyFloatSerializer {
    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
        _dialect: &crate::dialect::DialectUnion,
    ) -> pyo3::PyResult<String> {
        if pyo3::ffi::PyFloat_CheckExact(ptr) == 1 {
            let v = pyo3::ffi::PyFloat_AsDouble(ptr);
            if !v.is_finite() {
                return Err(new_py_error!(
                    PyValueError,
                    "cannot serialize non-finite float {} as a numeric literal",
                    v
                ));
            }

            // {:?} on f64 round-trips and avoids the "1" vs "1.0" ambiguity.
            return Ok(format!("{v:?}"));
        }

        // int / Decimal / str: their textual form is already a valid literal.
        if pyo3::ffi::PyLong_CheckExact(ptr) == 1
            || pyo3::ffi::PyUnicode_CheckExact(ptr) == 1
            || pyo3::ffi::Py_TYPE(ptr) == crate::typeref::STD_DECIMAL_TYPE
        {
            let text = py_to_string(py, ptr)?;
            // Reject obvious garbage from str inputs cheaply: a literal must
            // start with a digit, sign, or dot.
            if pyo3::ffi::PyUnicode_CheckExact(ptr) == 1
                && !text
                    .trim()
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_digit() || c == '-' || c == '+' || c == '.')
            {
                return Err(new_py_error!(
                    PyValueError,
                    "string {:?} is not a valid numeric literal",
                    text
                ));
            }
            return Ok(text.trim().to_string());
        }

        Err(new_py_error!(
            PyTypeError,
            "expected float/int/Decimal/str for numeric serialization, got {}",
            crate::internal::utils::get_type_name(py, ptr)
        ))
    }
}

implement_pyclass! {
    #[derive(Debug, Clone, Copy, Default)]
    [skip_from_py_object] PyBooleanSerializer as "BooleanSerializer";
}

#[pyo3::pymethods]
impl PyBooleanSerializer {
    #[new]
    fn __new__() -> Self {
        Self
    }
}

impl Serialize for PyBooleanSerializer {
    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
        dialect: &crate::dialect::DialectUnion,
    ) -> pyo3::PyResult<String> {
        if pyo3::ffi::PyBool_Check(ptr) == 1 {
            let value = pyo3::ffi::Py_True() == ptr;
            Ok(dialect.bool_format().format(value).to_string())
        } else {
            Err(new_py_error!(
                PyTypeError,
                "expected bool for boolean serialization, got {}",
                crate::internal::utils::get_type_name(py, ptr)
            ))
        }
    }
}

implement_pyclass! {
    /// Serializes a Python `str` (or `enum.Enum` whose value is a str) as a quoted,
    /// escaped string literal: `"LIKE"` -> `'LIKE'`.
    #[derive(Debug, Clone, Copy, Default)]
    [skip_from_py_object] PyStringSerializer as "StringSerializer";
}

#[pyo3::pymethods]
impl PyStringSerializer {
    #[new]
    fn __new__() -> Self {
        Self
    }
}

impl Serialize for PyStringSerializer {
    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
        dialect: &crate::dialect::DialectUnion,
    ) -> pyo3::PyResult<String> {
        if pyo3::ffi::PyUnicode_CheckExact(ptr) == 1 {
            let raw = _serialize_string(py, ptr)?;
            let raw = dialect.escape_string(&raw);
            return Ok(dialect.quote_string(&raw));
        }

        // Accept enum members whose `.value` is a string (mirrors PyEnumType).
        if pyo3::ffi::Py_TYPE(ptr) == crate::typeref::STD_ENUM_TYPE {
            let value = pyo3::ffi::PyObject_GetAttrString(ptr, c"value".as_ptr());
            if value.is_null() {
                return Err(pyo3::PyErr::fetch(py));
            }

            let is_str = pyo3::ffi::PyUnicode_CheckExact(ptr) == 1;
            let raw = if is_str {
                _serialize_string(py, value)
            } else {
                Ok(String::new())
            };

            pyo3::ffi::Py_DECREF(value);
            if is_str {
                let raw = dialect.escape_string(&raw?);
                return Ok(dialect.quote_string(&raw));
            }
        }

        Err(new_py_error!(
            PyTypeError,
            "expected str/Enum for string serialization, got {}",
            crate::internal::utils::get_type_name(py, ptr)
        ))
    }
}

implement_pyclass! {
    /// Serializes a `uuid.UUID` (or 36-char `str`) as a quoted literal.
    #[derive(Debug, Clone, Copy, Default)]
    [skip_from_py_object] PyUuidSerializer as "UuidSerializer";
}

#[pyo3::pymethods]
impl PyUuidSerializer {
    #[new]
    fn __new__() -> Self {
        Self
    }
}

impl Serialize for PyUuidSerializer {
    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
        dialect: &crate::dialect::DialectUnion,
    ) -> pyo3::PyResult<String> {
        if pyo3::ffi::Py_TYPE(ptr) == crate::typeref::STD_UUID_TYPE
            || pyo3::ffi::PyUnicode_CheckExact(ptr) == 1
        {
            let text = py_to_string(py, ptr)?;
            let text = dialect.escape_string(&text);
            return Ok(dialect.quote_string(&text));
        } else {
            Err(new_py_error!(
                PyTypeError,
                "expected uuid.UUID/str for UUID serialization, got {}",
                crate::internal::utils::get_type_name(py, ptr)
            ))
        }
    }
}

implement_pyclass! {
    /// Serializes a `uuid.UUID` (or 36-char `str`) as a quoted literal.
    #[derive(Debug, Clone, Copy, Default)]
    [skip_from_py_object] PyBytesSerializer as "BytesSerializer";
}

#[pyo3::pymethods]
impl PyBytesSerializer {
    #[new]
    fn __new__() -> Self {
        Self
    }
}

impl Serialize for PyBytesSerializer {
    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
        dialect: &crate::dialect::DialectUnion,
    ) -> pyo3::PyResult<String> {
        if pyo3::ffi::PyBytes_CheckExact(ptr) == 1 {
            Ok(dialect.encode_bytes(read_bytes(ptr)))
        } else {
            Err(new_py_error!(
                PyTypeError,
                "expected bytes for binary serialization, got {}",
                crate::internal::utils::get_type_name(py, ptr)
            ))
        }
    }
}

implement_pyclass! {
    /// Serializes a bit string given as a `str` of `0`/`1` (e.g. `"1010"`) into a
    /// `B'1010'` literal.
    #[derive(Debug, Clone, Copy, Default)]
    [skip_from_py_object] PyBitSerializer as "BitSerializer";
}

#[pyo3::pymethods]
impl PyBitSerializer {
    #[new]
    fn __new__() -> Self {
        Self
    }
}

impl Serialize for PyBitSerializer {
    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
        dialect: &crate::dialect::DialectUnion,
    ) -> pyo3::PyResult<String> {
        if pyo3::ffi::PyUnicode_CheckExact(ptr) == 1 {
            let raw = _serialize_string(py, ptr)?;

            if raw.is_empty() || !raw.bytes().all(|b| b == b'0' || b == b'1') {
                return Err(new_py_error!(
                    PyValueError,
                    "expected a string of '0'/'1' for bit serialization, got {:?}",
                    raw
                ));
            }
            Ok(dialect.encode_bits(&raw))
        } else {
            Err(new_py_error!(
                PyTypeError,
                "expected str of bits for bit serialization, got {}",
                crate::internal::utils::get_type_name(py, ptr)
            ))
        }
    }
}

implement_pyclass! {
    /// Serializes any JSON-encodable Python object via `json.dumps`, then wraps the
    /// result as a quoted string literal.
    #[derive(Debug, Clone, Copy, Default)]
    [skip_from_py_object] PyJSONSerializer as "JSONSerializer";
}

#[pyo3::pymethods]
impl PyJSONSerializer {
    #[new]
    fn __new__() -> Self {
        Self
    }
}

impl Serialize for PyJSONSerializer {
    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
        dialect: &crate::dialect::DialectUnion,
    ) -> pyo3::PyResult<String> {
        // Reuse the existing json-dumps helper from sqltypes::json.
        let dumped = _serialize_object_with_pyjson(py, ptr)?;
        let text = _serialize_string(py, dumped);
        pyo3::ffi::Py_DECREF(dumped);

        let text = dialect.escape_string(&text?);
        Ok(dialect.quote_string(&text))
    }
}

implement_pyclass! {
    /// `datetime.date` -> `'YYYY-MM-DD'`.
    #[derive(Debug, Clone, Copy, Default)]
    [skip_from_py_object] PyDateSerializer as "DateSerializer";
}

#[pyo3::pymethods]
impl PyDateSerializer {
    #[new]
    fn __new__() -> Self {
        Self
    }
}

impl Serialize for PyDateSerializer {
    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
        dialect: &crate::dialect::DialectUnion,
    ) -> pyo3::PyResult<String> {
        if pyo3::ffi::Py_TYPE(ptr) == crate::typeref::STD_DATE_TYPE {
            let val: pyo3::Bound<'_, pyo3::types::PyDate> =
                pyo3::Bound::from_borrowed_ptr(py, ptr).cast_into()?;

            let date: chrono::NaiveDate = val.extract()?;
            let mut result = date.format("%Y-%m-%d").to_string();

            dialect.quote_string(&mut result);
            Ok(result)
        } else {
            Err(new_py_error!(
                PyTypeError,
                "expected datetime.date for date serialization, got {}",
                crate::internal::utils::get_type_name(py, ptr)
            ))
        }
    }
}

implement_pyclass! {
    /// `datetime.time` -> `'HH:MM:SS[.ffffff]'`.
    #[derive(Debug, Clone, Copy, Default)]
    [skip_from_py_object] PyTimeSerializer as "TimeSerializer";
}

#[pyo3::pymethods]
impl PyTimeSerializer {
    #[new]
    fn __new__() -> Self {
        Self
    }
}

impl Serialize for PyTimeSerializer {
    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
        dialect: &crate::dialect::DialectUnion,
    ) -> pyo3::PyResult<String> {
        if pyo3::ffi::Py_TYPE(ptr) == crate::typeref::STD_TIME_TYPE {
            let val: pyo3::Bound<'_, pyo3::types::PyTime> =
                pyo3::Bound::from_borrowed_ptr(py, ptr).cast_into()?;

            let date: chrono::NaiveTime = val.extract()?;
            let mut result = date.format("%H:%M:%S%.f").to_string();

            dialect.quote_string(&mut result);
            Ok(result)
        } else {
            Err(new_py_error!(
                PyTypeError,
                "expected datetime.time for time serialization, got {}",
                crate::internal::utils::get_type_name(py, ptr)
            ))
        }
    }
}

implement_pyclass! {
    /// `datetime.datetime` -> `'YYYY-MM-DD HH:MM:SS[.ffffff][+HH:MM]'`.
    ///
    /// `int`/`float` are treated as a UTC epoch and accepted too.
    #[derive(Debug, Clone, Copy, Default)]
    [skip_from_py_object] PyDatetimeSerializer as "DatetimeSerializer";
}

#[pyo3::pymethods]
impl PyDatetimeSerializer {
    #[new]
    fn __new__() -> Self {
        Self
    }
}

impl Serialize for PyDatetimeSerializer {
    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
        dialect: &crate::dialect::DialectUnion,
    ) -> pyo3::PyResult<String> {
        use chrono::TimeZone;
        use pyo3::types::PyTzInfoAccess;

        if pyo3::ffi::Py_TYPE(ptr) == crate::typeref::STD_DATETIME_TYPE {
            // isoformat(sep=' ') -> the SQL-friendly space-separated form.
            let val: pyo3::Bound<'_, pyo3::types::PyDateTime> =
                pyo3::Bound::from_borrowed_ptr(py, ptr).cast_into()?;

            let tzinfo = val.get_tzinfo();

            if tzinfo.is_none() {
                let datetime: chrono::DateTime<chrono::Utc> = val.extract()?;
                let mut result = datetime.format("%Y-%m-%d %H:%M:%S%.f%:z").to_string();

                dialect.quote_string(&mut result);
                return Ok(result);
            } else {
                let datetime: chrono::NaiveDateTime = val.extract()?;
                let mut result = datetime.format("%Y-%m-%d %H:%M:%S%.f").to_string();

                dialect.quote_string(&mut result);
                return Ok(result);
            }
        }

        if pyo3::ffi::PyLong_CheckExact(ptr) == 1 {
            let mut has_overflow: i32 = 0;
            let num = pyo3::ffi::PyLong_AsLongLongAndOverflow(ptr, &mut has_overflow);

            if has_overflow == 1 {
                return Err(pyo3::exceptions::PyOverflowError::new_err("out of range"));
            }

            if let chrono::offset::LocalResult::Single(datetime) = chrono::Utc.timestamp_opt(num, 0)
            {
                let mut result = datetime.format("%Y-%m-%d %H:%M:%S%.f%:z").to_string();
                dialect.quote_string(&mut result);
                return Ok(result);
            }

            return Err(new_py_error!(
                PyOverflowError,
                "timestamp is invalid for datetime serialization: {}",
                num
            ));
        }

        if pyo3::ffi::PyFloat_CheckExact(ptr) == 1 {
            let num = pyo3::ffi::PyFloat_AsDouble(ptr);

            if let chrono::offset::LocalResult::Single(datetime) = chrono::Utc
                .timestamp_opt(num.trunc() as i64, (num.fract() * 1_000_000_000.0) as u32)
            {
                let mut result = datetime.format("%Y-%m-%d %H:%M:%S%.f%:z").to_string();
                dialect.quote_string(&mut result);
                return Ok(result);
            }

            return Err(new_py_error!(
                PyOverflowError,
                "timestamp is invalid for datetime serialization: {}",
                num
            ));
        }

        Err(new_py_error!(
            PyTypeError,
            "expected datetime.datetime/int/float for datetime serialization, got {}",
            crate::internal::utils::get_type_name(py, ptr)
        ))
    }
}

implement_pyclass! {
    /// `str` (or `ipaddress.*`) -> quoted literal. Validated as IPv4/IPv6 network.
    #[derive(Debug, Clone, Copy, Default)]
    [skip_from_py_object] PyInetSerializer as "InetSerializer";
}

#[pyo3::pymethods]
impl PyInetSerializer {
    #[new]
    fn __new__() -> Self {
        Self
    }
}

impl Serialize for PyInetSerializer {
    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
        dialect: &crate::dialect::DialectUnion,
    ) -> pyo3::PyResult<String> {
        use std::str::FromStr;

        let text = py_to_string(py, ptr)?;
        let mut net = ipnetwork::IpNetwork::from_str(&text)
            .map(|net| net.to_string())
            .map_err(|e| new_py_error!(PyValueError, e.to_string()))?;

        dialect.quote_string(&mut net);
        return Ok(net);
    }
}

implement_pyclass! {
    /// `str` -> quoted literal. Validated as a MAC address.
    #[derive(Debug, Clone, Copy, Default)]
    [skip_from_py_object] PyMacAddressSerializer as "MacAddressSerializer";
}

#[pyo3::pymethods]
impl PyMacAddressSerializer {
    #[new]
    fn __new__() -> Self {
        Self
    }
}

impl Serialize for PyMacAddressSerializer {
    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
        dialect: &crate::dialect::DialectUnion,
    ) -> pyo3::PyResult<String> {
        use std::str::FromStr;
        let text = py_to_string(py, ptr)?;

        let mut addr = mac_address::MacAddress::from_str(&text)
            .map(|mac| mac.to_string())
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

        dialect.quote_string(&mut addr);
        return Ok(addr);
    }
}

implement_pyclass! {
    /// Serializes a Python `list`/`tuple` by serializing each element with `element`
    /// and wrapping in `ARRAY[...]`.
    #[derive(Debug, Clone)]
    [skip_from_py_object] PyArraySerializer as "ArraySerializer" {
        pub element: Arc<DataTypeSerializerUnion>,
    }
}

#[pyo3::pymethods]
impl PyArraySerializer {
    #[new]
    fn __new__(element: DataTypeSerializerUnion) -> Self {
        Self {
            element: Arc::new(element),
        }
    }
}

impl Serialize for PyArraySerializer {
    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
        dialect: &crate::dialect::DialectUnion,
    ) -> pyo3::PyResult<String> {
        let (len, getter): (
            isize,
            unsafe fn(*mut pyo3::ffi::PyObject, isize) -> *mut pyo3::ffi::PyObject,
        ) = if pyo3::ffi::PyList_CheckExact(ptr) == 1 {
            (pyo3::ffi::PyList_Size(ptr), |p, i| {
                pyo3::ffi::PyList_GetItem(p, i)
            })
        } else if pyo3::ffi::PyTuple_CheckExact(ptr) == 1 {
            (pyo3::ffi::PyTuple_Size(ptr), |p, i| {
                pyo3::ffi::PyTuple_GetItem(p, i)
            })
        } else {
            return Err(new_py_error!(
                PyTypeError,
                "expected list/tuple for array serialization, got {}",
                crate::internal::utils::get_type_name(py, ptr)
            ));
        };

        let mut parts: Vec<String> = Vec::with_capacity(len.max(0) as usize);
        for i in 0..len {
            let item = getter(ptr, i); // borrowed
            if item.is_null() {
                return Err(pyo3::PyErr::fetch(py));
            }
            parts.push(self.element.serialize(py, item, dialect)?);
        }

        Ok(format!("ARRAY[{}]", parts.join(", ")))
    }
}

implement_pyclass! {
    /// When you don't know about data type, you can use this. Uses values' `__str__` method
    /// and return it as a serialized SQL value.
    #[derive(Debug, Clone, Copy, Default)]
    [skip_from_py_object] PyUnspecifiedSerializer as "UnspecifiedSerializer";
}

#[pyo3::pymethods]
impl PyUnspecifiedSerializer {
    #[new]
    fn __new__() -> Self {
        Self
    }
}

impl Serialize for PyUnspecifiedSerializer {
    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
        _dialect: &crate::dialect::DialectUnion,
    ) -> pyo3::PyResult<String> {
        py_to_string(py, ptr)
    }
}

/// Converts Python strings into [`String`], in the fastest way.
#[inline]
unsafe fn _serialize_string(
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

/// `str(obj)` then read it as UTF-8. Used for arbitrary-precision ints,
/// `decimal.Decimal`, `uuid.UUID`, etc. where Python already has the right text.
#[inline]
unsafe fn py_to_string(py: pyo3::Python, ptr: *mut pyo3::ffi::PyObject) -> pyo3::PyResult<String> {
    let s = pyo3::ffi::PyObject_Str(ptr);
    if s.is_null() {
        return Err(pyo3::PyErr::fetch(py));
    }
    let out = _serialize_string(py, s);
    pyo3::ffi::Py_DECREF(s);
    out
}

/// Borrow the bytes of a Python `bytes` object.
#[inline]
unsafe fn read_bytes<'a>(ptr: *mut pyo3::ffi::PyObject) -> &'a [u8] {
    let buf = pyo3::ffi::PyBytes_AsString(ptr) as *const u8;
    let len = pyo3::ffi::PyBytes_Size(ptr) as usize;
    debug_assert!(!buf.is_null());
    std::slice::from_raw_parts(buf, len)
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
/// # Safety
/// - `ptr` should be borrowed and a valid pointer
#[inline(always)]
pub unsafe fn _serialize_object_with_pyjson(
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
