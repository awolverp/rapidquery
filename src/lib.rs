#![warn(clippy::print_stdout)]
#![warn(clippy::print_stderr)]
#![warn(clippy::dbg_macro)]
#![feature(likely_unlikely)]
#![feature(optimize_attribute)]
#![feature(once_cell_try)]

#[macro_use]
pub mod macro_rules;

pub mod ast;
pub mod dialect;
pub mod internal;
pub mod typeref;

/// RapidQuery core which is written in Rust.
#[pyo3::pymodule(gil_used = false)]
mod _lib {
    #[pymodule_export]
    pub use super::ast::ast_module;

    #[pymodule_export]
    pub use super::dialect::dialect_module;

    #[pymodule_init]
    #[cold]
    fn init(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
        use crate::internal::utils::PyModuleSubmoduleExt;

        crate::typeref::initialize_typeref(m.py());

        m.init_submodules()
    }
}
