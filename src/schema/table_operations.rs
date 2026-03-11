use super::base::PySchemaStatement;
use crate::common::table_ref::PyTableName;
use crate::internal::statements::ToSeaQuery;

pub const DROP_OPT_IF_EXISTS: u8 = 1 << 0;
pub const DROP_OPT_CASCADE: u8 = 1 << 1;
pub const DROP_OPT_RESTRICT: u8 = 1 << 2;

crate::implement_pyclass! {
    /// Represents a DROP TABLE SQL statement.
    ///
    /// Builds table deletion statements with support for:
    /// - Conditional deletion (IF EXISTS) to avoid errors
    /// - CASCADE to drop dependent objects
    /// - RESTRICT to prevent deletion if dependencies exist
    ///
    /// @signature (
    ///     self,
    ///     name: Table | TableName | str,
    ///     *,
    ///     if_exists: bool = False,
    ///     cascade: bool = False,
    ///     restrict: bool = False,
    /// )
    mutable [subclass, extends=PySchemaStatement] PyDropTable(DropTableState) as "DropTable" {
        name: PyTableName,
        options: u8,
    }
}
crate::implement_pyclass! {
    /// Represents a RENAME TABLE SQL statement.
    ///
    /// Changes the name of an existing table to a new name. Both names can be
    /// schema-qualified if needed.
    ///
    /// @signature (self, from_name: Table | TableName | str, to_name: Table | TableName | str)
    mutable [subclass, extends=PySchemaStatement] PyRenameTable(RenameTableState) as "RenameTable" {
        from_name: PyTableName,
        to_name: PyTableName,
    }
}
crate::implement_pyclass! {
    /// Represents a TRUNCATE TABLE SQL statement.
    ///
    /// Quickly removes all rows from a table, typically faster than DELETE
    /// and with different transaction and trigger behavior depending on the
    /// database system.
    ///
    /// @signature (self, name: Table | TableName | str)
    mutable [subclass, extends=PySchemaStatement] PyTruncateTable(TruncateTableState) as "TruncateTable" {
        name: PyTableName,
    }
}

impl Clone for DropTableState {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            options: self.options,
        }
    }
}

impl ToSeaQuery<sea_query::TableDropStatement> for DropTableState {
    fn to_sea_query<'a>(&self, _py: pyo3::Python<'a>) -> sea_query::TableDropStatement {
        let mut stmt = sea_query::TableDropStatement::new();
        stmt.table(self.name.clone());

        if self.options & DROP_OPT_IF_EXISTS > 0 {
            stmt.if_exists();
        }
        if self.options & DROP_OPT_RESTRICT > 0 {
            stmt.restrict();
        }
        if self.options & DROP_OPT_CASCADE > 0 {
            stmt.cascade();
        }

        stmt
    }
}

#[pyo3::pymethods]
impl PyDropTable {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(
        args: &pyo3::Bound<'_, pyo3::types::PyTuple>,
        kwds: Option<&pyo3::Bound<'_, pyo3::types::PyDict>>,
    ) -> (Self, PySchemaStatement) {
        (Self::uninit(), PySchemaStatement)
    }

    #[pyo3(signature = (name, *, if_exists=false, cascade=false, restrict=false))]
    fn __init__(
        &self,
        name: &pyo3::Bound<'_, pyo3::PyAny>,
        if_exists: bool,
        cascade: bool,
        restrict: bool,
    ) -> pyo3::PyResult<()> {
        let name = PyTableName::try_from(name)?;

        let mut options = 0u8;
        if if_exists {
            options |= DROP_OPT_IF_EXISTS;
        }
        if cascade {
            options |= DROP_OPT_CASCADE;
        }
        if restrict {
            options |= DROP_OPT_RESTRICT;
        }

        let state = DropTableState { name, options };
        self.0.set(state);
        Ok(())
    }

    /// The table name to drop.
    ///
    /// @signature (self) -> TableName
    /// @setter Table | TableName | str
    #[getter]
    fn name(&self) -> PyTableName {
        let lock = self.0.lock();
        lock.name.clone()
    }

    #[setter]
    fn set_name(&self, val: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        let val = PyTableName::try_from(val)?;

        let mut lock = self.0.lock();
        lock.name = val;
        Ok(())
    }

    /// @signature (self) -> bool
    /// @setter bool
    #[getter]
    fn if_exists(&self) -> bool {
        self.0.lock().options & DROP_OPT_IF_EXISTS > 0
    }

    #[setter]
    fn set_if_exists(&self, value: bool) {
        let mut lock = self.0.lock();
        if value {
            lock.options |= DROP_OPT_IF_EXISTS;
        } else {
            lock.options &= !DROP_OPT_IF_EXISTS;
        }
    }

    /// @signature (self) -> bool
    /// @setter bool
    #[getter]
    fn cascade(&self) -> bool {
        self.0.lock().options & DROP_OPT_CASCADE > 0
    }

    #[setter]
    fn set_cascade(&self, value: bool) {
        let mut lock = self.0.lock();
        if value {
            lock.options |= DROP_OPT_CASCADE;
        } else {
            lock.options &= !DROP_OPT_CASCADE;
        }
    }

    /// @signature (self) -> bool
    /// @setter bool
    #[getter]
    fn restrict(&self) -> bool {
        self.0.lock().options & DROP_OPT_RESTRICT > 0
    }

    #[setter]
    fn set_restrict(&self, value: bool) {
        let mut lock = self.0.lock();
        if value {
            lock.options |= DROP_OPT_RESTRICT;
        } else {
            lock.options &= !DROP_OPT_RESTRICT;
        }
    }

    fn __copy__(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::Py<Self>> {
        let lock = self.0.lock();
        pyo3::Py::new(py, (lock.clone().into(), PySchemaStatement))
    }

    #[pyo3(signature = (backend, /))]
    #[allow(clippy::wrong_self_convention)]
    fn to_sql(&self, py: pyo3::Python<'_>, backend: String) -> pyo3::PyResult<String> {
        let lock = self.0.lock();
        let stmt = lock.to_sea_query(py);
        drop(lock);

        crate::build_schema_statement!(backend, stmt)
    }

    fn __repr__(&self) -> String {
        use std::io::Write;

        let lock = self.0.lock();
        let mut s: Vec<u8> = Vec::with_capacity(20);

        write!(s, "<DropTable {}", lock.name.__repr__()).unwrap();

        if lock.options & DROP_OPT_IF_EXISTS > 0 {
            write!(s, " if_exists=True").unwrap();
        }
        if lock.options & DROP_OPT_CASCADE > 0 {
            write!(s, " cascade=True").unwrap();
        }
        if lock.options & DROP_OPT_RESTRICT > 0 {
            write!(s, " restrict=True").unwrap();
        }
        write!(s, ">").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}

impl Clone for RenameTableState {
    fn clone(&self) -> Self {
        Self {
            from_name: self.from_name.clone(),
            to_name: self.to_name.clone(),
        }
    }
}

impl ToSeaQuery<sea_query::TableRenameStatement> for RenameTableState {
    fn to_sea_query<'a>(&self, _py: pyo3::Python<'a>) -> sea_query::TableRenameStatement {
        let mut stmt = sea_query::TableRenameStatement::new();
        stmt.table(self.from_name.clone(), self.to_name.clone());
        stmt
    }
}

#[pyo3::pymethods]
impl PyRenameTable {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(
        args: &pyo3::Bound<'_, pyo3::types::PyTuple>,
        kwds: Option<&pyo3::Bound<'_, pyo3::types::PyDict>>,
    ) -> (Self, PySchemaStatement) {
        (Self::uninit(), PySchemaStatement)
    }

    #[pyo3(signature = (from_name, to_name))]
    fn __init__(
        &self,
        from_name: &pyo3::Bound<'_, pyo3::PyAny>,
        to_name: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<()> {
        let from_name = PyTableName::try_from(from_name)?;
        let to_name = PyTableName::try_from(to_name)?;

        let state = RenameTableState { from_name, to_name };
        self.0.set(state);
        Ok(())
    }

    /// The current name of the table.
    ///
    /// @signature (self) -> TableName
    /// @setter Table | TableName | str
    #[getter]
    #[allow(clippy::wrong_self_convention)]
    fn from_name(&self) -> PyTableName {
        let lock = self.0.lock();
        lock.from_name.clone()
    }

    #[setter]
    fn set_from_name(&self, val: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        let val = PyTableName::try_from(val)?;

        let mut lock = self.0.lock();
        lock.from_name = val;
        Ok(())
    }

    /// The new name for the table.
    ///
    /// @signature (self) -> TableName
    /// @setter Table | TableName | str
    #[getter]
    fn to_name(&self) -> PyTableName {
        let lock = self.0.lock();
        lock.to_name.clone()
    }

    #[setter]
    fn set_to_name(&self, val: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        let val = PyTableName::try_from(val)?;

        let mut lock = self.0.lock();
        lock.to_name = val;
        Ok(())
    }

    fn __copy__(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::Py<Self>> {
        let lock = self.0.lock();
        pyo3::Py::new(py, (lock.clone().into(), PySchemaStatement))
    }

    #[pyo3(signature = (backend, /))]
    #[allow(clippy::wrong_self_convention)]
    fn to_sql(&self, py: pyo3::Python<'_>, backend: String) -> pyo3::PyResult<String> {
        let lock = self.0.lock();
        let stmt = lock.to_sea_query(py);
        drop(lock);

        crate::build_schema_statement!(backend, stmt)
    }

    fn __repr__(&self) -> String {
        let lock = self.0.lock();
        format!(
            "<RenameTable from_name={} to_name={}>",
            lock.from_name.__repr__(),
            lock.to_name.__repr__()
        )
    }
}

impl Clone for TruncateTableState {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
        }
    }
}

impl ToSeaQuery<sea_query::TableTruncateStatement> for TruncateTableState {
    fn to_sea_query<'a>(&self, _py: pyo3::Python<'a>) -> sea_query::TableTruncateStatement {
        let mut stmt = sea_query::TableTruncateStatement::new();
        stmt.table(self.name.clone());
        stmt
    }
}

#[pyo3::pymethods]
impl PyTruncateTable {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(
        args: &pyo3::Bound<'_, pyo3::types::PyTuple>,
        kwds: Option<&pyo3::Bound<'_, pyo3::types::PyDict>>,
    ) -> (Self, PySchemaStatement) {
        (Self::uninit(), PySchemaStatement)
    }

    #[pyo3(signature = (name))]
    fn __init__(&self, name: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        let name = PyTableName::try_from(name)?;

        let state = TruncateTableState { name };
        self.0.set(state);
        Ok(())
    }

    /// The name of the table to truncate.
    ///
    /// @signature (self) -> TableName
    /// @setter Table | TableName | str
    #[getter]
    fn name(&self) -> PyTableName {
        let lock = self.0.lock();
        lock.name.clone()
    }

    #[setter]
    fn set_name(&self, val: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        let val = PyTableName::try_from(val)?;

        let mut lock = self.0.lock();
        lock.name = val;
        Ok(())
    }

    fn __copy__(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::Py<Self>> {
        let lock = self.0.lock();
        pyo3::Py::new(py, (lock.clone().into(), PySchemaStatement))
    }

    #[pyo3(signature = (backend, /))]
    #[allow(clippy::wrong_self_convention)]
    fn to_sql(&self, py: pyo3::Python<'_>, backend: String) -> pyo3::PyResult<String> {
        let lock = self.0.lock();
        let stmt = lock.to_sea_query(py);
        drop(lock);

        crate::build_schema_statement!(backend, stmt)
    }

    fn __repr__(&self) -> String {
        let lock = self.0.lock();
        format!("<TruncateTable name={}>", lock.name.__repr__())
    }
}
