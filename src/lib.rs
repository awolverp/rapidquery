#![allow(unused_unsafe)]
#![allow(clippy::macro_metavars_in_unsafe)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![warn(clippy::print_stdout)]
#![warn(clippy::print_stderr)]
#![warn(clippy::dbg_macro)]
#![feature(optimize_attribute)]
#![feature(once_cell_try)]

// TODO List
// 1.  [x] Select statement
// 2.  [x] Case statement
// 3.  [x] Complete Expr
// 4.  [x] Export package classes
// 5.  [ ] Update docstrings & documentation
// 6.  [ ] Write tests
// 7.  [ ] Update & automate workflows
// 8.  [ ] Write CTE
// 9.  [ ] Bump version to 0.1.0
// 10. [ ] Publish
// 11. [ ] Complete Dialect-Only functions

pub mod internal;
mod typeref;

mod common;
mod mysql;
mod postgres;
mod query;
mod schema;
mod sqlite;
mod sqltypes;

#[pyo3::pymodule(gil_used = false)]
mod _lib {
    use pyo3::types::PyModuleMethods;

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
        m.add(
            "__stub_imports__",
            vec![
                "from . import sqltypes as sqltypes",
                "from . import schema as schema",
                "from . import query as query",
                "from . import common as common",
                "from . import sqlite as sqlite",
                "from . import postgres as postgres",
                "from . import mysql as mysql",
            ],
        )?;

        crate::typeref::initialize_typeref(m.py());
        Ok(())
    }
}
