use pyo3::types::PyAnyMethods;
use pyo3::types::PyTzInfoAccess;

use crate::dialect::PyDialect;
use crate::dialect::errors::FormatError;
use crate::dialect::flags;
use crate::internal::alias;
use crate::internal::utils;
use std::fmt::Write;
use std::str::FromStr;

/// `str(obj)` copied into a Rust `String`.
///
/// # Safety
/// `ptr` must be *borrowed*, non-null, and a valid Python object.
#[inline]
pub(super) unsafe fn pyobject_string(py: pyo3::Python<'_>, ptr: *mut pyo3::ffi::PyObject) -> pyo3::PyResult<String> {
    let s = pyo3::ffi::PyObject_Str(ptr);
    if s.is_null() {
        return Err(pyo3::PyErr::fetch(py));
    }
    let out = utils::borrow_py_str_buffer(py, s).map(str::to_owned);
    pyo3::ffi::Py_DECREF(s);
    out
}

fn write_quoted(dialect: &flags::DialectFlags, out: &mut String, raw: &str) -> pyo3::PyResult<()> {
    dialect.write_string_literal(out, raw).map_err(Into::into)
}

fn write_with_keyword(
    dialect: &flags::DialectFlags,
    out: &mut String,
    keyword: Option<&str>,
    raw: &str,
) -> pyo3::PyResult<()> {
    if let Some(keyword) = keyword {
        out.push_str(keyword);
        out.push(' ');
    }
    write_quoted(dialect, out, raw)
}

fn timestamp_keyword(format: flags::TimestampFormat, aware: bool) -> Option<&'static str> {
    match format {
        flags::TimestampFormat::Quoted => None,
        flags::TimestampFormat::Timestamptz => Some(if aware { "TIMESTAMPTZ" } else { "TIMESTAMP" }),
        flags::TimestampFormat::WithTimezone => Some(if aware { "TIMESTAMP WITH TIMEZONE" } else { "TIMESTAMP" }),
    }
}

#[inline]
pub(super) unsafe fn write_integer(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    out: &mut String,
) -> pyo3::PyResult<()> {
    if pyo3::ffi::PyLong_CheckExact(ptr) != 1 {
        return rapidquery_macros::pyerr!(
            Err(PyTypeError),
            "expected int for integer serialization, got {}",
            utils::get_type_name(py, ptr)
        );
    }

    let mut overflow = 0i32;
    let n = pyo3::ffi::PyLong_AsLongLongAndOverflow(ptr, &mut overflow);

    if overflow != 0 {
        // Fallback to `str(obj)` for very long integers.
        let text = pyobject_string(py, ptr)?;
        out.push_str(&text);
        return Ok(());
    }

    if n == -1 && !pyo3::ffi::PyErr_Occurred().is_null() {
        return Err(pyo3::PyErr::fetch(py));
    }

    let _ = write!(out, "{n}");
    Ok(())
}

#[inline]
pub(super) unsafe fn write_float(
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

        // `{:?}` so `1.0` stays  distinct from `1`
        let _ = write!(out, "{v:?}");
        return Ok(());
    }

    if pyo3::ffi::PyLong_CheckExact(ptr) == 1 || crate::typeref::check_exact(ptr, crate::typeref::STD_DECIMAL_TYPE) {
        // Fallback to `str(obj)` for int/decimal.Decimal
        let text = pyobject_string(py, ptr)?;
        out.push_str(&text);
        return Ok(());
    }

    rapidquery_macros::pyerr!(
        Err(PyTypeError),
        "expected float/int/Decimal for numeric serialization, got {}",
        utils::get_type_name(py, ptr)
    )
}

#[inline]
pub(super) unsafe fn write_bool(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &flags::DialectFlags,
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
            utils::get_type_name(py, ptr)
        )
    }
}

#[inline]
pub(super) unsafe fn write_str(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &flags::DialectFlags,
    out: &mut String,
) -> pyo3::PyResult<()> {
    if pyo3::ffi::PyUnicode_Check(ptr) == 1 {
        return dialect
            .write_string_literal(out, utils::borrow_py_str_buffer(py, ptr)?)
            .map_err(Into::into);
    }

    rapidquery_macros::pyerr!(
        Err(PyTypeError),
        "expected str for string serialization, got {}",
        utils::get_type_name(py, ptr)
    )
}

#[inline]
pub(super) unsafe fn write_uuid(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &flags::DialectFlags,
    out: &mut String,
) -> pyo3::PyResult<()> {
    let text = if crate::typeref::check_exact(ptr, crate::typeref::STD_UUID_TYPE) {
        pyobject_string(py, ptr)?
    } else if pyo3::ffi::PyUnicode_CheckExact(ptr) == 1 {
        let borrowed = utils::borrow_py_str_buffer(py, ptr)?;
        uuid::Uuid::try_parse(borrowed)
            .map_err(|e| rapidquery_macros::pyerr!(PyValueError, "invalid uuid str: {}", e))?
            .hyphenated()
            .to_string()
    } else {
        return rapidquery_macros::pyerr!(
            Err(PyTypeError),
            "expected uuid.UUID/str for UUID serialization, got {}",
            utils::get_type_name(py, ptr)
        );
    };

    match dialect.uuid_format() {
        flags::UuidFormat::Quoted => write_quoted(dialect, out, &text),
        flags::UuidFormat::UuidKeyword => {
            out.push_str("UUID ");
            write_quoted(dialect, out, &text)
        }
        flags::UuidFormat::CastUuid => {
            write_quoted(dialect, out, &text)?;
            out.push_str("::uuid");
            Ok(())
        }
    }
}

#[inline]
pub(super) unsafe fn write_bytes(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &flags::DialectFlags,
    out: &mut String,
) -> pyo3::PyResult<()> {
    if pyo3::ffi::PyBytes_CheckExact(ptr) == 1 {
        let buf = pyo3::ffi::PyBytes_AsString(ptr) as *const u8;
        let len = pyo3::ffi::PyBytes_Size(ptr) as usize;
        debug_assert!(!buf.is_null());

        dialect
            .format_bytes(out, std::slice::from_raw_parts(buf, len))
            .map_err(Into::into)
    } else {
        rapidquery_macros::pyerr!(
            Err(PyTypeError),
            "expected bytes for binary serialization, got {}",
            utils::get_type_name(py, ptr)
        )
    }
}

#[inline]
pub(super) unsafe fn write_bit(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &flags::DialectFlags,
    out: &mut String,
) -> pyo3::PyResult<()> {
    if pyo3::ffi::PyUnicode_CheckExact(ptr) == 1 {
        let raw = utils::borrow_py_str_buffer(py, ptr)?;

        if raw.is_empty() || !raw.bytes().all(|b| b == b'0' || b == b'1') {
            return rapidquery_macros::pyerr!(
                Err(PyValueError),
                "expected a string of '0'/'1' for bit serialization, got {:?}",
                raw
            );
        }

        dialect.format_bits(out, raw).map_err(Into::into)
    } else {
        rapidquery_macros::pyerr!(
            Err(PyTypeError),
            "expected str of bits for bit serialization, got {}",
            utils::get_type_name(py, ptr)
        )
    }
}

#[inline]
pub(super) unsafe fn write_json(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &flags::DialectFlags,
    out: &mut String,
) -> pyo3::PyResult<()> {
    crate::internal::modules::json_dumps(unsafe { pyo3::Bound::from_borrowed_ptr(py, ptr) }, |text| match dialect
        .json_format()
    {
        flags::JsonFormat::Quoted => write_quoted(dialect, out, text),
        flags::JsonFormat::JsonKeyword => {
            out.push_str("JSON ");
            write_quoted(dialect, out, text)
        }
        flags::JsonFormat::ParseJson => {
            out.push_str("PARSE_JSON(");
            write_quoted(dialect, out, text)?;
            out.push(')');
            Ok(())
        }
    })
}

#[inline]
pub(super) unsafe fn write_date(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &flags::DialectFlags,
    out: &mut String,
) -> pyo3::PyResult<()> {
    if crate::typeref::check_exact(ptr, crate::typeref::STD_DATE_TYPE) {
        let val: pyo3::Bound<'_, pyo3::types::PyDate> = pyo3::Bound::from_borrowed_ptr(py, ptr).cast_into()?;

        let date: chrono::NaiveDate = val.extract()?;
        let mut body = String::new();
        let _ = date.format("%Y-%m-%d").write_to(&mut body);
        let keyword = match dialect.date_format() {
            flags::DateFormat::Quoted => None,
            flags::DateFormat::Keyword => Some("DATE"),
        };
        write_with_keyword(dialect, out, keyword, &body)
    } else {
        rapidquery_macros::pyerr!(
            Err(PyTypeError),
            "expected datetime.date for date serialization, got {}",
            utils::get_type_name(py, ptr)
        )
    }
}

#[inline]
pub(super) unsafe fn write_time(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &flags::DialectFlags,
    out: &mut String,
) -> pyo3::PyResult<()> {
    if crate::typeref::check_exact(ptr, crate::typeref::STD_TIME_TYPE) {
        let val: pyo3::Bound<'_, pyo3::types::PyTime> = pyo3::Bound::from_borrowed_ptr(py, ptr).cast_into()?;

        let time: chrono::NaiveTime = val.extract()?;
        let mut body = String::new();
        let _ = time.format("%H:%M:%S%.f").write_to(&mut body);
        let keyword = match dialect.time_format() {
            flags::TimeFormat::Quoted => None,
            flags::TimeFormat::Keyword => Some("TIME"),
        };
        write_with_keyword(dialect, out, keyword, &body)
    } else {
        rapidquery_macros::pyerr!(
            Err(PyTypeError),
            "expected datetime.time for time serialization, got {}",
            utils::get_type_name(py, ptr)
        )
    }
}

#[inline]
pub(super) unsafe fn write_datetime(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &flags::DialectFlags,
    out: &mut String,
) -> pyo3::PyResult<()> {
    use chrono::TimeZone;

    if crate::typeref::check_exact(ptr, crate::typeref::STD_DATETIME_TYPE) {
        let val: pyo3::Bound<'_, pyo3::types::PyDateTime> = pyo3::Bound::from_borrowed_ptr(py, ptr).cast_into()?;

        if val.get_tzinfo().is_none() {
            let datetime: chrono::NaiveDateTime = val.extract()?;
            let mut body = String::new();
            let _ = datetime.format("%Y-%m-%d %H:%M:%S%.f").write_to(&mut body);
            return write_with_keyword(
                dialect,
                out,
                timestamp_keyword(dialect.timestamp_format(), false),
                &body,
            );
        }

        let datetime: chrono::DateTime<chrono::FixedOffset> = val.extract()?;
        let mut body = String::new();
        let _ = datetime.format("%Y-%m-%d %H:%M:%S%.f%:z").write_to(&mut body);
        return write_with_keyword(dialect, out, timestamp_keyword(dialect.timestamp_format(), true), &body);
    }

    if pyo3::ffi::PyLong_CheckExact(ptr) == 1 {
        let mut has_overflow: i32 = 0;
        let num = pyo3::ffi::PyLong_AsLongLongAndOverflow(ptr, &mut has_overflow);
        if has_overflow != 0 {
            return rapidquery_macros::pyerr!(Err(PyOverflowError), "out of range");
        }

        return match chrono::Utc.timestamp_opt(num, 0) {
            chrono::offset::LocalResult::Single(datetime) => {
                let mut body = String::new();
                let _ = datetime.format("%Y-%m-%d %H:%M:%S%.f%:z").write_to(&mut body);
                write_with_keyword(dialect, out, timestamp_keyword(dialect.timestamp_format(), true), &body)
            }
            _ => rapidquery_macros::pyerr!(
                Err(PyOverflowError),
                "timestamp is invalid for datetime serialization: {}",
                num
            ),
        };
    }

    if pyo3::ffi::PyFloat_CheckExact(ptr) == 1 {
        let num = pyo3::ffi::PyFloat_AsDouble(ptr);
        return match chrono::Utc.timestamp_opt(num.trunc() as i64, (num.fract() * 1_000_000_000.0) as u32) {
            chrono::offset::LocalResult::Single(datetime) => {
                let mut body = String::new();
                let _ = datetime.format("%Y-%m-%d %H:%M:%S%.f%:z").write_to(&mut body);
                write_with_keyword(dialect, out, timestamp_keyword(dialect.timestamp_format(), true), &body)
            }
            _ => rapidquery_macros::pyerr!(
                Err(PyOverflowError),
                "timestamp is invalid for datetime serialization: {}",
                num
            ),
        };
    }

    rapidquery_macros::pyerr!(
        Err(PyTypeError),
        "expected datetime.datetime/int/float for datetime serialization, got {}",
        utils::get_type_name(py, ptr)
    )
}

#[inline]
pub(super) unsafe fn write_inet(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &flags::DialectFlags,
    out: &mut String,
) -> pyo3::PyResult<()> {
    let text = pyobject_string(py, ptr)?;
    let net =
        ipnetwork::IpNetwork::from_str(&text).map_err(|e| rapidquery_macros::pyerr!(PyValueError, e.to_string()))?;
    let body = net.to_string();

    match dialect.inet_format() {
        flags::InetFormat::Unsupported => Err(FormatError::Unsupported("InetSerializer").into()),
        flags::InetFormat::Quoted => write_quoted(dialect, out, &body),
        flags::InetFormat::Cast => {
            write_quoted(dialect, out, &body)?;
            out.push_str("::inet");
            Ok(())
        }
    }
}

#[inline]
pub(super) unsafe fn write_macaddr(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &flags::DialectFlags,
    out: &mut String,
) -> pyo3::PyResult<()> {
    let text = pyobject_string(py, ptr)?;
    let addr =
        mac_address::MacAddress::from_str(&text).map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    let body = addr.to_string();

    match dialect.macaddr_format() {
        flags::MacAddrFormat::Unsupported => Err(FormatError::Unsupported("MacAddressSerializer").into()),
        flags::MacAddrFormat::Quoted => write_quoted(dialect, out, &body),
        flags::MacAddrFormat::Cast => {
            write_quoted(dialect, out, &body)?;
            out.push_str("::macaddr");
            Ok(())
        }
    }
}

pub(super) unsafe fn write_array(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &PyDialect,
    element: &super::inner::SerializerType,
    out: &mut String,
) -> pyo3::PyResult<()> {
    let (len, getter): (
        isize,
        unsafe fn(*mut pyo3::ffi::PyObject, isize) -> *mut pyo3::ffi::PyObject,
    ) = if pyo3::ffi::PyList_CheckExact(ptr) == 1 {
        (pyo3::ffi::PyList_Size(ptr), |p, i| pyo3::ffi::PyList_GetItem(p, i))
    } else if pyo3::ffi::PyTuple_CheckExact(ptr) == 1 {
        (pyo3::ffi::PyTuple_Size(ptr), |p, i| pyo3::ffi::PyTuple_GetItem(p, i))
    } else {
        return rapidquery_macros::pyerr!(
            Err(PyTypeError),
            "expected list/tuple for array serialization, got {}",
            utils::get_type_name(py, ptr)
        );
    };

    let close = match dialect.0.array_format() {
        flags::ArrayFormat::Unsupported => {
            return Err(FormatError::Unsupported("ArraySerializer").into());
        }
        flags::ArrayFormat::Constructor => {
            out.push_str("ARRAY[");
            ']'
        }
        flags::ArrayFormat::List => {
            out.push('[');
            ']'
        }
        flags::ArrayFormat::Function => {
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
        element.serialize(dialect, &borrowed, out)?;
    }
    out.push(close);
    Ok(())
}

#[inline]
pub(super) unsafe fn write_custom(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &PyDialect,
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
            utils::get_type_name(py, output.as_ptr())
        );
    }

    out.push_str(utils::borrow_py_str_buffer(py, output.as_ptr())?);
    Ok(())
}
