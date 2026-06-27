use pyo3::types::{PyAnyMethods, PyModule, PyModuleMethods};

/// Returns the type name of a [`pyo3::ffi::PyObject`].
///
/// Returns `"<unknown>"` on failure.
#[inline]
pub unsafe fn get_type_name<'a>(py: pyo3::Python<'a>, obj: *mut pyo3::ffi::PyObject) -> String {
    use pyo3::types::{PyStringMethods, PyTypeMethods};

    let type_ = pyo3::ffi::Py_TYPE(obj);

    if type_.is_null() {
        String::from("<unknown>")
    } else {
        let obj = pyo3::types::PyType::from_borrowed_type_ptr(py, type_);

        obj.fully_qualified_name()
            .map(|x| x.to_string_lossy().into_owned())
            .unwrap_or_else(|_| String::from("<unknown>"))
    }
}

// --- https://github.com/PyO3/pyo3/issues/759#issuecomment-4041245073 ---

fn _inner_init(module: &pyo3::Bound<'_, PyModule>, path: &str) -> pyo3::PyResult<()> {
    let py = module.py();
    let sys_modules = py.import("sys")?.getattr("modules")?;
    for submod_name in module.index()? {
        let submod_name = submod_name.extract::<String>()?;
        let submod = module.getattr(&submod_name)?;
        let modpath = format!("{path}.{submod_name}");
        if let Ok(submod) = submod.cast::<PyModule>() {
            _inner_init(&submod, &modpath)?;
            sys_modules.set_item(&modpath, submod)?;
        }
    }
    Ok(())
}

pub trait PyModuleSubmoduleExt {
    fn init_submodules(&self) -> pyo3::PyResult<()>;
}

impl PyModuleSubmoduleExt for pyo3::Bound<'_, pyo3::types::PyModule> {
    fn init_submodules(&self) -> pyo3::PyResult<()> {
        let mod_path = self.name()?.extract::<String>()?;
        _inner_init(self, &mod_path)?;
        Ok(())
    }
}
