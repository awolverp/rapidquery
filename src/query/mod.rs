pub mod base;
pub mod case;
pub mod delete;
pub mod insert;
pub mod on_conflict;
pub mod ordering;
pub mod returning;
pub mod select;
pub mod update;
pub mod window;
pub mod with;

#[pyo3::pymodule(name = "query")]
pub mod query_module {
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

    #[pymodule_export]
    use super::select::PySelectLabel;

    #[pymodule_export]
    use super::select::PySelectStatement;

    #[pymodule_export]
    use super::case::PyCaseStatement;

    #[pymodule_export]
    use super::with::PyWithClause;

    #[pymodule_export]
    use super::with::PyWithQuery;
}
