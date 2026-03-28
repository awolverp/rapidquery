#![allow(unused_unsafe)]
#![allow(clippy::macro_metavars_in_unsafe)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![warn(clippy::print_stdout)]
#![warn(clippy::print_stderr)]
#![warn(clippy::dbg_macro)]
#![feature(optimize_attribute)]
#![feature(once_cell_try)]

pub mod internal;
mod typeref;

mod common;
mod mysql;
mod postgres;
mod query;
mod schema;
mod sqlite;
mod sqltypes;

/// RapidQuery core which is written in Rust.
#[pyo3::pymodule(gil_used = false)]
mod _lib {
    #[pymodule_export]
    use super::common::common_module;
    #[pymodule_export]
    use super::mysql::mysql_module;
    #[pymodule_export]
    use super::postgres::postgres_module;
    #[pymodule_export]
    use super::query::query_module;
    #[pymodule_export]
    use super::schema::schema_module;
    #[pymodule_export]
    use super::sqlite::sqlite_module;
    #[pymodule_export]
    use super::sqltypes::sqltypes_module;

    #[pymodule_init]
    #[cold]
    fn init(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
        crate::typeref::initialize_typeref(m.py());
        Ok(())
    }
}
