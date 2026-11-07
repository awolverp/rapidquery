use std::str::FromStr;

use pyo3::types::PyAnyMethods;
use pyo3::types::PyTzInfoAccess;

use crate::dialect;
use crate::exc;
use crate::internal::alias;
use crate::internal::utils;
use crate::internal::writer;

/// # Safety
/// `ptr` must be a borrowed, non-null Python object. The GIL must be held.
unsafe fn reject_type(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    what: &'static str,
    expected: &'static str,
) -> exc::Error {
    exc::Error::serialize_type(what, expected, &alias::BoundObject::from_borrowed_ptr(py, ptr))
}

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

#[inline]
pub(super) unsafe fn write_integer<W: writer::FastWrite>(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    out: &mut W,
) -> exc::Result<()> {
    if pyo3::ffi::PyLong_Check(ptr) != 1 {
        return Err(reject_type(py, ptr, "IntegerSerializer", "int"));
    }

    let mut overflow = 0i32;
    let n = pyo3::ffi::PyLong_AsLongLongAndOverflow(ptr, &mut overflow);

    if overflow != 0 {
        // Fallback to `str(obj)` for very long integers.
        let text = pyobject_string(py, ptr)?;
        out.write_str(&text);
        return Ok(());
    }

    if n == -1 && !pyo3::ffi::PyErr_Occurred().is_null() {
        return Err(pyo3::PyErr::fetch(py).into());
    }

    out.write_int(n);
    Ok(())
}

#[inline]
pub(super) unsafe fn write_float<W: writer::FastWrite>(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    out: &mut W,
) -> exc::Result<()> {
    if pyo3::ffi::PyFloat_CheckExact(ptr) == 1 {
        let v = pyo3::ffi::PyFloat_AsDouble(ptr);
        if !v.is_finite() {
            return Err(exc::Error::serialize(
                "FloatSerializer",
                "cannot serialize non-finite float",
            ));
        }

        out.write_float(v);
        return Ok(());
    }

    if pyo3::ffi::PyLong_CheckExact(ptr) == 1 || crate::typeref::check_exact(ptr, crate::typeref::STD_DECIMAL_TYPE) {
        // Fallback to `str(obj)` for int/decimal.Decimal
        let text = pyobject_string(py, ptr)?;
        out.write_str(&text);
        return Ok(());
    }

    Err(reject_type(
        py,
        ptr,
        "FloatSerializer",
        "float, int, or decimal.Decimal",
    ))
}

#[inline]
pub(super) unsafe fn write_bool<W: writer::FastWrite>(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    cfg: &dialect::LiteralConfig,
    out: &mut W,
) -> exc::Result<()> {
    if pyo3::ffi::PyBool_Check(ptr) == 1 {
        let value = pyo3::ffi::Py_True() == ptr;
        cfg.write_boolean(out, value);
        return Ok(());
    }

    Err(reject_type(py, ptr, "BooleanSerializer", "bool"))
}

#[inline]
pub(super) unsafe fn write_str<W: writer::FastWrite>(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    cfg: &dialect::StringConfig,
    out: &mut W,
) -> exc::Result<()> {
    if pyo3::ffi::PyUnicode_Check(ptr) == 1 {
        return cfg
            .write_string_literal(out, utils::borrow_py_str_buffer(py, ptr)?)
            .map_err(Into::into);
    }

    Err(reject_type(py, ptr, "StringSerializer", "str"))
}

#[inline]
pub(super) unsafe fn write_uuid<W: writer::FastWrite>(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    cfg: &dialect::LiteralConfig,
    out: &mut W,
) -> exc::Result<()> {
    let Some(tmpl) = &cfg.uuid else {
        return Err(exc::Error::dialect("UUID literal"));
    };

    let text = {
        // uuid.UUID
        if crate::typeref::check_exact(ptr, crate::typeref::STD_UUID_TYPE) {
            pyobject_string(py, ptr)?
        }
        // str
        else if pyo3::ffi::PyUnicode_CheckExact(ptr) == 1 {
            let borrowed = utils::borrow_py_str_buffer(py, ptr)?;

            uuid::Uuid::try_parse(borrowed)
                .map_err(|e| exc::Error::serialize("UuidSerializer", format!("invalid string uuid: {e}")))?
                .hyphenated()
                .to_string()
        } else {
            return Err(reject_type(py, ptr, "UuidSerializer", "uuid.UUID or str"));
        }
    };

    tmpl.write_to(
        out,
        // no special keys
        |_, out| out.write_str(text.as_str()),
    );
    Ok(())
}

#[inline]
pub(super) unsafe fn write_bytes<W: writer::FastWrite>(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    cfg: &dialect::LiteralConfig,
    out: &mut W,
) -> exc::Result<()> {
    if pyo3::ffi::PyBytes_CheckExact(ptr) == 1 {
        let buf = pyo3::ffi::PyBytes_AsString(ptr) as *const u8;
        let len = pyo3::ffi::PyBytes_Size(ptr) as usize;
        debug_assert!(!buf.is_null());

        cfg.write_bytes(out, std::slice::from_raw_parts(buf, len));
        Ok(())
    } else {
        Err(reject_type(py, ptr, "BytesSerializer", "bytes"))
    }
}

pub(super) unsafe fn write_bit<W: writer::FastWrite>(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    cfg: &dialect::LiteralConfig,
    out: &mut W,
) -> exc::Result<()> {
    if pyo3::ffi::PyUnicode_CheckExact(ptr) == 1 {
        let raw = utils::borrow_py_str_buffer(py, ptr)?;

        if raw.is_empty() || !raw.bytes().all(|b| b == b'0' || b == b'1') {
            return Err(exc::Error::serialize("BitSerializer", "value must be binary ('0'/'1')"));
        }

        cfg.write_bits(out, raw)
    } else {
        Err(reject_type(py, ptr, "BitSerializer", "str"))
    }
}

#[inline]
pub(super) unsafe fn write_json<W: writer::FastWrite>(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    literal_cfg: &dialect::LiteralConfig,
    string_cfg: &dialect::StringConfig,
    out: &mut W,
) -> exc::Result<()> {
    crate::internal::modules::json_dumps(
        unsafe { pyo3::Bound::from_borrowed_ptr(py, ptr) },
        // write result into `out`
        |text| {
            literal_cfg.json.write_to_result(
                out,
                // no special keys
                |_, out| string_cfg.write_string_literal(out, text),
            )?;
            Ok(())
        },
    )
}

#[inline]
pub(super) unsafe fn write_date<W: writer::FastWrite>(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    cfg: &dialect::TemporalConfig,
    out: &mut W,
) -> exc::Result<()> {
    if crate::typeref::check_exact(ptr, crate::typeref::STD_DATE_TYPE) {
        let val: pyo3::Bound<'_, pyo3::types::PyDate> = pyo3::Bound::from_borrowed_ptr(py, ptr).cast_into()?;
        let date: chrono::NaiveDate = val.extract()?;

        cfg.date
            .write_to(out, |_, out| out.write_str(&date.format("%Y-%m-%d").to_string()));
        Ok(())
    } else {
        Err(reject_type(py, ptr, "DateSerializer", "datetime.date"))
    }
}

pub(super) unsafe fn write_time<W: writer::FastWrite>(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    cfg: &dialect::TemporalConfig,
    out: &mut W,
) -> exc::Result<()> {
    if crate::typeref::check_exact(ptr, crate::typeref::STD_TIME_TYPE) {
        let val: pyo3::Bound<'_, pyo3::types::PyTime> = pyo3::Bound::from_borrowed_ptr(py, ptr).cast_into()?;
        let time: chrono::NaiveTime = val.extract()?;

        cfg.time
            .write_to(out, |_, out| out.write_str(&time.format("%H:%M:%S%.f").to_string()));
        Ok(())
    } else {
        Err(reject_type(py, ptr, "TimeSerializer", "datetime.time"))
    }
}

#[inline]
pub(super) unsafe fn write_datetime<W: writer::FastWrite>(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    cfg: &dialect::TemporalConfig,
    out: &mut W,
) -> exc::Result<()> {
    use chrono::TimeZone;

    if crate::typeref::check_exact(ptr, crate::typeref::STD_DATETIME_TYPE) {
        let val: pyo3::Bound<'_, pyo3::types::PyDateTime> = pyo3::Bound::from_borrowed_ptr(py, ptr).cast_into()?;

        if val.get_tzinfo().is_none() {
            let datetime: chrono::NaiveDateTime = val.extract()?;

            cfg.datetime_naive.write_to(out, |_, out| {
                out.write_str(&datetime.format("%Y-%m-%d %H:%M:%S%.f").to_string())
            });
            return Ok(());
        }

        let datetime: chrono::DateTime<chrono::FixedOffset> = val.extract()?;

        cfg.datetime_aware.write_to(out, |_, out| {
            out.write_str(&datetime.format("%Y-%m-%d %H:%M:%S%.f%:z").to_string())
        });
        return Ok(());
    }

    if pyo3::ffi::PyLong_CheckExact(ptr) == 1 {
        let mut has_overflow: i32 = 0;
        let num = pyo3::ffi::PyLong_AsLongLongAndOverflow(ptr, &mut has_overflow);
        if has_overflow != 0 {
            return Err(exc::Error::serialize("DatetimeSerializer", "is out of range"));
        }

        return match chrono::Utc.timestamp_opt(num, 0) {
            chrono::offset::LocalResult::Single(datetime) => {
                cfg.datetime_aware.write_to(out, |_, out| {
                    out.write_str(&datetime.format("%Y-%m-%d %H:%M:%S%.f%:z").to_string())
                });
                Ok(())
            }
            _ => Err(exc::Error::serialize("DatetimeSerializer", "is out of range")),
        };
    }

    if pyo3::ffi::PyFloat_CheckExact(ptr) == 1 {
        let num = pyo3::ffi::PyFloat_AsDouble(ptr);
        return match chrono::Utc.timestamp_opt(num.trunc() as i64, (num.fract() * 1_000_000_000.0) as u32) {
            chrono::offset::LocalResult::Single(datetime) => {
                cfg.datetime_aware.write_to(out, |_, out| {
                    out.write_str(&datetime.format("%Y-%m-%d %H:%M:%S%.f%:z").to_string())
                });
                Ok(())
            }
            _ => Err(exc::Error::serialize("DatetimeSerializer", "is out of range")),
        };
    }

    Err(reject_type(py, ptr, "DatetimeSerializer", "datetime.datetime"))
}

pub(super) unsafe fn write_inet<W: writer::FastWrite>(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    cfg: &dialect::LiteralConfig,
    out: &mut W,
) -> exc::Result<()> {
    let Some(tmpl) = &cfg.inet else {
        return Err(exc::Error::dialect("INET literal"));
    };

    let text = pyobject_string(py, ptr)?;
    let net =
        ipnetwork::IpNetwork::from_str(&text).map_err(|e| exc::Error::serialize("InetSerializer", e.to_string()))?;

    let body = net.to_string();

    tmpl.write_to(out, |_, out| out.write_str(&body));
    Ok(())
}

pub(super) unsafe fn write_macaddr<W: writer::FastWrite>(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    cfg: &dialect::LiteralConfig,
    out: &mut W,
) -> exc::Result<()> {
    let Some(tmpl) = &cfg.macaddr else {
        return Err(exc::Error::dialect("MACADDR literal"));
    };

    let text = pyobject_string(py, ptr)?;
    let addr = mac_address::MacAddress::from_str(&text)
        .map_err(|e| exc::Error::serialize("MacAddressSerializer", e.to_string()))?;

    let body = addr.to_string();

    tmpl.write_to(out, |_, out| out.write_str(&body));
    Ok(())
}

pub(super) unsafe fn write_array<W: writer::FastWrite>(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &dialect::PyDialect,
    element: &super::inner::SerializerType,
    out: &mut W,
) -> exc::Result<()> {
    let Some(tmpl) = &dialect.literal.array else {
        return Err(exc::Error::dialect("ARRAY literal"));
    };

    let (len, getter): (
        isize,
        unsafe fn(*mut pyo3::ffi::PyObject, isize) -> *mut pyo3::ffi::PyObject,
    ) = {
        // list
        if pyo3::ffi::PyList_CheckExact(ptr) == 1 {
            (pyo3::ffi::PyList_Size(ptr), |p, i| pyo3::ffi::PyList_GetItem(p, i))
        }
        // tuple
        else if pyo3::ffi::PyTuple_CheckExact(ptr) == 1 {
            (pyo3::ffi::PyTuple_Size(ptr), |p, i| pyo3::ffi::PyTuple_GetItem(p, i))
        } else {
            return Err(reject_type(py, ptr, "ArraySerializer", "list or tuple"));
        }
    };

    tmpl.write_to_result(
        out,
        // no special keys
        |_, out| {
            for i in 0..len {
                out.write_str_if(i > 0, ", ");

                let item = getter(ptr, i);
                if item.is_null() {
                    return Err(pyo3::PyErr::fetch(py).into());
                }
                let borrowed = alias::BoundObject::from_borrowed_ptr(py, item);
                element.serialize(dialect, &borrowed, out)?;
            }
            Ok(())
        },
    )
}

pub(super) unsafe fn write_custom<W: writer::FastWrite>(
    py: pyo3::Python<'_>,
    ptr: *mut pyo3::ffi::PyObject,
    dialect: &dialect::PyDialect,
    function: &pyo3::Py<pyo3::PyAny>,
    object: &alias::BoundObject<'_>,
    out: &mut W,
) -> exc::Result<()> {
    let _ = ptr;

    let output = function.call1(py, (dialect.clone(), object))?;
    if pyo3::ffi::PyUnicode_CheckExact(output.as_ptr()) != 1 {
        return Err(exc::Error::serialize_type("CustomSerializer", "str", output.bind(py)));
    }

    out.write_str(utils::borrow_py_str_buffer(py, output.as_ptr())?);
    Ok(())
}
