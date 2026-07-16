use pyo3::types::PyAnyMethods;

use crate::internal::alias;
use crate::internal::utils::get_type_name;

/// SQL data-type value serializers.
///
/// One variant per *distinct serialization behaviour* - many SQL data types
/// collapse onto the same serializer.
#[derive(Debug, Clone)]
pub enum DataTypeSerializer {
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
    Integer,
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
    Float,
    /// BOOLEAN / BOOL.
    ///
    /// Supports:
    /// - `Bool`
    /// - `Boolean`
    Boolean,
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
    String,
    /// UUID.
    ///
    /// Supports:
    /// - `UUID`
    Uuid,
    /// Binary blobs (BINARY, VARBINARY, BLOB, BYTEA, BYTES, …).
    ///
    /// Supports:
    /// - `Binary`
    /// - `Varbinary`
    /// - `Blob`
    /// - `LongBlob`
    /// - `Bytes`
    /// - `Bytea`
    Bytes,
    /// Bit strings (BIT, BIT VARYING, VARBIT).
    ///
    /// Supports:
    /// - `Bit`
    /// - `BitVarying`
    /// - `VarBit`
    Bit,
    /// JSON / JSONB.
    ///
    /// Supports:
    /// - `JSON`
    /// - `JSONB`
    JSON,
    /// DATE.
    ///
    /// Supports:
    /// - `Date`
    /// - `Date32`
    Date,
    /// TIME.
    ///
    /// Supports:
    /// - `Time`
    Time,
    /// DATETIME / TIMESTAMP / DATETIME64.
    ///
    /// Supports:
    /// - `Datetime`
    /// - `Datetime64`
    /// - `Timestamp`
    /// - `TimestampNtz`
    /// - `TimestampWithTz`
    Datetime,
    /// INET.
    ///
    /// Supports:
    /// - `Inet`
    Inet,
    /// MACADDR.
    ///
    /// Supports:
    /// - `MacAddr`
    MacAddress,
    /// ARRAY of a single element type.
    ///
    /// Supports:
    /// - `Array`
    Array(Box<DataTypeSerializer>),
    /// Unspecified serializer.
    ///
    /// This is also useful when user doesn't specify the serializer,
    /// and also we couldn't detect the type from datatype.
    Unspecified,
    /// Custom serializer
    Custom(pyo3::Py<pyo3::PyAny>),
}

impl DataTypeSerializer {
    #[inline]
    pub fn infer_from_name(value: &super::name::DataTypeName) -> Self {
        use super::name::DataTypeGroup;

        match value.group() {
            DataTypeGroup::STRING => Self::String,
            DataTypeGroup::UUID => Self::Uuid,
            DataTypeGroup::BYTES => Self::Bytes,
            DataTypeGroup::BIT => Self::Bit,
            DataTypeGroup::INTEGER => Self::Integer,
            DataTypeGroup::FLOAT => Self::Float,
            DataTypeGroup::BOOLEAN => Self::Boolean,
            DataTypeGroup::DATE => Self::Date,
            DataTypeGroup::TIME => Self::Time,
            DataTypeGroup::DATETIME => Self::Datetime,
            DataTypeGroup::JSON | DataTypeGroup::JSONB => Self::JSON,
            DataTypeGroup::INET => Self::Inet,
            DataTypeGroup::MACADDRESS => Self::MacAddress,
            DataTypeGroup::CUSTOM => Self::Unspecified,
        }
    }

    pub fn serialize<'py>(
        &self,
        dialect: &crate::native::dialect::SqlDialect,
        object: &alias::BoundObject<'py>,
    ) -> pyo3::PyResult<String> {
        unsafe {
            match self {
                Self::Integer => serialize_integer(object.py(), object.as_ptr()),
                Self::Float => serialize_float(object.py(), object.as_ptr()),
                Self::Boolean => serialize_bool(object.py(), object.as_ptr(), &dialect),
                Self::String => serialize_str(object.py(), object.as_ptr(), &dialect),
                Self::Bytes => serialize_bytes(object.py(), object.as_ptr(), &dialect),
                Self::Bit => serialize_bit(object.py(), object.as_ptr(), &dialect),
                Self::JSON => serialize_json(object.py(), object.as_ptr(), &dialect),
                Self::Uuid => serialize_uuid(object.py(), object.as_ptr(), &dialect),
                Self::Date => serialize_date(object.py(), object.as_ptr(), &dialect),
                Self::Time => serialize_time(object.py(), object.as_ptr(), &dialect),
                Self::Datetime => serialize_datetime(object.py(), object.as_ptr(), &dialect),
                Self::Inet => serialize_inet(object.py(), object.as_ptr(), &dialect),
                Self::MacAddress => serialize_mac_address(object.py(), object.as_ptr(), &dialect),
                Self::Array(element) => {
                    serialize_array(object.py(), object.as_ptr(), &dialect, element)
                }
                Self::Unspecified => py_to_string(object.py(), object.as_ptr()),
                Self::Custom(s) => {
                    let dialect_arg = dialect.name();
                    let output = s.call1(object.py(), (dialect_arg, object))?;
                    _serialize_string(object.py(), output.as_ptr())
                }
            }
        }
    }
}

/// Serializes any Python `int` as a bare decimal literal.
///
/// Uses `str(int)` so arbitrary precision (beyond i64/u64) is preserved - one
/// serializer genuinely covers TINYINT through BIGINT and the unsigned variants.
#[inline]
unsafe fn serialize_integer(
    py: pyo3::Python,
    ptr: *mut pyo3::ffi::PyObject,
) -> pyo3::PyResult<String> {
    if pyo3::ffi::PyLong_CheckExact(ptr) == 1 {
        // bool is a subclass of int in Python, but PyLong_CheckExact rejects
        // it, so True/False never sneaks through here.
        py_to_string(py, ptr)
    } else {
        new_pyerr!(
            Err(PyTypeError),
            "expected int for integer serialization, got {}",
            get_type_name(py, ptr)
        )
    }
}

/// Serializes Python `float`, `int`, `decimal.Decimal`, or numeric `str` as a
/// bare numeric literal.
///
/// `Decimal`/`int` go through their exact text form (no precision loss);
/// only a real `float` is formatted from the f64. Non-finite floats are rejected.
#[inline]
unsafe fn serialize_float(
    py: pyo3::Python,
    ptr: *mut pyo3::ffi::PyObject,
) -> pyo3::PyResult<String> {
    if pyo3::ffi::PyFloat_CheckExact(ptr) == 1 {
        let v = pyo3::ffi::PyFloat_AsDouble(ptr);
        if !v.is_finite() {
            return new_pyerr!(
                Err(PyValueError),
                "cannot serialize non-finite float {} as a numeric literal",
                v
            );
        }

        // {:?} on f64 round-trips and avoids the "1" vs "1.0" ambiguity.
        return Ok(format!("{v:?}"));
    }

    // int / Decimal: their textual form is already a valid literal.
    if pyo3::ffi::PyLong_CheckExact(ptr) == 1
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
            return new_pyerr!(
                Err(PyValueError),
                "string {:?} is not a valid numeric literal",
                text
            );
        }
        return Ok(text.trim().to_string());
    }

    new_pyerr!(
        Err(PyTypeError),
        "expected float/int/Decimal/str for numeric serialization, got {}",
        get_type_name(py, ptr)
    )
}

#[inline]
unsafe fn serialize_bool(
    py: pyo3::Python,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &crate::native::dialect::SqlDialect,
) -> pyo3::PyResult<String> {
    if pyo3::ffi::PyBool_Check(ptr) == 1 {
        let value = pyo3::ffi::Py_True() == ptr;
        Ok(dialect.bool_format().format(value).to_string())
    } else {
        new_pyerr!(
            Err(PyTypeError),
            "expected bool for boolean serialization, got {}",
            get_type_name(py, ptr)
        )
    }
}

/// Serializes a Python `str` (or `enum.Enum` whose value is a str) as a quoted,
/// escaped string literal: `"LIKE"` -> `'LIKE'`.
#[inline]
unsafe fn serialize_str(
    py: pyo3::Python,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &crate::native::dialect::SqlDialect,
) -> pyo3::PyResult<String> {
    const ENUM_VALUE_NAME: &std::ffi::CStr = pyo3::ffi::c_str!("value");

    if pyo3::ffi::PyUnicode_CheckExact(ptr) == 1 {
        let raw = _serialize_string(py, ptr)?;
        let raw = dialect.escape_string(&raw);
        return Ok(dialect.quote_string(&raw));
    }

    // Accept enum members whose `.value` is a string.
    if (*pyo3::ffi::Py_TYPE(ptr)).ob_base.ob_base.ob_type == crate::typeref::STD_ENUM_TYPE {
        let value = pyo3::ffi::PyObject_GetAttrString(ptr, ENUM_VALUE_NAME.as_ptr());
        if value.is_null() {
            return Err(pyo3::PyErr::fetch(py));
        }

        let is_str = pyo3::ffi::PyUnicode_CheckExact(value) == 1;
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

    new_pyerr!(
        Err(PyTypeError),
        "expected str/Enum for string serialization, got {}",
        get_type_name(py, ptr)
    )
}

/// Serializes a `uuid.UUID` (or 36-char `str`) as a quoted literal.
#[inline]
unsafe fn serialize_uuid(
    py: pyo3::Python,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &crate::native::dialect::SqlDialect,
) -> pyo3::PyResult<String> {
    if pyo3::ffi::Py_TYPE(ptr) == crate::typeref::STD_UUID_TYPE
        || pyo3::ffi::PyUnicode_CheckExact(ptr) == 1
    {
        let text = py_to_string(py, ptr)?;
        let text = dialect.escape_string(&text);
        return Ok(dialect.quote_string(&text));
    } else {
        new_pyerr!(
            Err(PyTypeError),
            "expected uuid.UUID/str for UUID serialization, got {}",
            get_type_name(py, ptr)
        )
    }
}

#[inline]
unsafe fn serialize_bytes(
    py: pyo3::Python,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &crate::native::dialect::SqlDialect,
) -> pyo3::PyResult<String> {
    if pyo3::ffi::PyBytes_CheckExact(ptr) == 1 {
        Ok(dialect.encode_bytes(_read_bytes(ptr)))
    } else {
        Err(new_pyerr!(
            PyTypeError,
            "expected bytes for binary serialization, got {}",
            get_type_name(py, ptr)
        ))
    }
}

/// Serializes a bit string given as a `str` of `0`/`1` (e.g. `"1010"`) into a
/// `B'1010'` literal.
#[inline]
unsafe fn serialize_bit(
    py: pyo3::Python,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &crate::native::dialect::SqlDialect,
) -> pyo3::PyResult<String> {
    if pyo3::ffi::PyUnicode_CheckExact(ptr) == 1 {
        let raw = _serialize_string(py, ptr)?;

        if raw.is_empty() || !raw.bytes().all(|b| b == b'0' || b == b'1') {
            return new_pyerr!(
                Err(PyValueError),
                "expected a string of '0'/'1' for bit serialization, got {:?}",
                raw
            );
        }
        Ok(dialect.encode_bits(&raw))
    } else {
        new_pyerr!(
            Err(PyTypeError),
            "expected str of bits for bit serialization, got {}",
            get_type_name(py, ptr)
        )
    }
}

/// Serializes any JSON-encodable Python object via `json.dumps`, then wraps the
/// result as a quoted string literal.
#[inline]
unsafe fn serialize_json(
    py: pyo3::Python,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &crate::native::dialect::SqlDialect,
) -> pyo3::PyResult<String> {
    let dumped = _serialize_object_with_pyjson(py, ptr)?;
    let text = _serialize_string(py, dumped);
    pyo3::ffi::Py_DECREF(dumped);

    let text = dialect.escape_string(&text?);
    Ok(dialect.quote_string(&text))
}

/// `datetime.date` -> `'YYYY-MM-DD'`.
#[inline]
unsafe fn serialize_date(
    py: pyo3::Python,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &crate::native::dialect::SqlDialect,
) -> pyo3::PyResult<String> {
    if pyo3::ffi::Py_TYPE(ptr) == crate::typeref::STD_DATE_TYPE {
        let val: pyo3::Bound<'_, pyo3::types::PyDate> =
            pyo3::Bound::from_borrowed_ptr(py, ptr).cast_into()?;

        let date: chrono::NaiveDate = val.extract()?;
        let result = date.format("%Y-%m-%d").to_string();

        Ok(dialect.quote_string(&result))
    } else {
        Err(new_pyerr!(
            PyTypeError,
            "expected datetime.date for date serialization, got {}",
            get_type_name(py, ptr)
        ))
    }
}

/// `datetime.time` -> `'HH:MM:SS[.ffffff]'`.
#[inline]
unsafe fn serialize_time(
    py: pyo3::Python,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &crate::native::dialect::SqlDialect,
) -> pyo3::PyResult<String> {
    if pyo3::ffi::Py_TYPE(ptr) == crate::typeref::STD_TIME_TYPE {
        let val: pyo3::Bound<'_, pyo3::types::PyTime> =
            pyo3::Bound::from_borrowed_ptr(py, ptr).cast_into()?;

        let date: chrono::NaiveTime = val.extract()?;
        let result = date.format("%H:%M:%S%.f").to_string();

        Ok(dialect.quote_string(&result))
    } else {
        Err(new_pyerr!(
            PyTypeError,
            "expected datetime.time for time serialization, got {}",
            get_type_name(py, ptr)
        ))
    }
}

/// `datetime.datetime` -> `'YYYY-MM-DD HH:MM:SS[.ffffff][+HH:MM]'`.
///
/// `int`/`float` are treated as a UTC epoch and accepted too.
#[inline]
unsafe fn serialize_datetime(
    py: pyo3::Python,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &crate::native::dialect::SqlDialect,
) -> pyo3::PyResult<String> {
    use chrono::TimeZone;
    use pyo3::types::PyTzInfoAccess;

    if pyo3::ffi::Py_TYPE(ptr) == crate::typeref::STD_DATETIME_TYPE {
        // isoformat(sep=' ') -> the SQL-friendly space-separated form.
        let val: pyo3::Bound<'_, pyo3::types::PyDateTime> =
            pyo3::Bound::from_borrowed_ptr(py, ptr).cast_into()?;

        let tzinfo = val.get_tzinfo();

        if tzinfo.is_none() {
            let datetime: chrono::NaiveDateTime = val.extract()?;
            let result = datetime.format("%Y-%m-%d %H:%M:%S%.f").to_string();

            return Ok(dialect.quote_string(&result));
        } else {
            let datetime: chrono::DateTime<chrono::Utc> = val.extract()?;
            let result = datetime.format("%Y-%m-%d %H:%M:%S%.f%:z").to_string();

            return Ok(dialect.quote_string(&result));
        }
    }

    if pyo3::ffi::PyLong_CheckExact(ptr) == 1 {
        let mut has_overflow: i32 = 0;
        let num = pyo3::ffi::PyLong_AsLongLongAndOverflow(ptr, &mut has_overflow);

        if has_overflow == 1 {
            return Err(pyo3::exceptions::PyOverflowError::new_err("out of range"));
        }

        if let chrono::offset::LocalResult::Single(datetime) = chrono::Utc.timestamp_opt(num, 0) {
            let result = datetime.format("%Y-%m-%d %H:%M:%S%.f%:z").to_string();
            return Ok(dialect.quote_string(&result));
        }

        return Err(new_pyerr!(
            PyOverflowError,
            "timestamp is invalid for datetime serialization: {}",
            num
        ));
    }

    if pyo3::ffi::PyFloat_CheckExact(ptr) == 1 {
        let num = pyo3::ffi::PyFloat_AsDouble(ptr);

        if let chrono::offset::LocalResult::Single(datetime) =
            chrono::Utc.timestamp_opt(num.trunc() as i64, (num.fract() * 1_000_000_000.0) as u32)
        {
            let result = datetime.format("%Y-%m-%d %H:%M:%S%.f%:z").to_string();
            return Ok(dialect.quote_string(&result));
        }

        return Err(new_pyerr!(
            PyOverflowError,
            "timestamp is invalid for datetime serialization: {}",
            num
        ));
    }

    Err(new_pyerr!(
        PyTypeError,
        "expected datetime.datetime/int/float for datetime serialization, got {}",
        get_type_name(py, ptr)
    ))
}

/// `str` (or `ipaddress.*`) -> quoted literal. Validated as IPv4/IPv6 network.
#[inline]
unsafe fn serialize_inet(
    py: pyo3::Python,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &crate::native::dialect::SqlDialect,
) -> pyo3::PyResult<String> {
    use std::str::FromStr;

    let text = py_to_string(py, ptr)?;
    let net = ipnetwork::IpNetwork::from_str(&text)
        .map(|net| net.to_string())
        .map_err(|e| new_pyerr!(PyValueError, e.to_string()))?;

    Ok(dialect.quote_string(&net))
}

/// `str` -> quoted literal. Validated as a MAC address.
#[inline]
unsafe fn serialize_mac_address(
    py: pyo3::Python,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &crate::native::dialect::SqlDialect,
) -> pyo3::PyResult<String> {
    use std::str::FromStr;
    let text = py_to_string(py, ptr)?;

    let addr = mac_address::MacAddress::from_str(&text)
        .map(|mac| mac.to_string())
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    Ok(dialect.quote_string(&addr))
}

/// Serializes a Python `list`/`tuple` by serializing each element with `element`
/// and wrapping in `ARRAY[...]`.
#[inline]
unsafe fn serialize_array(
    py: pyo3::Python,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &crate::native::dialect::SqlDialect,
    element: &DataTypeSerializer,
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
        return new_pyerr!(
            Err(PyTypeError),
            "expected list/tuple for array serialization, got {}",
            get_type_name(py, ptr)
        );
    };

    let mut parts: Vec<String> = Vec::with_capacity(len.max(0) as usize);
    for i in 0..len {
        let item = getter(ptr, i); // borrowed
        if item.is_null() {
            return Err(pyo3::PyErr::fetch(py));
        }

        let borrowed = alias::BoundObject::from_borrowed_ptr(py, item);
        parts.push(element.serialize(dialect, &borrowed)?);
    }

    Ok(format!("ARRAY[{}]", parts.join(", ")))
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
unsafe fn _read_bytes<'a>(ptr: *mut pyo3::ffi::PyObject) -> &'a [u8] {
    let buf = pyo3::ffi::PyBytes_AsString(ptr) as *const u8;
    let len = pyo3::ffi::PyBytes_Size(ptr) as usize;
    debug_assert!(!buf.is_null());
    std::slice::from_raw_parts(buf, len)
}

/// Import json module only once
#[inline(always)]
pub fn _import_json_module(
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
    let json = _import_json_module(py)?;
    let dumps_func = json.getattr("dumps")?;

    unsafe {
        let arg1 = pyo3::Bound::from_borrowed_ptr(py, ptr);
        dumps_func.call1((arg1,)).map(|x| x.into_ptr())
    }
}
