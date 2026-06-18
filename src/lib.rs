#![allow(unused_unsafe)]
#![allow(clippy::macro_metavars_in_unsafe)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![warn(clippy::print_stdout)]
#![warn(clippy::print_stderr)]
#![warn(clippy::dbg_macro)]

/// RapidQuery core which is written in Rust.
#[pyo3::pymodule(gil_used = false)]
mod _lib {
    #[pymodule_init]
    #[cold]
    fn init(_m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
        Ok(())
    }
}
