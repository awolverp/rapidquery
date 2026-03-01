pub mod delete;
pub mod insert;
pub mod on_conflict;
pub mod ordering;
pub mod returning;
pub mod select;
pub mod update;
pub mod window;

/// Create a new `DeleteStatement`.
///
/// @signature (table: Table | TableName | str) -> DeleteStatement
#[pyo3::pyfunction]
#[pyo3(name = "delete")]
#[inline(always)]
pub fn py_delete<'a>(
    table: &pyo3::Bound<'a, pyo3::PyAny>,
) -> pyo3::PyResult<pyo3::Bound<'a, delete::PyDeleteStatement>> {
    let value = delete::PyDeleteStatement::__new__(table)?;

    pyo3::Bound::new(table.py(), value)
}

/// Create a new `InsertStatement`.
///
/// @signature (table: Table | TableName | str) -> InsertStatement
#[pyo3::pyfunction]
#[pyo3(name = "insert")]
#[inline(always)]
pub fn py_insert<'a>(
    table: &pyo3::Bound<'a, pyo3::PyAny>,
) -> pyo3::PyResult<pyo3::Bound<'a, insert::PyInsertStatement>> {
    let value = insert::PyInsertStatement::__new__(table)?;

    pyo3::Bound::new(table.py(), value)
}

/// Create a new `UpdateStatement`.
///
/// @signature (table: Table | TableName | str) -> UpdateStatement
#[pyo3::pyfunction]
#[pyo3(name = "update")]
#[inline(always)]
pub fn py_update<'a>(
    table: &pyo3::Bound<'a, pyo3::PyAny>,
) -> pyo3::PyResult<pyo3::Bound<'a, update::PyUpdateStatement>> {
    let value = update::PyUpdateStatement::__new__(table)?;

    pyo3::Bound::new(table.py(), value)
}
