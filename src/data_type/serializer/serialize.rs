//! Write a Python value as a SQL literal into a `String`.

use std::fmt::Write;
use std::str::FromStr;

use pyo3::types::{PyAnyMethods, PyTzInfoAccess};

use crate::dialect::PySqlDialect;
use crate::dialect::inner::{
    ArrayFormat, Dialect, FormatError, InetFormat, JsonFormat, MacFormat, StringFormat, UuidFormat,
};
use crate::internal::alias;
use crate::internal::utils::{borrow_py_str_buffer, get_type_name};

impl super::serializer_enum::SerializerEnum {
    pub fn write_sql(
        &self,
        dialect: &PySqlDialect,
        object: &alias::BoundObject<'_>,
        out: &mut String,
    ) -> pyo3::PyResult<()> {
        let py = object.py();
        let ptr = object.as_ptr();
        unsafe {
            match self {
                Self::Integer => write_integer(py, ptr, out),
                Self::Float => write_float(py, ptr, out),
                Self::Boolean => write_bool(py, ptr, &dialect.0, out),
                Self::String => write_str(py, ptr, &dialect.0, out),
                Self::Uuid => write_uuid(py, ptr, &dialect.0, out),
                Self::Bytes => write_bytes(py, ptr, &dialect.0, out),
                Self::Bit => write_bit(py, ptr, &dialect.0, out),
                Self::Json => write_json(py, ptr, &dialect.0, out),
                Self::Date => write_date(py, ptr, out),
                Self::Time => write_time(py, ptr, out),
                Self::Datetime => write_datetime(py, ptr, out),
                Self::Inet => write_inet(py, ptr, &dialect.0, out),
                Self::MacAddress => write_mac_address(py, ptr, &dialect.0, out),
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
fn write_quoted(dialect: &Dialect, out: &mut String, raw: &str) -> pyo3::PyResult<()> {
    dialect.write_string_literal(out, raw).map_err(Into::into)
}

/// Open a string literal and write `body` inside it.
///
/// `body` is digits, `-`, `:`, `.`, `+`, and space only: no quote, backslash,
/// NUL, or C0. `PrefixBigE` therefore stays a plain quote, the same branch
/// `write_string_literal` takes when the body needs no C escape. `PrefixBigN`
/// still writes `N'`.
#[inline]
fn write_quoted_body(
    dialect: &Dialect,
    out: &mut String,
    body_fn: impl FnOnce(&mut String) -> std::fmt::Result,
) -> pyo3::PyResult<()> {
    match dialect.string_format {
        StringFormat::PrefixBigN => out.push_str("N'"),
        StringFormat::SingleQuote | StringFormat::PrefixBigE => out.push('\''),
    }
    body_fn(out).map_err(|_| rapidquery_macros::pyerr!(PyRuntimeError, "failed to write SQL"))?;
    out.push('\'');
    Ok(())
}

/// Serializes any Python `int` as a bare decimal literal (`str(int)`).
///
/// Values that fit in `i64` are written with [`write!`]. Larger values use
/// `PyObject_Str`. `bool` is rejected (`PyLong_CheckExact`).
///
/// # Safety
/// `ptr` must be *borrowed*, non-null, and a valid Python object.
#[inline]
unsafe fn write_integer(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    out: &mut String,
) -> pyo3::PyResult<()> {
    if pyo3::ffi::PyLong_CheckExact(ptr) != 1 {
        return rapidquery_macros::pyerr!(
            Err(PyTypeError),
            "expected int for integer serialization, got {}",
            get_type_name(py, ptr)
        );
    }

    let mut overflow = 0i32;
    let n = pyo3::ffi::PyLong_AsLongLongAndOverflow(ptr, &mut overflow);
    if overflow != 0 {
        let text = py_to_string(py, ptr)?;
        out.push_str(&text);
        return Ok(());
    }

    if n == -1 && !pyo3::ffi::PyErr_Occurred().is_null() {
        return Err(pyo3::PyErr::fetch(py));
    }

    write!(out, "{n}").map_err(|_| rapidquery_macros::pyerr!(PyRuntimeError, "failed to write SQL"))
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
    dialect: &Dialect,
    out: &mut String,
) -> pyo3::PyResult<()> {
    if pyo3::ffi::PyBool_Check(ptr) == 1 {
        let value = pyo3::ffi::Py_True() == ptr;
        dialect.format_boolean(out, value)?;
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
    dialect: &Dialect,
    out: &mut String,
) -> pyo3::PyResult<()> {
    if pyo3::ffi::PyUnicode_Check(ptr) == 1 {
        return write_quoted(dialect, out, borrow_py_str_buffer(py, ptr)?);
    }

    rapidquery_macros::pyerr!(
        Err(PyTypeError),
        "expected str for string serialization, got {}",
        get_type_name(py, ptr)
    )
}

#[inline]
fn write_uuid_text(
    dialect: &Dialect,
    out: &mut String,
    body: impl FnOnce(&mut String) -> std::fmt::Result,
) -> pyo3::PyResult<()> {
    match dialect.uuid_format {
        UuidFormat::Quoted => write_quoted_body(dialect, out, body),
        UuidFormat::UuidKeyword => {
            out.push_str("UUID ");
            write_quoted_body(dialect, out, body)
        }
        UuidFormat::CastUuid => {
            write_quoted_body(dialect, out, body)?;
            out.push_str("::uuid");
            Ok(())
        }
    }
}

/// # Safety
/// `ptr` must be *borrowed*, non-null, and a valid Python object.
#[inline]
unsafe fn write_uuid(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &Dialect,
    out: &mut String,
) -> pyo3::PyResult<()> {
    if crate::typeref::check_exact(ptr, crate::typeref::STD_UUID_TYPE) {
        let rendered = pyo3::ffi::PyObject_Str(ptr);
        if rendered.is_null() {
            return Err(pyo3::PyErr::fetch(py));
        }
        let result = match borrow_py_str_buffer(py, rendered) {
            Ok(text) => write_uuid_text(dialect, out, |w| w.write_str(text)),
            Err(err) => Err(err),
        };
        pyo3::ffi::Py_DECREF(rendered);
        result
    } else if pyo3::ffi::PyUnicode_CheckExact(ptr) == 1 {
        let borrowed = borrow_py_str_buffer(py, ptr)?;
        let parsed = uuid::Uuid::try_parse(borrowed)
            .map_err(|e| rapidquery_macros::pyerr!(PyValueError, "invalid uuid str: {}", e))?;

        write_uuid_text(dialect, out, |w| write!(w, "{}", parsed.hyphenated()))
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
    dialect: &Dialect,
    out: &mut String,
) -> pyo3::PyResult<()> {
    if pyo3::ffi::PyBytes_CheckExact(ptr) == 1 {
        dialect.format_bytes(out, py_bytes(ptr))?;
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
    dialect: &Dialect,
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
        dialect.format_bits(out, raw)?;
        Ok(())
    } else {
        rapidquery_macros::pyerr!(
            Err(PyTypeError),
            "expected str of bits for bit serialization, got {}",
            get_type_name(py, ptr)
        )
    }
}

#[inline]
fn write_json_text(dialect: &Dialect, out: &mut String, raw: &str) -> pyo3::PyResult<()> {
    match dialect.json_format {
        JsonFormat::Quoted => write_quoted(dialect, out, raw),
        JsonFormat::JsonKeyword => {
            out.push_str("JSON ");
            write_quoted(dialect, out, raw)
        }
        JsonFormat::ParseJson => {
            out.push_str("PARSE_JSON(");
            write_quoted(dialect, out, raw)?;
            out.push(')');
            Ok(())
        }
        JsonFormat::CastJson => {
            write_quoted(dialect, out, raw)?;
            out.push_str("::json");
            Ok(())
        }
        JsonFormat::CastJsonb => {
            write_quoted(dialect, out, raw)?;
            out.push_str("::jsonb");
            Ok(())
        }
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
    dialect: &Dialect,
    out: &mut String,
) -> pyo3::PyResult<()> {
    crate::internal::modules::json_dumps(
        unsafe { pyo3::Bound::from_borrowed_ptr(py, ptr) },
        |text| write_json_text(dialect, out, text),
    )
}

/// # Safety
/// `ptr` must be *borrowed*, non-null, and a valid Python object.
#[inline]
unsafe fn write_date(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    out: &mut String,
) -> pyo3::PyResult<()> {
    if crate::typeref::check_exact(ptr, crate::typeref::STD_DATE_TYPE) {
        let val: pyo3::Bound<'_, pyo3::types::PyDate> =
            pyo3::Bound::from_borrowed_ptr(py, ptr).cast_into()?;

        let date: chrono::NaiveDate = val.extract()?;

        let _ = date.format("%Y-%m-%d").write_to(out);
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
    out: &mut String,
) -> pyo3::PyResult<()> {
    if crate::typeref::check_exact(ptr, crate::typeref::STD_TIME_TYPE) {
        let val: pyo3::Bound<'_, pyo3::types::PyTime> =
            pyo3::Bound::from_borrowed_ptr(py, ptr).cast_into()?;

        let time: chrono::NaiveTime = val.extract()?;
        let _ = time.format("%H:%M:%S%.f").write_to(out);
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
    out: &mut String,
) -> pyo3::PyResult<()> {
    use chrono::TimeZone;

    if crate::typeref::check_exact(ptr, crate::typeref::STD_DATETIME_TYPE) {
        let val: pyo3::Bound<'_, pyo3::types::PyDateTime> =
            pyo3::Bound::from_borrowed_ptr(py, ptr).cast_into()?;

        if val.get_tzinfo().is_none() {
            let datetime: chrono::NaiveDateTime = val.extract()?;

            let _ = datetime.format("%Y-%m-%d %H:%M:%S%.f").write_to(out);
            return Ok(());
        }

        let datetime: chrono::DateTime<chrono::FixedOffset> = val.extract()?;
        let _ = datetime.format("%Y-%m-%d %H:%M:%S%.f%:z").write_to(out);
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
                let _ = datetime.format("%Y-%m-%d %H:%M:%S%.f%:z").write_to(out);
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
                let _ = datetime.format("%Y-%m-%d %H:%M:%S%.f%:z").write_to(out);
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
    dialect: &Dialect,
    out: &mut String,
) -> pyo3::PyResult<()> {
    let text = py_to_string(py, ptr)?;
    let net = ipnetwork::IpNetwork::from_str(&text)
        .map_err(|e| rapidquery_macros::pyerr!(PyValueError, e.to_string()))?;

    match dialect.inet_format {
        None => Err(FormatError::Unsupported("InetSerializer").into()),
        Some(InetFormat::Quoted) => write_quoted_body(dialect, out, |w| write!(w, "{net}")),
        Some(InetFormat::Cast) => {
            write_quoted_body(dialect, out, |w| write!(w, "{net}"))?;
            out.push_str("::inet");
            Ok(())
        }
    }
}

/// # Safety
/// `ptr` must be *borrowed*, non-null, and a valid Python object.
#[inline]
unsafe fn write_mac_address(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &Dialect,
    out: &mut String,
) -> pyo3::PyResult<()> {
    let text = py_to_string(py, ptr)?;
    let addr = mac_address::MacAddress::from_str(&text)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    match dialect.mac_format {
        None => Err(FormatError::Unsupported("MacAddressSerializer").into()),
        Some(MacFormat::Quoted) => write_quoted_body(dialect, out, |w| write!(w, "{addr}")),
        Some(MacFormat::Cast) => {
            write_quoted_body(dialect, out, |w| write!(w, "{addr}"))?;
            out.push_str("::macaddr");
            Ok(())
        }
    }
}

/// # Safety
/// `ptr` must be *borrowed*, non-null, and a valid Python object.
#[inline]
unsafe fn write_array(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &PySqlDialect,
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

    let close = match dialect.0.array_format {
        None => {
            return Err(FormatError::Unsupported("ArraySerializer").into());
        }
        Some(ArrayFormat::Constructor) => {
            out.push_str("ARRAY[");
            ']'
        }
        Some(ArrayFormat::List) => {
            out.push('[');
            ']'
        }
        Some(ArrayFormat::Function) => {
            out.push_str("array(");
            ')'
        }
    };

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
    out.push(close);
    Ok(())
}

/// # Safety
/// `ptr` must be *borrowed*, non-null, and a valid Python object.
#[inline]
unsafe fn write_custom(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &PySqlDialect,
    function: &pyo3::Py<pyo3::PyAny>,
    object: &alias::BoundObject<'_>,
    out: &mut String,
) -> pyo3::PyResult<()> {
    let _ = ptr;

    let output = function.call1(py, (dialect.clone(), object))?;
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
