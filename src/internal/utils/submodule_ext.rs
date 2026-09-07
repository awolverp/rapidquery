//! Register nested `#[pymodule]` exports in `sys.modules`.
//!
//! PyO3 nested modules are attributes of the parent, but `import
//! parent.child` looks up `sys.modules["parent.child"]`. Without this walk,
//! `import rapidquery._core.dialect` fails even though `_core.dialect` works.
//!
//! Workaround: <https://github.com/PyO3/pyo3/issues/759#issuecomment-4041245073>
//!
//! The child's `__name__` stays whatever `#[pymodule(name = ...)]` set
//! (`"dialect"`, not `"rapidquery._core.dialect"`). Import still succeeds
//! because the `sys.modules` key is the dotted path.

use pyo3::types::{PyAnyMethods, PyModule, PyModuleMethods};

fn _inner_init(module: &pyo3::Bound<'_, PyModule>, path: &str) -> pyo3::PyResult<()> {
    let py = module.py();
    let sys_modules = py.import("sys")?.getattr("modules")?;
    for submod_name in module.index()? {
        let submod_name = submod_name.extract::<String>()?;
        let submod = module.getattr(&submod_name)?;
        let modpath = format!("{path}.{submod_name}");
        if let Ok(submod) = submod.cast::<PyModule>() {
            _inner_init(submod, &modpath)?;
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
