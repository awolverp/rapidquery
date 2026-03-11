use sea_query::IntoIden;

use super::base::PySchemaStatement;
use crate::common::expression::PyExpr;
use crate::common::table_ref::PyTableName;
use crate::internal::statements::ToSeaQuery;

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

pub const OPT_PRIMARY: u8 = 1 << 0;
pub const OPT_UNIQUE: u8 = 1 << 1;
pub const OPT_IF_NOT_EXISTS: u8 = 1 << 2;
pub const OPT_NULLS_NOT_DISTINCT: u8 = 1 << 3;

crate::implement_pyclass! {
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
    /// @signature (self, name: str, order: _IndexColumnOrder | None = None, prefix: int | None = None)
    #[derive(Debug, Clone)]
    immutable [subclass] PyIndexColumn(IndexColumnState) as "IndexColumn" {
        pub name: sea_query::DynIden,
        pub order: Option<sea_query::IndexOrder>,
        pub prefix: Option<u32>,
    }
}
crate::implement_pyclass! {
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
    ///     self,
    ///     columns: typing.Iterable[_IndexColumnValue],
    ///     name: str | None = None,
    ///     table: Table | TableName | str | None = None,
    ///     *,
    ///     primary: bool = False,
    ///     if_not_exists: bool = False,
    ///     nulls_not_distinct: bool = False,
    ///     unique: bool = False,
    ///     index_type: str | None = None,
    ///     where: object | None = None,
    ///     include: typing.Iterable[str] = (),
    /// )
    #[derive(Debug)]
    mutable [subclass, extends=PySchemaStatement] PyIndex(IndexState) as "Index" {
        pub name: Option<String>,
        pub columns: Vec<PyIndexColumn>,
        pub table: Option<PyTableName>,
        pub options: u8,
        pub index_type: Option<String>,
        pub r#where: Option<PyExpr>,
        pub include: Vec<String>,
    }
}
crate::implement_pyclass! {
    /// Represents a DROP INDEX SQL statement.
    ///
    /// Builds index deletion statements with support for:
    /// - Conditional deletion (IF EXISTS)
    /// - Table-specific index dropping
    ///
    /// @signature (self, name: str, table: Table | TableName | str, if_exists: bool = False)
    #[derive(Debug)]
    mutable [subclass, extends=PySchemaStatement] PyDropIndex(DropIndexState) as "DropIndex" {
        pub name: String,
        pub table: PyTableName,
        pub if_exists: bool,
    }
}

impl sea_query::IntoIndexColumn for PyIndexColumn {
    fn into_index_column(self) -> sea_query::IndexColumn {
        let inner = self.0.into_inner().unwrap();

        match (inner.prefix, inner.order) {
            (Some(p), Some(o)) => (inner.name, p, o).into_index_column(),
            (Some(p), None) => (inner.name, p).into_index_column(),
            (None, Some(o)) => (inner.name, o).into_index_column(),
            (None, None) => inner.name.into_index_column(),
        }
    }
}

impl TryFrom<&pyo3::Bound<'_, pyo3::PyAny>> for PyIndexColumn {
    type Error = pyo3::PyErr;

    fn try_from(value: &pyo3::Bound<'_, pyo3::PyAny>) -> Result<Self, Self::Error> {
        unsafe {
            if pyo3::ffi::PyObject_TypeCheck(value.as_ptr(), crate::typeref::INDEX_COLUMN_TYPE) == 1
            {
                let casted_value = value.cast_unchecked::<Self>();
                return Ok(casted_value.get().clone());
            }

            let column_ref =
                crate::common::column_ref::PyColumnRef::try_from(value).map_err(|_| {
                    crate::new_py_error!(
                        PyTypeError,
                        "expected IndexColumn, ColumnRef, Column, str, or object.__column_ref__ \
                         property, got {}",
                        crate::internal::get_type_name(value.py(), value.as_ptr())
                    )
                })?;

            match column_ref.name {
                Some(x) => {
                    let inner = IndexColumnState {
                        name: x,
                        order: None,
                        prefix: None,
                    };
                    Ok(inner.into())
                }
                None => Err(pyo3::exceptions::PyValueError::new_err(
                    "IndexColumn cannot accept asterisk '*' as column",
                )),
            }
        }
    }
}

#[pyo3::pymethods]
impl PyIndexColumn {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(
        args: &pyo3::Bound<'_, pyo3::types::PyTuple>,
        kwds: Option<&pyo3::Bound<'_, pyo3::types::PyDict>>,
    ) -> Self {
        Self::uninit()
    }

    #[pyo3(signature=(name, order=None, prefix=None))]
    fn __init__(
        &self,
        name: String,
        order: Option<String>,
        prefix: Option<u32>,
    ) -> pyo3::PyResult<()> {
        let state = IndexColumnState {
            name: sea_query::Alias::new(name).into_iden(),
            order: match order {
                None => None,
                Some(x) => Some(map_str_to_index_order(x)?),
            },
            prefix,
        };
        unsafe {
            self.0.set(state);
        }
        Ok(())
    }

    /// The name of the column to include in the index.
    ///
    /// @signature (self) -> str
    #[getter]
    fn name(&self) -> String {
        self.0.as_ref().name.to_string()
    }

    /// Number of characters to index for string columns (prefix indexing).
    ///
    /// @signature (self) -> int | None
    #[getter]
    fn prefix(&self) -> Option<u32> {
        self.0.as_ref().prefix
    }

    /// Sort order for this column.
    ///
    /// @signature (self) -> _IndexColumnOrder | None
    #[getter]
    fn order(&self) -> Option<String> {
        self.0.as_ref().order.clone().map(map_index_order_to_str)
    }

    fn __copy__(&self) -> Self {
        self.clone()
    }

    fn __repr__(&self) -> String {
        use std::io::Write;

        let inner = self.0.as_ref();
        let mut s = Vec::new();
        write!(&mut s, "<IndexColumn {:?}", inner.name.to_string()).unwrap();

        if let Some(x) = inner.prefix {
            write!(&mut s, " prefix={}", x).unwrap();
        }
        if let Some(x) = &inner.order {
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

        if self.options & OPT_PRIMARY > 0 {
            stmt.primary();
        }
        if self.options & OPT_IF_NOT_EXISTS > 0 {
            stmt.if_not_exists();
        }
        if self.options & OPT_NULLS_NOT_DISTINCT > 0 {
            stmt.nulls_not_distinct();
        }
        if self.options & OPT_UNIQUE > 0 {
            stmt.unique();
        }

        stmt
    }
}

#[pyo3::pymethods]
impl PyIndex {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(
        args: &pyo3::Bound<'_, pyo3::types::PyTuple>,
        kwds: Option<&pyo3::Bound<'_, pyo3::types::PyDict>>,
    ) -> (Self, PySchemaStatement) {
        (Self::uninit(), PySchemaStatement)
    }

    #[
        pyo3(
            signature = (
                columns,
                name=None,
                table=None,
                *,
                primary=false,
                if_not_exists=false,
                nulls_not_distinct=false,
                unique=false,
                index_type=None,
                r#where=None,
                include=Vec::new()
            )
        )
    ]
    fn __init__(
        &self,
        columns: Vec<pyo3::Bound<'_, pyo3::PyAny>>,
        name: Option<String>,
        table: Option<&pyo3::Bound<'_, pyo3::PyAny>>,
        primary: bool,
        if_not_exists: bool,
        nulls_not_distinct: bool,
        unique: bool,
        index_type: Option<String>,
        r#where: Option<&pyo3::Bound<'_, pyo3::PyAny>>,
        include: Vec<String>,
    ) -> pyo3::PyResult<()> {
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

        let mut options = 0u8;
        if primary {
            options |= OPT_PRIMARY;
        }
        if unique {
            options |= OPT_UNIQUE;
        }
        if nulls_not_distinct {
            options |= OPT_NULLS_NOT_DISTINCT;
        }
        if if_not_exists {
            options |= OPT_IF_NOT_EXISTS;
        }

        let state = IndexState {
            name,
            columns: cols,
            table,
            options,
            index_type,
            r#where,
            include,
        };
        self.0.set(state);

        Ok(())
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
    /// @signature (self) -> Table | TableName | None
    /// @setter Table | TableName | str | None
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

    /// Whether this is a primary key constraint.
    ///
    /// @signature (self) -> bool
    /// @setter bool
    #[getter]
    fn primary(&self) -> bool {
        self.0.lock().options & OPT_PRIMARY > 0
    }

    #[setter]
    fn set_primary(&self, value: bool) {
        let mut lock = self.0.lock();
        if value {
            lock.options |= OPT_PRIMARY;
        } else {
            lock.options &= !OPT_PRIMARY;
        }
    }

    /// Whether this is a unique constraint.
    ///
    /// @signature (self) -> bool
    /// @setter bool
    #[getter]
    fn unique(&self) -> bool {
        self.0.lock().options & OPT_UNIQUE > 0
    }

    #[setter]
    fn set_unique(&self, value: bool) {
        let mut lock = self.0.lock();
        if value {
            lock.options |= OPT_UNIQUE;
        } else {
            lock.options &= !OPT_UNIQUE;
        }
    }

    /// Whether NULL values should be considered equal for uniqueness.
    ///
    /// @signature (self) -> bool
    /// @setter bool
    #[getter]
    fn nulls_not_distinct(&self) -> bool {
        self.0.lock().options & OPT_NULLS_NOT_DISTINCT > 0
    }

    #[setter]
    fn set_nulls_not_distinct(&self, value: bool) {
        let mut lock = self.0.lock();
        if value {
            lock.options |= OPT_NULLS_NOT_DISTINCT;
        } else {
            lock.options &= !OPT_NULLS_NOT_DISTINCT;
        }
    }

    /// Whether to use IF NOT EXISTS clause.
    ///
    /// @signature (self) -> bool
    /// @setter bool
    #[getter]
    fn if_not_exists(&self) -> bool {
        self.0.lock().options & OPT_IF_NOT_EXISTS > 0
    }

    #[setter]
    fn set_if_not_exists(&self, value: bool) {
        let mut lock = self.0.lock();
        if value {
            lock.options |= OPT_IF_NOT_EXISTS;
        } else {
            lock.options &= !OPT_IF_NOT_EXISTS;
        }
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

    #[pyo3(signature = (backend, /))]
    #[allow(clippy::wrong_self_convention)]
    fn to_sql(&self, py: pyo3::Python<'_>, backend: String) -> pyo3::PyResult<String> {
        let lock = self.0.lock();
        let stmt = lock.to_sea_query(py);
        drop(lock);

        crate::build_schema_statement!(backend, stmt)
    }

    pub fn __repr__(&self) -> String {
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

        if lock.options & OPT_IF_NOT_EXISTS > 0 {
            write!(s, " if_not_exists=True").unwrap();
        }
        if lock.options & OPT_PRIMARY > 0 {
            write!(s, " primary=True").unwrap();
        }
        if lock.options & OPT_UNIQUE > 0 {
            write!(s, " unique=True").unwrap();
        }
        if lock.options & OPT_NULLS_NOT_DISTINCT > 0 {
            write!(s, " nulls_not_distinct=True").unwrap();
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
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(
        args: &pyo3::Bound<'_, pyo3::types::PyTuple>,
        kwds: Option<&pyo3::Bound<'_, pyo3::types::PyDict>>,
    ) -> (Self, PySchemaStatement) {
        (Self::uninit(), PySchemaStatement)
    }

    #[pyo3(signature = (name, table, if_exists=false))]
    fn __init__(
        &self,
        name: String,
        table: &pyo3::Bound<'_, pyo3::PyAny>,
        if_exists: bool,
    ) -> pyo3::PyResult<()> {
        let table = PyTableName::try_from(table)?;

        let state = DropIndexState {
            name,
            table,
            if_exists,
        };
        self.0.set(state);
        Ok(())
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
    /// @setter Table | TableName | str
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
