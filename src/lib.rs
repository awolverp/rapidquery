#![allow(unused_unsafe)]
#![allow(clippy::macro_metavars_in_unsafe)]
#![allow(clippy::too_many_arguments)]
#![warn(clippy::print_stdout)]
#![warn(clippy::print_stderr)]
#![warn(clippy::dbg_macro)]
#![feature(optimize_attribute)]
#![feature(once_cell_try)]
#![feature(sync_unsafe_cell)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use crate::internal::RefBoundObject;

pub mod internal;
mod typeref;

mod common;
mod query;
mod schema;
mod sqltypes;

/// Create a new `InsertStatement`.
///
/// @signature (table: schema.Table | common.TableName | str) -> query.InsertStatement
#[pyo3::pyfunction]
#[pyo3(name = "insert")]
#[inline]
pub fn py_insert<'a>(
    table: RefBoundObject<'a>,
) -> pyo3::PyResult<pyo3::Bound<'a, query::insert::PyInsertStatement>> {
    let stmt = query::insert::PyInsertStatement::uninit();
    stmt.__init__(table)?;

    pyo3::Bound::new(table.py(), (stmt, query::base::PyQueryStatement))
}

/// Create a new `DeleteStatement`.
///
/// @signature (table: schema.Table | common.TableName | str) -> query.DeleteStatement
#[pyo3::pyfunction]
#[pyo3(name = "delete")]
#[inline]
pub fn py_delete<'a>(
    table: RefBoundObject<'a>,
) -> pyo3::PyResult<pyo3::Bound<'a, query::delete::PyDeleteStatement>> {
    let stmt = query::delete::PyDeleteStatement::uninit();
    stmt.__init__(table)?;

    pyo3::Bound::new(table.py(), (stmt, query::base::PyQueryStatement))
}

/// Create a new `PyUpdateStatement`.
///
/// @signature (table: schema.Table | common.TableName | str) -> query.UpdateStatement
#[pyo3::pyfunction]
#[pyo3(name = "update")]
#[inline]
pub fn py_update<'a>(
    table: RefBoundObject<'a>,
) -> pyo3::PyResult<pyo3::Bound<'a, query::update::PyUpdateStatement>> {
    let stmt = query::update::PyUpdateStatement::uninit();
    stmt.__init__(table)?;

    pyo3::Bound::new(table.py(), (stmt, query::base::PyQueryStatement))
}

/// Create a new `Returning`.
///
/// @signature (*args: common.Column | common.ColumnRef | str) -> query.Returning
#[pyo3::pyfunction]
#[pyo3(name = "returning", signature=(*args))]
#[inline]
pub fn py_returning<'a>(
    args: &pyo3::Bound<'a, pyo3::types::PyTuple>,
) -> pyo3::PyResult<pyo3::Bound<'a, query::returning::PyReturning>> {
    let clause = query::returning::PyReturning::__new__(args)?;

    pyo3::Bound::new(args.py(), clause)
}

/// Create a new `WindowStatement`.
///
/// @signature (*partition_by: common.Expr | common.Column | common.ColumnRef | str) -> query.WindowStatement
#[pyo3::pyfunction]
#[pyo3(name = "window", signature=(*partition_by))]
#[inline(always)]
pub fn py_window<'a>(
    partition_by: pyo3::Bound<'a, pyo3::types::PyTuple>,
) -> pyo3::PyResult<pyo3::Bound<'a, query::window::PyWindowStatement>> {
    let stmt = query::window::PyWindowStatement::uninit();
    stmt.__init__(&partition_by)?;

    pyo3::Bound::new(partition_by.py(), stmt)
}

#[pyo3::pymodule(gil_used = false)]
mod _lib {
    use pyo3::types::PyModuleMethods;

    #[pymodule_export]
    use super::common::common_module;
    #[pymodule_export]
    use super::query::query_module;
    #[pymodule_export]
    use super::schema::schema_module;
    #[pymodule_export]
    use super::sqltypes::sqltypes_module;

    #[pymodule_export]
    use super::py_delete;
    #[pymodule_export]
    use super::py_insert;
    #[pymodule_export]
    use super::py_returning;
    #[pymodule_export]
    use super::py_update;
    #[pymodule_export]
    use super::py_window;

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
            ],
        )?;

        crate::typeref::initialize_typeref(m.py());
        Ok(())
    }
}
