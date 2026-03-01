use crate::common::PySchemaStatement;
use crate::common::PyTableName;
use crate::utils::ToSeaQuery;

implement_state_pyclass! {
    /// Represents a DROP TABLE SQL statement.
    ///
    /// Builds table deletion statements with support for:
    /// - Conditional deletion (IF EXISTS) to avoid errors
    /// - CASCADE to drop dependent objects
    /// - RESTRICT to prevent deletion if dependencies exist
    ///
    /// @signature (name: Table | TableName | str, options: int = 0)
    pub struct [extends=PySchemaStatement] PyDropTable(DropTableState) as "DropTable" {
        name: PyTableName,
        options: u8,
    }
}
implement_state_pyclass! {
    /// Represents a RENAME TABLE SQL statement.
    ///
    /// Changes the name of an existing table to a new name. Both names can be
    /// schema-qualified if needed.
    ///
    /// @signature (from_name: Table | TableName | str, to_name: Table | TableName | str)
    pub struct [extends=PySchemaStatement] PyRenameTable(RenameTableState) as "RenameTable" {
        from_name: PyTableName,
        to_name: PyTableName,
    }
}
implement_state_pyclass! {
    /// Represents a TRUNCATE TABLE SQL statement.
    ///
    /// Quickly removes all rows from a table, typically faster than DELETE
    /// and with different transaction and trigger behavior depending on the
    /// database system.
    ///
    /// @signature (name: Table | TableName | str)
    pub struct [extends=PySchemaStatement] PyTruncateTable(TruncateTableState) as "TruncateTable" {
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

        if self.options & PyDropTable::OPT_IF_EXISTS > 0 {
            stmt.if_exists();
        }
        if self.options & PyDropTable::OPT_RESTRICT > 0 {
            stmt.restrict();
        }
        if self.options & PyDropTable::OPT_CASCADE > 0 {
            stmt.cascade();
        }

        stmt
    }
}

#[pyo3::pymethods]
impl PyDropTable {
    #[classattr]
    pub const OPT_IF_EXISTS: u8 = 1 << 0;
    #[classattr]
    pub const OPT_CASCADE: u8 = 1 << 1;
    #[classattr]
    pub const OPT_RESTRICT: u8 = 1 << 2;

    #[new]
    #[pyo3(signature = (name, options=0))]
    fn __new__(
        name: &pyo3::Bound<'_, pyo3::PyAny>,
        options: u8,
    ) -> pyo3::PyResult<(Self, PySchemaStatement)> {
        let name = PyTableName::try_from(name)?;

        let state = DropTableState { name, options };
        Ok((state.into(), PySchemaStatement))
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

    /// Specified options.
    ///
    /// @signature (self) -> int
    /// @setter int
    #[getter]
    fn options(&self) -> u8 {
        self.0.lock().options
    }

    #[setter]
    fn set_options(&self, val: u8) {
        let mut lock = self.0.lock();
        lock.options = val;
    }

    /// Shorthand for `self.options & OPT_IF_EXISTS > 0`.
    ///
    /// @signature (self) -> bool
    #[getter]
    fn if_exists(&self) -> bool {
        self.0.lock().options & Self::OPT_IF_EXISTS > 0
    }

    /// Shorthand for `self.options & OPT_CASCADE > 0`.
    ///
    /// @signature (self) -> bool
    #[getter]
    fn cascade(&self) -> bool {
        self.0.lock().options & Self::OPT_CASCADE > 0
    }

    /// Shorthand for `self.options & OPT_RESTRICT > 0`.
    ///
    /// @signature (self) -> bool
    #[getter]
    fn restrict(&self) -> bool {
        self.0.lock().options & Self::OPT_RESTRICT > 0
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

        build_schema_statement!(backend, stmt)
    }

    fn __repr__(&self) -> String {
        use std::io::Write;

        let lock = self.0.lock();
        let mut s: Vec<u8> = Vec::with_capacity(20);

        write!(s, "<DropTable {}", lock.name.__repr__()).unwrap();

        if lock.options & Self::OPT_IF_EXISTS > 0 {
            write!(s, " OPT_IF_EXISTS").unwrap();
        }
        if lock.options & Self::OPT_CASCADE > 0 {
            write!(s, " OPT_CASCADE").unwrap();
        }
        if lock.options & Self::OPT_RESTRICT > 0 {
            write!(s, " OPT_RESTRICT").unwrap();
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
    #[pyo3(signature = (from_name, to_name))]
    fn __new__(
        from_name: &pyo3::Bound<'_, pyo3::PyAny>,
        to_name: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<(Self, PySchemaStatement)> {
        let from_name = PyTableName::try_from(from_name)?;
        let to_name = PyTableName::try_from(to_name)?;

        let state = RenameTableState { from_name, to_name };
        Ok((state.into(), PySchemaStatement))
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

        build_schema_statement!(backend, stmt)
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
    #[pyo3(signature = (name))]
    fn __new__(name: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<(Self, PySchemaStatement)> {
        let name = PyTableName::try_from(name)?;

        let state = TruncateTableState { name };
        Ok((state.into(), PySchemaStatement))
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

        build_schema_statement!(backend, stmt)
    }

    fn __repr__(&self) -> String {
        let lock = self.0.lock();
        format!("<TruncateTable name={}>", lock.name.__repr__())
    }
}
