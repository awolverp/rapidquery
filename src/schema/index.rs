use sea_query::IntoIden;

use super::base::PySchemaStatement;
use crate::common::expression::PyExpr;
use crate::common::table_ref::PyTableName;
use crate::internal::repr::ReprFormatter;
use crate::internal::{BoundArgs, BoundKwargs, BoundObject, RefBoundObject, ToSeaQuery};

#[inline]
fn map_str_to_index_order(value: String) -> pyo3::PyResult<sea_query::IndexOrder> {
    match value.to_ascii_uppercase().as_str() {
        "ASC" => Ok(sea_query::IndexOrder::Asc),
        "DESC" => Ok(sea_query::IndexOrder::Desc),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "unknown order: {}",
            value
        ))),
    }
}

#[inline]
fn map_index_order_to_str(value: &sea_query::IndexOrder) -> String {
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
    #[derive(Debug, Clone)]
    [] PyIndexColumn as "IndexColumn" {
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
    #[derive(Debug)]
    mutable [subclass, extends=PySchemaStatement] PyDropIndex(DropIndexState) as "DropIndex" {
        pub name: String,
        pub table: PyTableName,
        pub if_exists: bool,
    }
}

impl sea_query::IntoIndexColumn for PyIndexColumn {
    fn into_index_column(self) -> sea_query::IndexColumn {
        match (self.prefix, self.order) {
            (Some(p), Some(o)) => (self.name, p, o).into_index_column(),
            (Some(p), None) => (self.name, p).into_index_column(),
            (None, Some(o)) => (self.name, o).into_index_column(),
            (None, None) => self.name.into_index_column(),
        }
    }
}

impl TryFrom<RefBoundObject<'_>> for PyIndexColumn {
    type Error = pyo3::PyErr;

    fn try_from(value: RefBoundObject<'_>) -> Result<Self, Self::Error> {
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
                        "expected IndexColumn, ColumnRef, Column, str, or object.to_column_ref \
                         property, got {}",
                        crate::internal::get_type_name(value.py(), value.as_ptr())
                    )
                })?;

            match column_ref.name {
                Some(x) => {
                    let inner = Self {
                        name: x,
                        order: None,
                        prefix: None,
                    };
                    Ok(inner)
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
    #[pyo3(signature=(name, order=None, prefix=None))]
    fn __new__(name: String, order: Option<String>, prefix: Option<u32>) -> pyo3::PyResult<Self> {
        let state = Self {
            name: sea_query::Alias::new(name).into_iden(),
            order: match order {
                None => None,
                Some(x) => Some(map_str_to_index_order(x)?),
            },
            prefix,
        };
        Ok(state)
    }

    /// The name of the column to include in the index.
    #[getter]
    fn name(&self) -> String {
        self.name.to_string()
    }

    /// Number of characters to index for string columns (prefix indexing).
    #[getter]
    fn prefix(&self) -> Option<u32> {
        self.prefix
    }

    /// Sort order for this column.
    #[getter]
    fn order(&self) -> Option<String> {
        self.order.as_ref().map(map_index_order_to_str)
    }

    fn __copy__(&self) -> Self {
        self.clone()
    }

    fn __repr__(&self) -> String {
        ReprFormatter::new("IndexColumn")
            .iden("name", &self.name)
            .optional_display("prefix", self.prefix)
            .optional_map("order", self.order.as_ref(), map_index_order_to_str)
            .finish()
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
    fn __new__(args: BoundArgs<'_>, kwds: Option<BoundKwargs<'_>>) -> (Self, PySchemaStatement) {
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
        columns: Vec<BoundObject<'_>>,
        name: Option<String>,
        table: Option<RefBoundObject<'_>>,
        primary: bool,
        if_not_exists: bool,
        nulls_not_distinct: bool,
        unique: bool,
        index_type: Option<String>,
        r#where: Option<RefBoundObject<'_>>,
        include: Vec<String>,
    ) -> pyo3::PyResult<()> {
        let table = match table {
            Some(x) => Some(PyTableName::try_from(x)?),
            None => None,
        };

        let r#where = match r#where {
            None => None,
            Some(x) => unsafe {
                if pyo3::ffi::Py_TYPE(x.as_ptr()) != crate::typeref::EXPR_TYPE {
                    return crate::new_error!(
                        PyTypeError,
                        "expected Expr or None, got {}",
                        crate::internal::get_type_name(x.py(), x.as_ptr())
                    );
                }

                Some(x.cast_unchecked::<PyExpr>().get().clone())
            },
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

    /// Index name.
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
    #[getter]
    fn table(&self) -> Option<PyTableName> {
        let lock = self.0.lock();
        lock.table.clone()
    }

    #[setter]
    fn set_table(&self, val: Option<RefBoundObject<'_>>) -> pyo3::PyResult<()> {
        let val = match val {
            Some(x) => Some(PyTableName::try_from(x)?),
            None => None,
        };

        let mut lock = self.0.lock();
        lock.table = val;
        Ok(())
    }

    /// Whether this is a primary key constraint.
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
    #[getter]
    fn columns(&self) -> Vec<PyIndexColumn> {
        let lock = self.0.lock();
        lock.columns.clone()
    }

    #[setter]
    fn set_columns(&self, val: Vec<BoundObject<'_>>) -> pyo3::PyResult<()> {
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
    #[getter]
    fn r#where(&self) -> Option<PyExpr> {
        let lock = self.0.lock();
        lock.r#where.clone()
    }

    #[setter]
    fn set_where(&self, val: Option<RefBoundObject<'_>>) -> pyo3::PyResult<()> {
        let val = match val {
            None => None,
            Some(x) => Some(PyExpr::try_from(x)?),
        };

        let mut lock = self.0.lock();
        lock.r#where = val;
        Ok(())
    }

    /// Additional columns to include in the index for covering queries
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

    pub fn __repr__(slf: pyo3::PyRef<'_, Self>) -> String {
        let lock = slf.0.lock();

        let mut fmt = ReprFormatter::new_with_pyref(&slf)
            .optional_quote("name", lock.name.as_ref())
            .optional_map("table", lock.table.as_ref(), |x| x.__repr__())
            .take();

        fmt.vec("columns", true)
            .display_iter(lock.columns.iter().map(|x| x.__repr__()))
            .finish(&mut fmt);

        fmt.optional_boolean("if_not_exists", lock.options & OPT_IF_NOT_EXISTS > 0)
            .optional_boolean("primary", lock.options & OPT_PRIMARY > 0)
            .optional_boolean("unique", lock.options & OPT_UNIQUE > 0)
            .optional_boolean(
                "nulls_not_distinct",
                lock.options & OPT_NULLS_NOT_DISTINCT > 0,
            )
            .optional_quote("index_type", lock.index_type.as_ref())
            .optional_map("where", lock.r#where.as_ref(), |x| x.__repr__());

        fmt.vec("include", true)
            .display_iter(lock.include.iter())
            .finish(&mut fmt);

        fmt.finish()
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
    fn __new__(args: BoundArgs<'_>, kwds: Option<BoundKwargs<'_>>) -> (Self, PySchemaStatement) {
        (Self::uninit(), PySchemaStatement)
    }

    #[pyo3(signature = (name, table, if_exists=false))]
    fn __init__(
        &self,
        name: String,
        table: RefBoundObject<'_>,
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
    #[getter]
    fn table(&self) -> PyTableName {
        let lock = self.0.lock();
        lock.table.clone()
    }

    #[setter]
    fn set_table(&self, val: RefBoundObject<'_>) -> pyo3::PyResult<()> {
        let val = PyTableName::try_from(val)?;

        let mut lock = self.0.lock();
        lock.table = val;
        Ok(())
    }

    /// Whether to use IF EXISTS clause to avoid errors.
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

    fn __repr__(slf: pyo3::PyRef<'_, Self>) -> String {
        let lock = slf.0.lock();

        ReprFormatter::new_with_pyref(&slf)
            .quote("name", &lock.name)
            .map("table", &lock.table, |x| x.__repr__())
            .optional_boolean("if_exists", lock.if_exists)
            .finish()
    }
}
