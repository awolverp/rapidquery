#![allow(unsafe_op_in_unsafe_fn)]
#![cfg_attr(feature = "nightly", feature(optimize_attribute))]

pub mod data_type;
pub mod dialect;
pub mod ident;
pub mod internal;
pub mod sql;
pub mod typeref;
pub mod value;

/// RapidQuery native extension.
#[pyo3::pymodule(gil_used = false)]
mod _core {
    #[pymodule_export]
    use super::data_type::data_type_mod;

    #[pymodule_export]
    use super::dialect::dialect_mod;

    #[pymodule_export]
    use super::ident::{PyAsterisk, PyIdent, PyUnquote};

    #[pymodule_export]
    use super::value::PyValue;

    #[pymodule_init]
    #[cold]
    fn init(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
        use crate::internal::utils::PyModuleSubmoduleExt;

        // Cache PyTypeObject* before any Python code can extract a dialect.
        crate::typeref::initialize_typeref(m.py());
        m.init_submodules()
    }
}
