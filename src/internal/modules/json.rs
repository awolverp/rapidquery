//! Process-wide JSON dumps: `orjson` if importable, else stdlib `json`.
//!
//! The result is borrowed and passed to a callback so the SQL writer can
//! escape it without an intermediate `String`.

use once_cell::sync::OnceCell;
use pyo3::types::{PyAnyMethods, PyModule, PyTuple};

use crate::internal::utils::py_str;

#[cfg(all(feature = "only-stdlib-json", feature = "only-orjson"))]
compile_error!(
    "Testing features `only-stdlib-json` and `only-orjson` cannot be used at a same time."
);

enum JsonDumper {
    Orjson {
        dumps: pyo3::Py<pyo3::PyAny>,
    },
    Stdlib {
        dumps: pyo3::Py<pyo3::PyAny>,
        /// `("ensure_ascii", "allow_nan", "separators")` for vectorcall.
        kwnames: pyo3::Py<PyTuple>,
        separators: pyo3::Py<PyTuple>,
    },
}

static DUMPER: OnceCell<JsonDumper> = OnceCell::new();

fn init_orjson(module: &pyo3::Bound<'_, PyModule>) -> pyo3::PyResult<JsonDumper> {
    let dumps = module.getattr("dumps")?.unbind();
    Ok(JsonDumper::Orjson { dumps })
}

fn init_stdlib(py: pyo3::Python<'_>) -> pyo3::PyResult<JsonDumper> {
    let module = py.import("json")?;
    let dumps = module.getattr("dumps")?.unbind();
    let kwnames = PyTuple::new(py, ["ensure_ascii", "allow_nan", "separators"])?.unbind();
    let separators = PyTuple::new(py, [",", ":"])?.unbind();

    Ok(JsonDumper::Stdlib {
        dumps,
        kwnames,
        separators,
    })
}

#[cfg(feature = "only-orjson")]
#[cold]
fn init_import(py: pyo3::Python<'_>) -> pyo3::PyResult<JsonDumper> {
    let module = py.import("orjson")?;
    init_orjson(&module)
}

#[cfg(feature = "only-stdlib-json")]
#[cold]
fn init_import(py: pyo3::Python<'_>) -> pyo3::PyResult<JsonDumper> {
    init_stdlib(py)
}

#[cfg(not(any(feature = "only-stdlib-json", feature = "only-orjson")))]
#[cold]
fn init_import(py: pyo3::Python<'_>) -> pyo3::PyResult<JsonDumper> {
    if let Ok(module) = py.import("orjson")
        && let Ok(dumper) = init_orjson(&module)
    {
        return Ok(dumper);
    }
    init_stdlib(py)
}

/// Borrow JSON text from a `bytes` (orjson) or `str` (stdlib) dumps result.
///
/// # Safety
/// `result` is a new reference still owned by the caller. The GIL is held.
/// The `&str` is only valid until `result` is `DECREF`'d.
#[inline]
unsafe fn visit_json_text(
    py: pyo3::Python<'_>,
    result: *mut pyo3::ffi::PyObject,
    f: impl FnOnce(&str) -> pyo3::PyResult<()>,
) -> pyo3::PyResult<()> {
    if pyo3::ffi::PyBytes_CheckExact(result) == 1 {
        let buf = pyo3::ffi::PyBytes_AsString(result) as *const u8;
        let len = pyo3::ffi::PyBytes_Size(result) as usize;
        debug_assert!(!buf.is_null());
        // SAFETY: `orjson.dumps` returns UTF-8 JSON. The buffer is the
        // exact `bytes` payload and stays alive until `result` is DECREF'd.
        let text = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(buf, len)) };
        return f(text);
    }

    if pyo3::ffi::PyUnicode_CheckExact(result) == 1 {
        return f(py_str(py, result)?);
    }

    rapidquery_macros::pyerr!(
        Err(PyTypeError),
        "JSON dumps must return bytes or str, got {}",
        crate::internal::utils::get_type_name(py, result)
    )
}

/// JSON-encode `obj` and pass the text to `f` before dropping the dumps result.
///
/// # Safety
/// `obj` must be a valid borrowed Python object. The GIL must be held.
#[inline]
pub unsafe fn dumps(
    obj: pyo3::Bound<'_, pyo3::PyAny>,
    f: impl FnOnce(&str) -> pyo3::PyResult<()>,
) -> pyo3::PyResult<()> {
    let py = obj.py();
    let dumper = DUMPER.get_or_try_init(|| init_import(py))?;

    let result = match dumper {
        JsonDumper::Orjson { dumps } => unsafe {
            pyo3::ffi::PyObject_CallOneArg(dumps.as_ptr(), obj.as_ptr())
        },
        JsonDumper::Stdlib {
            dumps,
            kwnames,
            separators,
        } => {
            let false_ = unsafe { pyo3::ffi::Py_False() };
            let mut args = [obj.as_ptr(), false_, false_, separators.as_ptr()];
            unsafe {
                pyo3::ffi::PyObject_Vectorcall(
                    dumps.as_ptr(),
                    args.as_mut_ptr(),
                    1,
                    kwnames.as_ptr(),
                )
            }
        }
    };

    if result.is_null() {
        return Err(pyo3::PyErr::fetch(py));
    }

    let out = unsafe { visit_json_text(py, result, f) };
    unsafe { pyo3::ffi::Py_DECREF(result) };
    out
}
