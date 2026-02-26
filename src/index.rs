use pyo3::types::PyStringMethods;
use sea_query::IntoIden;

use crate::common::PySchemaStatement;
use crate::common::PyTableName;
use crate::expression::PyExpr;
use crate::utils::ToSeaQuery;

#[inline]
fn map_str_to_index_order(value: impl AsRef<str>) -> pyo3::PyResult<sea_query::IndexOrder> {
    match value.as_ref() {
        "asc" | "ASC" => Ok(sea_query::IndexOrder::Asc),
        "desc" | "DESC" => Ok(sea_query::IndexOrder::Desc),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "unknown order: {}",
            value.as_ref()
        ))),
    }
}

#[inline]
fn map_index_order_to_str(value: sea_query::IndexOrder) -> String {
    match value {
        sea_query::IndexOrder::Asc => String::from("ASC"),
        sea_query::IndexOrder::Desc => String::from("DESC"),
    }
}

implement_pyclass! {
    /// Defines a column within an index specification.
    ///
    /// Represents a single column's participation in an index, including:
    /// - The column name
    /// - Optional prefix length (for partial indexing)
    /// - Sort order (ascending or descending)
    ///
    /// Used within `Index` to specify which columns are indexed
    /// and how they should be ordered.
    ///
    /// NOTE: this class is immutable and frozen.
    ///
    /// @alias _IndexColumnOrder = typing.Literal["ASC", "DESC"]
    /// @signature (name: str, order: _IndexColumnOrder | None = None, prefix: int | None = None)
    #[derive(Debug, Clone)]
    pub struct [] PyIndexColumn as "IndexColumn" {
        pub name: String,
        pub order: Option<sea_query::IndexOrder>,
        pub prefix: Option<u32>,
    }
}
implement_state_pyclass! {
    /// Represents a database index specification.
    ///
    /// This class defines the structure and properties of a database index,
    /// including column definitions, uniqueness constraints, index type,
    /// and partial indexing conditions.
    ///
    /// You can use it to generate `CREATE INDEX` SQL expressions.
    ///
    /// @alias _IndexColumnValue = IndexColumn | Column | ColumnRef | str
    /// @signature (
    ///     columns: typing.Iterable[_IndexColumnValue],
    ///     table: TableName | str | None = None,
    ///     name: str | None = None,
    ///     options: int = 0,
    ///     *,
    ///     index_type: str | None = None,
    ///     where: object | None = None,
    ///     include: typing.Iterable[str] = (),
    /// )
    #[derive(Debug)]
    pub struct [extends=PySchemaStatement] PyIndex(IndexState) as "Index" {
        pub name: Option<String>,
        pub columns: Vec<PyIndexColumn>,
        pub table: Option<PyTableName>,
        pub options: u8,
        pub index_type: Option<String>,
        pub r#where: Option<PyExpr>,
        pub include: Vec<String>,
    }
}
implement_state_pyclass! {
    /// Represents a DROP INDEX SQL statement.
    ///
    /// Builds index deletion statements with support for:
    /// - Conditional deletion (IF EXISTS)
    /// - Table-specific index dropping
    ///
    /// @signature (name: str, table: TableName | str, if_exists: bool = False)
    pub struct [extends=PySchemaStatement] PyDropIndex(DropIndexState) as "DropIndex" {
        pub name: String,
        pub table: PyTableName,
        pub if_exists: bool,
    }
}

impl sea_query::IntoIndexColumn for PyIndexColumn {
    fn into_index_column(self) -> sea_query::IndexColumn {
        match (self.prefix, self.order) {
            (Some(p), Some(o)) => (sea_query::Alias::new(self.name), p, o).into_index_column(),
            (Some(p), None) => (sea_query::Alias::new(self.name), p).into_index_column(),
            (None, Some(o)) => (sea_query::Alias::new(self.name), o).into_index_column(),
            (None, None) => sea_query::Alias::new(self.name).into_index_column(),
        }
    }
}

impl From<String> for PyIndexColumn {
    fn from(value: String) -> Self {
        Self {
            name: value,
            order: None,
            prefix: None,
        }
    }
}

impl TryFrom<&pyo3::Bound<'_, pyo3::PyAny>> for PyIndexColumn {
    type Error = pyo3::PyErr;

    fn try_from(value: &pyo3::Bound<'_, pyo3::PyAny>) -> Result<Self, Self::Error> {
        unsafe {
            if pyo3::ffi::Py_TYPE(value.as_ptr()) == crate::typeref::INDEX_COLUMN_TYPE {
                let casted_value = value.cast_unchecked::<Self>();
                return Ok(casted_value.get().clone());
            }

            if pyo3::ffi::Py_TYPE(value.as_ptr()) == crate::typeref::COLUMN_REF_TYPE {
                let casted_value = value.cast_unchecked::<crate::common::PyColumnRef>();
                let get_value = casted_value.get();

                match &get_value.name {
                    Some(x) => return Ok(Self::from(x.to_string())),
                    None => {
                        return Err(pyo3::exceptions::PyValueError::new_err(
                            "IndexColumn cannot accept asterisk '*' as column",
                        ))
                    }
                }
            }

            if pyo3::ffi::Py_TYPE(value.as_ptr()) == crate::typeref::COLUMN_TYPE {
                let casted_value = value.cast_unchecked::<crate::column::PyColumn>();
                let get_value = casted_value.get();

                return Ok(Self::from(get_value.0.lock().name.clone()));
            }

            if let Ok(x) = value.cast_exact::<pyo3::types::PyString>() {
                return Ok(Self::from(x.to_str()?.to_owned()));
            }

            Err(typeerror!(
                "expected IndexColumn, Column, ColumnRef or str, got {:?}",
                value.py(),
                value.as_ptr()
            ))
        }
    }
}

#[pyo3::pymethods]
impl PyIndexColumn {
    #[new]
    #[
        pyo3(
            signature = (
                name,
                order=None,
                prefix=None,
            )
        )
    ]
    fn __new__(name: String, order: Option<String>, prefix: Option<u32>) -> pyo3::PyResult<Self> {
        Ok(Self {
            name,
            order: match order {
                None => None,
                Some(x) => Some(map_str_to_index_order(x)?),
            },
            prefix,
        })
    }

    /// The name of the column to include in the index.
    ///
    /// @signature (self) -> str
    #[getter]
    fn name(&self) -> String {
        self.name.clone()
    }

    /// Number of characters to index for string columns (prefix indexing).
    ///
    /// @signature (self) -> int | None
    #[getter]
    fn prefix(&self) -> Option<u32> {
        self.prefix
    }

    /// Sort order for this column.
    ///
    /// @signature (self) -> _IndexColumnOrder | None
    #[getter]
    fn order(&self) -> Option<String> {
        self.order.clone().map(map_index_order_to_str)
    }

    fn __copy__(&self) -> Self {
        self.clone()
    }

    fn __repr__(&self) -> String {
        use std::io::Write;

        let mut s = Vec::new();
        write!(&mut s, "<IndexColumn {:?}", self.name).unwrap();

        if let Some(x) = self.prefix {
            write!(&mut s, " prefix={}", x).unwrap();
        }
        if let Some(x) = &self.order {
            if matches!(x, sea_query::IndexOrder::Asc) {
                write!(&mut s, " order='asc'").unwrap();
            } else {
                write!(&mut s, " order='desc'").unwrap();
            }
        }
        write!(&mut s, ">").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}

impl Clone for IndexState {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            columns: self.columns.clone(),
            table: self.table.clone(),
            options: self.options,
            index_type: self.index_type.clone(),
            r#where: self.r#where.clone(),
            include: self.include.clone(),
        }
    }
}

impl ToSeaQuery<sea_query::IndexCreateStatement> for IndexState {
    #[cfg_attr(feature = "optimize", optimize(speed))]
    fn to_sea_query<'a>(&self, _py: pyo3::Python<'a>) -> sea_query::IndexCreateStatement {
        let mut stmt = sea_query::IndexCreateStatement::new();

        if let Some(x) = &self.name {
            stmt.name(x);
        }
        if let Some(x) = &self.table {
            stmt.table(x.clone());
        }

        for col in self.columns.iter() {
            stmt.col(col.clone());
        }

        if let Some(x) = &self.index_type {
            stmt.index_type(sea_query::IndexType::Custom(
                sea_query::Alias::new(x.clone()).into_iden(),
            ));
        }

        for include in self.include.iter() {
            stmt.include(sea_query::Alias::new(include));
        }

        if self.options & PyIndex::OPT_PRIMARY > 0 {
            stmt.primary();
        }
        if self.options & PyIndex::OPT_IF_NOT_EXISTS > 0 {
            stmt.if_not_exists();
        }
        if self.options & PyIndex::OPT_NULLS_NOT_DISTINCT > 0 {
            stmt.nulls_not_distinct();
        }
        if self.options & PyIndex::OPT_UNIQUE > 0 {
            stmt.unique();
        }

        stmt
    }
}

#[pyo3::pymethods]
impl PyIndex {
    #[classattr]
    pub const OPT_PRIMARY: u8 = 1 << 0;
    #[classattr]
    pub const OPT_UNIQUE: u8 = 1 << 1;
    #[classattr]
    pub const OPT_IF_NOT_EXISTS: u8 = 1 << 2;
    #[classattr]
    pub const OPT_NULLS_NOT_DISTINCT: u8 = 1 << 3;

    #[new]
    #[
        pyo3(
            signature = (
                columns,
                table=None,
                name=None,
                options=0,
                *,
                index_type=None,
                r#where=None,
                include=Vec::new()
            )
        )
    ]
    fn __new__(
        columns: Vec<pyo3::Bound<'_, pyo3::PyAny>>,
        table: Option<&pyo3::Bound<'_, pyo3::PyAny>>,
        name: Option<String>,
        options: u8,
        index_type: Option<String>,
        r#where: Option<&pyo3::Bound<'_, pyo3::PyAny>>,
        include: Vec<String>,
    ) -> pyo3::PyResult<(Self, PySchemaStatement)> {
        let table = match table {
            Some(x) => Some(PyTableName::try_from(x)?),
            None => None,
        };

        if columns.is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "columns cannot be empty",
            ));
        }

        let mut cols = Vec::with_capacity(columns.len());
        for c in columns.into_iter() {
            cols.push(PyIndexColumn::try_from(&c)?);
        }

        let r#where = match r#where {
            None => None,
            Some(x) => Some(PyExpr::try_from(x)?),
        };

        let state = IndexState {
            name,
            columns: cols,
            table,
            options,
            index_type,
            r#where,
            include,
        };
        Ok((state.into(), PySchemaStatement))
    }

    /// Index name
    ///
    /// @signature (self) -> str | None
    /// @setter str | None
    #[getter]
    fn name(&self) -> Option<String> {
        let lock = self.0.lock();
        lock.name.clone()
    }

    #[setter]
    fn set_name(&self, val: Option<String>) {
        let mut lock = self.0.lock();
        lock.name = val;
    }

    /// The table on which to create the index.
    ///
    /// @signature (self) -> TableName | None
    /// @setter TableName | str | None
    #[getter]
    fn table(&self) -> Option<PyTableName> {
        let lock = self.0.lock();
        lock.table.clone()
    }

    #[setter]
    fn set_table(&self, val: Option<&pyo3::Bound<'_, pyo3::PyAny>>) -> pyo3::PyResult<()> {
        let val = match val {
            Some(x) => Some(PyTableName::try_from(x)?),
            None => None,
        };

        let mut lock = self.0.lock();
        lock.table = val;
        Ok(())
    }

    /// Index specified options.
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

    /// Whether this is a primary key constraint.
    ///
    /// @signature (self) -> bool
    #[getter]
    fn primary(&self) -> bool {
        self.0.lock().options & Self::OPT_PRIMARY > 0
    }

    /// Whether this is a unique constraint.
    ///
    /// @signature (self) -> bool
    #[getter]
    fn unique(&self) -> bool {
        self.0.lock().options & Self::OPT_UNIQUE > 0
    }

    /// Whether NULL values should be considered equal for uniqueness.
    ///
    /// @signature (self) -> bool
    #[getter]
    fn nulls_not_distinct(&self) -> bool {
        self.0.lock().options & Self::OPT_NULLS_NOT_DISTINCT > 0
    }

    /// Whether to use IF NOT EXISTS clause.
    ///
    /// @signature (self) -> bool
    #[getter]
    fn if_not_exists(&self) -> bool {
        self.0.lock().options & Self::OPT_IF_NOT_EXISTS > 0
    }

    /// The columns that make up this index.
    ///
    /// @signature (self) -> typing.Sequence[IndexColumn]
    /// @setter typing.Iterable[_IndexColumnValue]
    #[getter]
    fn columns(&self) -> Vec<PyIndexColumn> {
        let lock = self.0.lock();
        lock.columns.clone()
    }

    #[setter]
    fn set_columns(&self, val: Vec<pyo3::Bound<'_, pyo3::PyAny>>) -> pyo3::PyResult<()> {
        if val.is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "columns cannot be empty",
            ));
        }

        let mut cols = Vec::with_capacity(val.len());
        for c in val.into_iter() {
            cols.push(PyIndexColumn::try_from(&c)?);
        }

        let mut lock = self.0.lock();
        lock.columns = cols;
        Ok(())
    }

    /// The type/algorithm for this index.
    ///
    /// @signature (self) -> str | None
    /// @setter str | None
    #[getter]
    fn index_type(&self) -> Option<String> {
        let lock = self.0.lock();
        lock.index_type.clone()
    }

    #[setter]
    fn set_index_type(&self, val: Option<String>) -> pyo3::PyResult<()> {
        let mut lock = self.0.lock();
        lock.index_type = val;
        Ok(())
    }

    /// Condition for partial indexing.
    ///
    /// @signature (self) -> Expr | None
    /// @setter object | None
    #[getter]
    fn r#where(&self) -> Option<PyExpr> {
        let lock = self.0.lock();
        lock.r#where.clone()
    }

    #[setter]
    fn set_where(&self, val: Option<&pyo3::Bound<'_, pyo3::PyAny>>) -> pyo3::PyResult<()> {
        let val = match val {
            None => None,
            Some(x) => Some(PyExpr::try_from(x)?),
        };

        let mut lock = self.0.lock();
        lock.r#where = val;
        Ok(())
    }

    /// Additional columns to include in the index for covering queries.
    ///
    /// @signature (self) -> typing.Sequence[str]
    /// @setter typing.Iterable[str]
    #[getter]
    fn include(&self) -> Vec<String> {
        let lock = self.0.lock();
        lock.include.clone()
    }

    #[setter]
    fn set_include(&self, val: Vec<String>) {
        let mut lock = self.0.lock();
        lock.include = val;
    }

    fn __copy__(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::Py<Self>> {
        let lock = self.0.lock();

        pyo3::Py::new(py, (lock.clone().into(), PySchemaStatement))
    }

    fn to_sql(&self, py: pyo3::Python<'_>, backend: String) -> pyo3::PyResult<String> {
        let lock = self.0.lock();
        let stmt = lock.to_sea_query(py);
        drop(lock);

        build_schema_statement!(backend, stmt)
    }

    fn __repr__(&self) -> String {
        use std::io::Write;

        let lock = self.0.lock();
        let mut s = Vec::with_capacity(50);

        write!(s, "<Index").unwrap();

        if let Some(x) = &lock.name {
            write!(s, " {:?}", x).unwrap();
        }
        if let Some(x) = &lock.table {
            write!(s, " {}", x.__repr__()).unwrap();
        }

        write!(s, " columns=[").unwrap();

        let n = lock.columns.len() - 1;
        for (index, col) in lock.columns.iter().enumerate() {
            if index == n {
                write!(s, "{}]", col.__repr__()).unwrap();
            } else {
                write!(s, "{}, ", col.__repr__()).unwrap();
            }
        }

        if lock.options & Self::OPT_IF_NOT_EXISTS > 0 {
            write!(s, " OPT_IF_NOT_EXISTS").unwrap();
        }
        if lock.options & Self::OPT_PRIMARY > 0 {
            write!(s, " OPT_PRIMARY").unwrap();
        }
        if lock.options & Self::OPT_UNIQUE > 0 {
            write!(s, " OPT_UNIQUE").unwrap();
        }
        if lock.options & Self::OPT_NULLS_NOT_DISTINCT > 0 {
            write!(s, " OPT_NULLS_NOT_DISTINCT").unwrap();
        }

        if let Some(x) = &lock.index_type {
            write!(s, " index_type={:?}", x).unwrap();
        }
        if !lock.include.is_empty() {
            write!(s, " include={:?}", lock.include).unwrap();
        }
        if let Some(x) = &lock.r#where {
            write!(s, " where={}", x.__repr__()).unwrap();
        }

        write!(s, ">").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}

impl Clone for DropIndexState {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            table: self.table.clone(),
            if_exists: self.if_exists,
        }
    }
}

impl ToSeaQuery<sea_query::IndexDropStatement> for DropIndexState {
    fn to_sea_query<'a>(&self, _py: pyo3::Python<'a>) -> sea_query::IndexDropStatement {
        let mut stmt = sea_query::IndexDropStatement::new();

        stmt.name(&self.name);
        stmt.table(self.table.clone());

        if self.if_exists {
            stmt.if_exists();
        }
        stmt
    }
}

#[pyo3::pymethods]
impl PyDropIndex {
    #[new]
    #[pyo3(signature = (name, table, if_exists=false))]
    fn __new__(
        name: String,
        table: &pyo3::Bound<'_, pyo3::PyAny>,
        if_exists: bool,
    ) -> pyo3::PyResult<(Self, PySchemaStatement)> {
        let table = PyTableName::try_from(table)?;

        let state = DropIndexState {
            name,
            table,
            if_exists,
        };
        Ok((state.into(), PySchemaStatement))
    }

    /// The name of the index to drop.
    ///
    /// @signature (self) -> str
    /// @setter str
    #[getter]
    fn name(&self) -> String {
        let lock = self.0.lock();
        lock.name.clone()
    }

    #[setter]
    fn set_name(&self, val: String) {
        let mut lock = self.0.lock();
        lock.name = val;
    }

    /// The table from which to drop the index.
    ///
    /// @signature (self) -> TableName
    /// @setter TableName | str
    #[getter]
    fn table(&self) -> PyTableName {
        let lock = self.0.lock();
        lock.table.clone()
    }

    #[setter]
    fn set_table(&self, val: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        let val = PyTableName::try_from(val)?;

        let mut lock = self.0.lock();
        lock.table = val;
        Ok(())
    }

    /// Whether to use IF EXISTS clause to avoid errors.
    ///
    /// @signature (self) -> bool
    /// @setter bool
    #[getter]
    fn if_exists(slf: pyo3::PyRef<'_, Self>) -> bool {
        slf.0.lock().if_exists
    }

    #[setter]
    fn set_if_exists(slf: pyo3::PyRef<'_, Self>, val: bool) {
        let mut lock = slf.0.lock();
        lock.if_exists = val;
    }

    fn __copy__(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::Py<Self>> {
        let lock = self.0.lock();

        pyo3::Py::new(py, (lock.clone().into(), PySchemaStatement))
    }

    fn to_sql(&self, py: pyo3::Python<'_>, backend: String) -> pyo3::PyResult<String> {
        let lock = self.0.lock();
        let stmt = lock.to_sea_query(py);
        drop(lock);

        build_schema_statement!(backend, stmt)
    }

    fn __repr__(&self) -> String {
        use std::io::Write;

        let lock = self.0.lock();
        let mut s = Vec::with_capacity(50);

        write!(
            s,
            "<DropIndex {:?} table={}",
            lock.name,
            lock.table.__repr__()
        )
        .unwrap();

        if lock.if_exists {
            write!(s, " if_exists=True").unwrap();
        }
        write!(s, ">").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}
