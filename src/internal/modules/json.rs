//! Process-wide JSON dumps: `orjson` if importable, else stdlib `json`.

use once_cell::sync::OnceCell;
use pyo3::types::{PyAnyMethods, PyBytesMethods, PyDict, PyModule, PyStringMethods, PyTuple};

enum JsonDumper {
    Orjson {
        dumps: pyo3::Py<pyo3::PyAny>,
        option: u32,
    },
    Stdlib {
        dumps: pyo3::Py<pyo3::PyAny>,
        separators: pyo3::Py<PyTuple>,
    },
}

static DUMPER: OnceCell<JsonDumper> = OnceCell::new();

fn orjson_option(module: &pyo3::Bound<'_, PyModule>) -> pyo3::PyResult<u32> {
    let flag = |name: &str| module.getattr(name)?.extract::<u32>();

    Ok(flag("OPT_PASSTHROUGH_DATETIME")?
        | flag("OPT_PASSTHROUGH_UUID")?
        | flag("OPT_PASSTHROUGH_DATACLASS")?
        | flag("OPT_PASSTHROUGH_SUBCLASS")?
        | flag("OPT_NON_STR_KEYS")?)
}

fn init_orjson(
    _py: pyo3::Python<'_>,
    module: &pyo3::Bound<'_, PyModule>,
) -> pyo3::PyResult<JsonDumper> {
    let dumps = module.getattr("dumps")?.unbind();
    let option = orjson_option(module)?;
    Ok(JsonDumper::Orjson { dumps, option })
}

fn init_stdlib(py: pyo3::Python<'_>) -> pyo3::PyResult<JsonDumper> {
    let module = py.import("json")?;
    let dumps = module.getattr("dumps")?.unbind();
    let separators = PyTuple::new(py, [",", ":"])?.unbind();
    Ok(JsonDumper::Stdlib { dumps, separators })
}

#[cold]
fn init_import(py: pyo3::Python<'_>) -> pyo3::PyResult<JsonDumper> {
    if let Ok(module) = py.import("orjson")
        && let Ok(dumper) = init_orjson(py, &module)
    {
        return Ok(dumper);
    }
    init_stdlib(py)
}

fn unicode_to_string(obj: pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<String> {
    let s = obj.cast::<pyo3::types::PyString>()?;
    Ok(s.to_str()?.to_owned())
}

fn dumps_result_to_string(obj: pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<String> {
    if let Ok(bytes) = obj.cast::<pyo3::types::PyBytes>() {
        let raw = bytes.as_bytes();
        return std::str::from_utf8(raw)
            .map(str::to_owned)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()));
    }
    unicode_to_string(obj)
}

/// JSON-encode `ptr` via cached `orjson.dumps` or `json.dumps`.
///
/// # Safety
/// `ptr` must be a valid borrowed `PyObject*`. The GIL must be held.
#[inline]
pub unsafe fn dumps(obj: pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<String> {
    let py = obj.py();
    let dumper = DUMPER.get_or_try_init(|| init_import(py))?;

    match dumper {
        JsonDumper::Orjson { dumps, option } => {
            // orjson.dumps(__obj, default=None, option=int)
            let out = dumps.bind(py).call1((&obj, py.None(), option))?;
            dumps_result_to_string(out)
        }
        JsonDumper::Stdlib { dumps, separators } => {
            let kwargs = PyDict::new(py);
            kwargs.set_item("ensure_ascii", false)?;
            kwargs.set_item("allow_nan", false)?;
            kwargs.set_item("separators", separators.bind(py))?;
            let out = dumps.bind(py).call((&obj,), Some(&kwargs))?;
            unicode_to_string(out)
        }
    }
}
