#![allow(unsafe_op_in_unsafe_fn)]

pub mod internal;
pub mod native;
pub mod pybindings;
pub mod typeref;

#[pyo3::pymodule(gil_used = false)]
mod _core {
    #[pymodule_export]
    use super::pybindings::dialect_mod;

    #[pymodule_init]
    #[cold]
    fn init(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
        use crate::internal::utils::PyModuleSubmoduleExt;

        crate::typeref::initialize_typeref(m.py());
        m.init_submodules()
    }
}
