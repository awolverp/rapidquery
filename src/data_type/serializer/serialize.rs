//! Write a Python value as a SQL literal into a `String`.
//!
//! One buffer, dialect writers (`write_string_literal`, `encode_bytes`, …).
//! Type checks are exact (`int` is not `bool`; `date` is not `datetime`).
//! JSON dumps go through [`crate::internal::modules::json_dumps`].

use std::fmt::Write;
use std::str::FromStr;

use pyo3::types::{PyAnyMethods, PyTzInfoAccess};

use crate::dialect::SqlDialect;
use crate::internal::alias;
use crate::internal::utils::{borrow_py_str_buffer, get_type_name};

impl super::serializer_enum::SerializerEnum {
    pub fn write_sql(
        &self,
        dialect: &SqlDialect,
        object: &alias::BoundObject<'_>,
        out: &mut String,
    ) -> pyo3::PyResult<()> {
        if !self.is_supported_by(dialect) {
            return rapidquery_macros::pyerr!(
                Err(PyNotImplementedError),
                "{} does not support {}",
                dialect.name(),
                super::class_name(self)
            );
        }

        let py = object.py();
        let ptr = object.as_ptr();
        unsafe {
            match self {
                Self::Integer => write_integer(py, ptr, out),
                Self::Float => write_float(py, ptr, out),
                Self::Boolean => write_bool(py, ptr, dialect, out),
                Self::String => write_str(py, ptr, dialect, out),
                Self::Uuid => write_uuid(py, ptr, dialect, out),
                Self::Bytes => write_bytes(py, ptr, dialect, out),
                Self::Bit => write_bit(py, ptr, dialect, out),
                Self::Json => write_json(py, ptr, dialect, out),
                Self::Date => write_date(py, ptr, dialect, out),
                Self::Time => write_time(py, ptr, dialect, out),
                Self::Datetime => write_datetime(py, ptr, dialect, out),
                Self::Inet => write_inet(py, ptr, dialect, out),
                Self::MacAddress => write_mac_address(py, ptr, dialect, out),
                Self::Array(element) => write_array(py, ptr, dialect, element, out),
                Self::Unspecified => {
                    let text = py_to_string(py, ptr)?;
                    out.push_str(&text);
                    Ok(())
                }
                Self::Custom(s) => write_custom(py, ptr, dialect, s, object, out),
            }
        }
    }
}

/// `str(obj)` copied into a Rust `String`. Used when the unicode is a
/// temporary (`PyObject_Str`) that must be `DECREF`'d before return.
#[inline]
unsafe fn py_to_string(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
) -> pyo3::PyResult<String> {
    let s = pyo3::ffi::PyObject_Str(ptr);
    if s.is_null() {
        return Err(pyo3::PyErr::fetch(py));
    }
    let out = borrow_py_str_buffer(py, s).map(str::to_owned);
    pyo3::ffi::Py_DECREF(s);
    out
}

#[inline]
unsafe fn py_bytes<'a>(ptr: *mut pyo3::ffi::PyObject) -> &'a [u8] {
    let buf = pyo3::ffi::PyBytes_AsString(ptr) as *const u8;
    let len = pyo3::ffi::PyBytes_Size(ptr) as usize;
    debug_assert!(!buf.is_null());
    std::slice::from_raw_parts(buf, len)
}

#[inline]
fn write_quoted(dialect: &SqlDialect, out: &mut String, raw: &str) {
    let _ = dialect.write_string_literal(out, raw);
}

/// Serializes any Python `int` as a bare decimal literal (`str(int)`),
/// including values beyond i64/u64. `bool` is rejected (`PyLong_CheckExact`).
///
/// # Safety
/// `ptr` must be *borrowed*, non-null, and a valid Python object.
#[inline]
unsafe fn write_integer(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    out: &mut String,
) -> pyo3::PyResult<()> {
    if pyo3::ffi::PyLong_CheckExact(ptr) == 1 {
        let text = py_to_string(py, ptr)?;
        out.push_str(&text);
        Ok(())
    } else {
        rapidquery_macros::pyerr!(
            Err(PyTypeError),
            "expected int for integer serialization, got {}",
            get_type_name(py, ptr)
        )
    }
}

/// `float` / exact `int` / `decimal.Decimal` as a bare numeric literal.
///
/// Only a real `float` is formatted from the f64 (`{:?}` so `1.0` stays
/// distinct from `1`). Non-finite floats are rejected. `str` is not accepted.
///
/// # Safety
/// `ptr` must be *borrowed*, non-null, and a valid Python object.
#[inline]
unsafe fn write_float(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    out: &mut String,
) -> pyo3::PyResult<()> {
    if pyo3::ffi::PyFloat_CheckExact(ptr) == 1 {
        let v = pyo3::ffi::PyFloat_AsDouble(ptr);
        if !v.is_finite() {
            return rapidquery_macros::pyerr!(
                Err(PyValueError),
                "cannot serialize non-finite float {} as a numeric literal",
                v
            );
        }
        let _ = write!(out, "{v:?}");
        return Ok(());
    }

    if pyo3::ffi::PyLong_CheckExact(ptr) == 1
        || crate::typeref::check_exact(ptr, crate::typeref::STD_DECIMAL_TYPE)
    {
        let text = py_to_string(py, ptr)?;
        out.push_str(&text);
        return Ok(());
    }

    rapidquery_macros::pyerr!(
        Err(PyTypeError),
        "expected float/int/Decimal for numeric serialization, got {}",
        get_type_name(py, ptr)
    )
}

/// # Safety
/// `ptr` must be *borrowed*, non-null, and a valid Python object.
#[inline]
unsafe fn write_bool(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &SqlDialect,
    out: &mut String,
) -> pyo3::PyResult<()> {
    if pyo3::ffi::PyBool_Check(ptr) == 1 {
        let value = pyo3::ffi::Py_True() == ptr;
        let _ = dialect.format_boolean(out, value);
        Ok(())
    } else {
        rapidquery_macros::pyerr!(
            Err(PyTypeError),
            "expected bool for boolean serialization, got {}",
            get_type_name(py, ptr)
        )
    }
}

/// Exact `str`, or an enum member whose `.value` is a `str`.
///
/// # Safety
/// `ptr` must be *borrowed*, non-null, and a valid Python object.
#[inline]
unsafe fn write_str(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &SqlDialect,
    out: &mut String,
) -> pyo3::PyResult<()> {
    if pyo3::ffi::PyUnicode_Check(ptr) == 1 {
        write_quoted(dialect, out, borrow_py_str_buffer(py, ptr)?);
        return Ok(());
    }

    rapidquery_macros::pyerr!(
        Err(PyTypeError),
        "expected str for string serialization, got {}",
        get_type_name(py, ptr)
    )
}

/// # Safety
/// `ptr` must be *borrowed*, non-null, and a valid Python object.
#[inline]
unsafe fn write_uuid(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &SqlDialect,
    out: &mut String,
) -> pyo3::PyResult<()> {
    if crate::typeref::check_exact(ptr, crate::typeref::STD_UUID_TYPE) {
        let text = py_to_string(py, ptr)?;
        write_quoted(dialect, out, &text);
        Ok(())
    } else if pyo3::ffi::PyUnicode_CheckExact(ptr) == 1 {
        let borrowed = borrow_py_str_buffer(py, ptr)?;
        let parsed = uuid::Uuid::try_parse(borrowed)
            .map_err(|e| rapidquery_macros::pyerr!(PyValueError, "invalid uuid str: {}", e))?;

        write_quoted(dialect, out, &parsed.hyphenated().to_string());
        Ok(())
    } else {
        rapidquery_macros::pyerr!(
            Err(PyTypeError),
            "expected uuid.UUID/str for UUID serialization, got {}",
            get_type_name(py, ptr)
        )
    }
}

/// # Safety
/// `ptr` must be *borrowed*, non-null, and a valid Python object.
#[inline]
unsafe fn write_bytes(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &SqlDialect,
    out: &mut String,
) -> pyo3::PyResult<()> {
    if pyo3::ffi::PyBytes_CheckExact(ptr) == 1 {
        let _ = dialect.encode_bytes(out, py_bytes(ptr));
        Ok(())
    } else {
        Err(rapidquery_macros::pyerr!(
            PyTypeError,
            "expected bytes for binary serialization, got {}",
            get_type_name(py, ptr)
        ))
    }
}

/// # Safety
/// `ptr` must be *borrowed*, non-null, and a valid Python object.
#[inline]
unsafe fn write_bit(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &SqlDialect,
    out: &mut String,
) -> pyo3::PyResult<()> {
    if pyo3::ffi::PyUnicode_CheckExact(ptr) == 1 {
        let raw = borrow_py_str_buffer(py, ptr)?;
        if raw.is_empty() || !raw.bytes().all(|b| b == b'0' || b == b'1') {
            return rapidquery_macros::pyerr!(
                Err(PyValueError),
                "expected a string of '0'/'1' for bit serialization, got {:?}",
                raw
            );
        }
        let _ = dialect.encode_bits(out, raw);
        Ok(())
    } else {
        rapidquery_macros::pyerr!(
            Err(PyTypeError),
            "expected str of bits for bit serialization, got {}",
            get_type_name(py, ptr)
        )
    }
}

/// JSON-encodable object → dumps text → dialect string literal.
///
/// # Safety
/// `ptr` must be *borrowed*, non-null, and a valid Python object.
#[inline]
unsafe fn write_json(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &SqlDialect,
    out: &mut String,
) -> pyo3::PyResult<()> {
    crate::internal::modules::json_dumps(
        unsafe { pyo3::Bound::from_borrowed_ptr(py, ptr) },
        |text| {
            write_quoted(dialect, out, text);
            Ok(())
        },
    )
}

/// # Safety
/// `ptr` must be *borrowed*, non-null, and a valid Python object.
#[inline]
unsafe fn write_date(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &SqlDialect,
    out: &mut String,
) -> pyo3::PyResult<()> {
    if crate::typeref::check_exact(ptr, crate::typeref::STD_DATE_TYPE) {
        let val: pyo3::Bound<'_, pyo3::types::PyDate> =
            pyo3::Bound::from_borrowed_ptr(py, ptr).cast_into()?;
        let date: chrono::NaiveDate = val.extract()?;
        let text = date.format("%Y-%m-%d").to_string();
        write_quoted(dialect, out, &text);
        Ok(())
    } else {
        Err(rapidquery_macros::pyerr!(
            PyTypeError,
            "expected datetime.date for date serialization, got {}",
            get_type_name(py, ptr)
        ))
    }
}

/// # Safety
/// `ptr` must be *borrowed*, non-null, and a valid Python object.
#[inline]
unsafe fn write_time(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &SqlDialect,
    out: &mut String,
) -> pyo3::PyResult<()> {
    if crate::typeref::check_exact(ptr, crate::typeref::STD_TIME_TYPE) {
        let val: pyo3::Bound<'_, pyo3::types::PyTime> =
            pyo3::Bound::from_borrowed_ptr(py, ptr).cast_into()?;
        let time: chrono::NaiveTime = val.extract()?;
        let text = time.format("%H:%M:%S%.f").to_string();
        write_quoted(dialect, out, &text);
        Ok(())
    } else {
        Err(rapidquery_macros::pyerr!(
            PyTypeError,
            "expected datetime.time for time serialization, got {}",
            get_type_name(py, ptr)
        ))
    }
}

/// Naive `datetime` keeps wall time. Aware `datetime` keeps its offset
/// (`DateTime<FixedOffset>`), not a conversion to UTC. `int`/`float` are
/// UTC epoch seconds.
///
/// # Safety
/// `ptr` must be *borrowed*, non-null, and a valid Python object.
#[inline]
unsafe fn write_datetime(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &SqlDialect,
    out: &mut String,
) -> pyo3::PyResult<()> {
    use chrono::TimeZone;

    if crate::typeref::check_exact(ptr, crate::typeref::STD_DATETIME_TYPE) {
        let val: pyo3::Bound<'_, pyo3::types::PyDateTime> =
            pyo3::Bound::from_borrowed_ptr(py, ptr).cast_into()?;

        if val.get_tzinfo().is_none() {
            let datetime: chrono::NaiveDateTime = val.extract()?;
            let text = datetime.format("%Y-%m-%d %H:%M:%S%.f").to_string();
            write_quoted(dialect, out, &text);
            return Ok(());
        }

        let datetime: chrono::DateTime<chrono::FixedOffset> = val.extract()?;
        let text = datetime.format("%Y-%m-%d %H:%M:%S%.f%:z").to_string();
        write_quoted(dialect, out, &text);
        return Ok(());
    }

    if pyo3::ffi::PyLong_CheckExact(ptr) == 1 {
        let mut has_overflow: i32 = 0;
        let num = pyo3::ffi::PyLong_AsLongLongAndOverflow(ptr, &mut has_overflow);
        if has_overflow != 0 {
            return Err(pyo3::exceptions::PyOverflowError::new_err("out of range"));
        }
        return match chrono::Utc.timestamp_opt(num, 0) {
            chrono::offset::LocalResult::Single(datetime) => {
                let text = datetime.format("%Y-%m-%d %H:%M:%S%.f%:z").to_string();
                write_quoted(dialect, out, &text);
                Ok(())
            }
            _ => Err(rapidquery_macros::pyerr!(
                PyOverflowError,
                "timestamp is invalid for datetime serialization: {}",
                num
            )),
        };
    }

    if pyo3::ffi::PyFloat_CheckExact(ptr) == 1 {
        let num = pyo3::ffi::PyFloat_AsDouble(ptr);
        return match chrono::Utc
            .timestamp_opt(num.trunc() as i64, (num.fract() * 1_000_000_000.0) as u32)
        {
            chrono::offset::LocalResult::Single(datetime) => {
                let text = datetime.format("%Y-%m-%d %H:%M:%S%.f%:z").to_string();
                write_quoted(dialect, out, &text);
                Ok(())
            }
            _ => Err(rapidquery_macros::pyerr!(
                PyOverflowError,
                "timestamp is invalid for datetime serialization: {}",
                num
            )),
        };
    }

    Err(rapidquery_macros::pyerr!(
        PyTypeError,
        "expected datetime.datetime/int/float for datetime serialization, got {}",
        get_type_name(py, ptr)
    ))
}

/// # Safety
/// `ptr` must be *borrowed*, non-null, and a valid Python object.
#[inline]
unsafe fn write_inet(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &SqlDialect,
    out: &mut String,
) -> pyo3::PyResult<()> {
    let text = py_to_string(py, ptr)?;
    let net = ipnetwork::IpNetwork::from_str(&text)
        .map(|net| net.to_string())
        .map_err(|e| rapidquery_macros::pyerr!(PyValueError, e.to_string()))?;
    write_quoted(dialect, out, &net);
    Ok(())
}

/// # Safety
/// `ptr` must be *borrowed*, non-null, and a valid Python object.
#[inline]
unsafe fn write_mac_address(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &SqlDialect,
    out: &mut String,
) -> pyo3::PyResult<()> {
    let text = py_to_string(py, ptr)?;
    let addr = mac_address::MacAddress::from_str(&text)
        .map(|mac| mac.to_string())
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    write_quoted(dialect, out, &addr);
    Ok(())
}

/// # Safety
/// `ptr` must be *borrowed*, non-null, and a valid Python object.
#[inline]
unsafe fn write_array(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &SqlDialect,
    element: &super::serializer_enum::SerializerEnum,
    out: &mut String,
) -> pyo3::PyResult<()> {
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
        return rapidquery_macros::pyerr!(
            Err(PyTypeError),
            "expected list/tuple for array serialization, got {}",
            get_type_name(py, ptr)
        );
    };

    out.push_str("ARRAY[");
    for i in 0..len {
        if i > 0 {
            out.push_str(", ");
        }
        let item = getter(ptr, i);
        if item.is_null() {
            return Err(pyo3::PyErr::fetch(py));
        }
        let borrowed = alias::BoundObject::from_borrowed_ptr(py, item);
        element.write_sql(dialect, &borrowed, out)?;
    }
    out.push(']');
    Ok(())
}

/// # Safety
/// `ptr` must be *borrowed*, non-null, and a valid Python object.
#[inline]
unsafe fn write_custom(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &SqlDialect,
    function: &pyo3::Py<pyo3::PyAny>,
    object: &alias::BoundObject<'_>,
    out: &mut String,
) -> pyo3::PyResult<()> {
    let _ = ptr;
    let output = function.call1(py, (dialect.name(), object))?;
    if pyo3::ffi::PyUnicode_CheckExact(output.as_ptr()) != 1 {
        return rapidquery_macros::pyerr!(
            Err(PyTypeError),
            "custom serializer must return str, got {}",
            get_type_name(py, output.as_ptr())
        );
    }
    out.push_str(borrow_py_str_buffer(py, output.as_ptr())?);
    Ok(())
}
