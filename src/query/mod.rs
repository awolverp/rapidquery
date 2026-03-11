pub mod base;
pub mod delete;
pub mod insert;
pub mod on_conflict;
pub mod ordering;
pub mod returning;
pub mod update;
pub mod window;

#[pyo3::pymodule(name = "query")]
pub mod query_module {
    use pyo3::types::PyModuleMethods;

    #[pymodule_export]
    use super::base::PyQueryStatement;

    #[pymodule_export]
    use super::on_conflict::PyOnConflict;

    #[pymodule_export]
    use super::returning::PyReturning;

    #[pymodule_export]
    use super::insert::PyInsertStatement;

    #[pymodule_export]
    use super::ordering::PyOrdering;

    #[pymodule_export]
    use super::delete::PyDeleteStatement;

    #[pymodule_export]
    use super::update::PyUpdateStatement;

    #[pymodule_export]
    use super::window::PyFrame;

    #[pymodule_export]
    use super::window::PyWindowStatement;

    #[pymodule_init]
    #[cold]
    fn init(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
        m.add(
            "__stub_imports__",
            vec![
                "from .common import Value, Expr, Column, ColumnRef, TableName",
                "from .schema import Table",
            ],
        )
    }
}
