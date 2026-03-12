use super::base::PySchemaStatement;
use crate::common::table_ref::PyTableName;
use crate::internal::{BoundArgs, BoundKwargs, BoundObject, PyObject, RefBoundObject, ToSeaQuery};

use pyo3::types::PyTupleMethods;

pub const OPT_IF_NOT_EXISTS: u8 = 1 << 0;
pub const OPT_TEMPORARY: u8 = 1 << 1;

crate::implement_pyclass! {
    /// Represents a complete database table definition.
    ///
    /// This class encapsulates all aspects of a table structure including:
    /// - Column definitions with their types and constraints
    /// - Indexes for query optimization
    /// - Foreign key relationships for referential integrity
    /// - Check constraints for data validation
    /// - Table-level options like engine, collation, and character set
    ///
    /// Used to generate CREATE TABLE SQL statements with full schema specifications.
    ///
    /// @signature (
    ///     self,
    ///     name: TableName | str,
    ///     *args: Column | Index | ForeignKey | Expr,
    ///     options: int = 0,
    ///     comment: str | None = None,
    ///     engine: str | None = None,
    ///     collate: str | None = None,
    ///     character_set: str | None = None,
    ///     extra: str | None = None,
    /// )
    mutable [subclass, extends=PySchemaStatement] PyTable(TableState) as "Table" {
        pub name: PyTableName,

        /// Always is `Vec<PyColumn>`
        pub columns: Vec<PyObject>,

        /// Always is `Vec<PyIndex>`
        pub indexes: Vec<PyObject>,

        /// Always is `Vec<PyForeignKey>`
        pub foreign_keys: Vec<PyObject>,

        /// Always is `Vec<PyExpr>`
        pub checks: Vec<PyObject>,

        pub options: u8,
        pub comment: Option<String>,
        pub engine: Option<String>,
        pub collate: Option<String>,
        pub character_set: Option<String>,
        pub extra: Option<String>,
    }
}

impl ToSeaQuery<sea_query::TableCreateStatement> for TableState {
    #[cfg_attr(feature = "optimize", optimize(speed))]
    fn to_sea_query<'a>(&self, py: pyo3::Python<'a>) -> sea_query::TableCreateStatement {
        let mut stmt = sea_query::TableCreateStatement::new();
        stmt.table(self.name.clone());

        // Add columns
        for column in self.columns.iter() {
            let col = unsafe { column.cast_bound_unchecked::<crate::common::column::PyColumn>(py) };

            let column_def: sea_query::ColumnDef = col.get().0.lock().to_sea_query(py);
            stmt.col(column_def);
        }

        // Add primary and unique indexes
        for index in self.indexes.iter() {
            let ix = unsafe { index.cast_bound_unchecked::<super::index::PyIndex>(py) };

            let lock = ix.get().0.lock();
            if lock.options & super::index::OPT_PRIMARY > 0
                || lock.options & super::index::OPT_UNIQUE > 0
            {
                let mut index_stmt = lock.to_sea_query(py);
                stmt.index(&mut index_stmt);
            }
        }

        // Add foreign keys
        for foreign_key in self.foreign_keys.iter() {
            let fk = unsafe {
                foreign_key.cast_bound_unchecked::<crate::common::foreign_key::PyForeignKey>(py)
            };

            let mut fk_stmt = fk.get().0.lock().to_sea_query(py);
            stmt.foreign_key(&mut fk_stmt);
        }

        // Add check constraints
        for check in self.checks.iter() {
            let simple_expr =
                unsafe { check.cast_bound_unchecked::<crate::common::expression::PyExpr>(py) };

            stmt.check(simple_expr.get().0.clone());
        }

        if self.options & OPT_IF_NOT_EXISTS > 0 {
            stmt.if_not_exists();
        }
        if self.options & OPT_TEMPORARY > 0 {
            stmt.temporary();
        }

        if let Some(x) = &self.comment {
            stmt.comment(x);
        }
        if let Some(x) = &self.engine {
            stmt.engine(x);
        }
        if let Some(x) = &self.collate {
            stmt.collate(x);
        }
        if let Some(x) = &self.character_set {
            stmt.character_set(x);
        }
        if let Some(x) = &self.extra {
            stmt.extra(x);
        }

        stmt
    }
}

#[pyo3::pymethods]
impl PyTable {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(args: BoundArgs<'_>, kwds: Option<BoundKwargs<'_>>) -> (Self, PySchemaStatement) {
        (Self::uninit(), PySchemaStatement)
    }

    #[
            pyo3(
                signature=(
                    name,
                    *args,
                    if_not_exists=false,
                    temporary=false,
                    comment=None,
                    engine=None,
                    collate=None,
                    character_set=None,
                    extra=None,
                )
            )
        ]
    fn __init__(
        &self,
        name: RefBoundObject<'_>,
        args: BoundArgs<'_>,
        if_not_exists: bool,
        temporary: bool,
        comment: Option<String>,
        engine: Option<String>,
        collate: Option<String>,
        character_set: Option<String>,
        extra: Option<String>,
    ) -> pyo3::PyResult<()> {
        let name = PyTableName::try_from(name)?;

        let mut columns = Vec::new();
        let mut indexes = Vec::new();
        let mut foreign_keys = Vec::new();
        let mut checks = Vec::new();

        for object in args.iter() {
            unsafe {
                // columns
                if pyo3::ffi::PyObject_TypeCheck(object.as_ptr(), crate::typeref::COLUMN_TYPE) == 1
                {
                    columns.push(object.unbind());
                }
                // indexes
                else if pyo3::ffi::PyObject_TypeCheck(object.as_ptr(), crate::typeref::INDEX_TYPE)
                    == 1
                {
                    indexes.push(object.unbind());
                }
                // foreign keys
                else if pyo3::ffi::PyObject_TypeCheck(
                    object.as_ptr(),
                    crate::typeref::FOREIGN_KEY_TYPE,
                ) == 1
                {
                    foreign_keys.push(object.unbind());
                }
                // expressions (check constraints)
                else if pyo3::ffi::Py_TYPE(object.as_ptr()) == crate::typeref::EXPR_TYPE {
                    checks.push(object.unbind());
                } else {
                    return crate::new_error!(
                        PyTypeError,
                        "expected Column, Index, ForeignKey, or Expr, got {}",
                        crate::internal::get_type_name(object.py(), object.as_ptr())
                    );
                }
            }
        }

        let mut options = 0u8;
        if if_not_exists {
            options |= OPT_IF_NOT_EXISTS;
        }
        if temporary {
            options |= OPT_TEMPORARY;
        }

        let state = TableState {
            name,
            columns,
            indexes,
            foreign_keys,
            checks,
            options,
            comment,
            engine,
            collate,
            character_set,
            extra,
        };
        self.0.set(state);
        Ok(())
    }

    /// The name of this table.
    ///
    /// @signature (self) -> TableName
    #[getter]
    fn name(&self) -> PyTableName {
        let lock = self.0.lock();
        lock.name.clone()
    }

    /// Table columns.
    ///
    /// @signature (self) -> typing.Sequence[Column]
    /// @setter typing.Iterable[Column]
    #[getter]
    fn columns(&self, py: pyo3::Python) -> Vec<PyObject> {
        let lock = self.0.lock();
        lock.columns.iter().map(|x| x.clone_ref(py)).collect()
    }

    #[setter]
    fn set_columns(&self, val: Vec<BoundObject<'_>>) -> pyo3::PyResult<()> {
        let mut columns = Vec::new();

        for object in val.into_iter() {
            unsafe {
                if pyo3::ffi::PyObject_TypeCheck(object.as_ptr(), crate::typeref::COLUMN_TYPE) == 0
                {
                    return crate::new_error!(
                        PyTypeError,
                        "expected iterable of Column, found {}",
                        crate::internal::get_type_name(object.py(), object.as_ptr())
                    );
                }
            }

            columns.push(object.unbind());
        }

        self.0.lock().columns = columns;
        Ok(())
    }

    /// Table indexes.
    ///
    /// @signature (self) -> typing.Sequence[Index]
    /// @setter typing.Iterable[Index]
    #[getter]
    fn indexes(&self, py: pyo3::Python) -> Vec<PyObject> {
        let lock = self.0.lock();
        lock.indexes.iter().map(|x| x.clone_ref(py)).collect()
    }

    #[setter]
    fn set_indexes(&self, val: Vec<BoundObject<'_>>) -> pyo3::PyResult<()> {
        let mut indexes = Vec::new();

        for object in val.into_iter() {
            unsafe {
                if pyo3::ffi::PyObject_TypeCheck(object.as_ptr(), crate::typeref::INDEX_TYPE) == 0 {
                    return crate::new_error!(
                        PyTypeError,
                        "expected iterable of Index, found {}",
                        crate::internal::get_type_name(object.py(), object.as_ptr())
                    );
                }
            }

            indexes.push(object.unbind());
        }

        self.0.lock().indexes = indexes;
        Ok(())
    }

    /// Table foreign keys.
    ///
    /// @signature (self) -> typing.Sequence[ForeignKey]
    /// @setter typing.Iterable[ForeignKey]
    #[getter]
    fn foreign_keys(&self, py: pyo3::Python) -> Vec<PyObject> {
        let lock = self.0.lock();
        lock.foreign_keys.iter().map(|x| x.clone_ref(py)).collect()
    }

    #[setter]
    fn set_foreign_keys(&self, val: Vec<BoundObject<'_>>) -> pyo3::PyResult<()> {
        let mut foreign_keys = Vec::new();

        for object in val.into_iter() {
            unsafe {
                if pyo3::ffi::PyObject_TypeCheck(object.as_ptr(), crate::typeref::FOREIGN_KEY_TYPE)
                    == 0
                {
                    return crate::new_error!(
                        PyTypeError,
                        "expected iterable of ForeignKey, found {}",
                        crate::internal::get_type_name(object.py(), object.as_ptr())
                    );
                }
            }

            foreign_keys.push(object.unbind());
        }

        self.0.lock().foreign_keys = foreign_keys;
        Ok(())
    }

    /// Table check constraints.
    ///
    /// @signature (self) -> typing.Sequence[Expr]
    /// @setter typing.Iterable[Expr]
    #[getter]
    fn checks(&self, py: pyo3::Python) -> Vec<PyObject> {
        let lock = self.0.lock();
        lock.checks.iter().map(|x| x.clone_ref(py)).collect()
    }

    #[setter]
    fn set_checks(&self, val: Vec<BoundObject<'_>>) -> pyo3::PyResult<()> {
        let mut checks = Vec::new();

        for object in val.into_iter() {
            unsafe {
                if pyo3::ffi::Py_TYPE(object.as_ptr()) != crate::typeref::EXPR_TYPE {
                    return crate::new_error!(
                        PyTypeError,
                        "expected iterable of Expr, found {}",
                        crate::internal::get_type_name(object.py(), object.as_ptr())
                    );
                }
            }

            checks.push(object.unbind());
        }

        self.0.lock().checks = checks;
        Ok(())
    }

    /// Whether to use IF NOT EXISTS clause to avoid errors if table exists.
    ///
    /// @signature (self) -> bool
    #[getter]
    fn if_not_exists(&self) -> bool {
        self.0.lock().options & OPT_IF_NOT_EXISTS > 0
    }

    /// Whether this is a temporary table that exists only for the session.
    ///
    /// @signature (self) -> bool
    #[getter]
    fn temporary(&self) -> bool {
        self.0.lock().options & OPT_TEMPORARY > 0
    }

    /// Comment describing the purpose of this table.
    ///
    /// @signature (self) -> str | None
    /// @setter str | None
    #[getter]
    fn comment(&self) -> Option<String> {
        self.0.lock().comment.clone()
    }

    #[setter]
    fn set_comment(&self, val: Option<String>) {
        self.0.lock().comment = val;
    }

    /// Storage engine for the table (e.g., InnoDB, MyISAM for MySQL).
    ///
    /// @signature (self) -> str | None
    /// @setter str | None
    #[getter]
    fn engine(&self) -> Option<String> {
        self.0.lock().engine.clone()
    }

    #[setter]
    fn set_engine(&self, val: Option<String>) {
        self.0.lock().engine = val;
    }

    /// Collation for string comparisons and sorting in this table.
    ///
    /// @signature (self) -> str | None
    /// @setter str | None
    #[getter]
    fn collate(&self) -> Option<String> {
        self.0.lock().collate.clone()
    }

    #[setter]
    fn set_collate(&self, val: Option<String>) {
        self.0.lock().collate = val;
    }

    /// Character set encoding for text data in this table.
    ///
    /// @signature (self) -> str | None
    /// @setter str | None
    #[getter]
    fn character_set(&self) -> Option<String> {
        self.0.lock().character_set.clone()
    }

    #[setter]
    fn set_character_set(&self, val: Option<String>) {
        self.0.lock().character_set = val;
    }

    /// Additional table-specific options for the CREATE TABLE statement.
    ///
    /// @signature (self) -> str | None
    /// @setter str | None
    #[getter]
    fn extra(&self) -> Option<String> {
        self.0.lock().extra.clone()
    }

    #[setter]
    fn set_extra(&self, val: Option<String>) {
        self.0.lock().extra = val;
    }

    #[pyo3(signature = (backend, /))]
    #[allow(clippy::wrong_self_convention)]
    fn to_sql(&self, py: pyo3::Python<'_>, backend: String) -> pyo3::PyResult<String> {
        let lock = self.0.lock();
        let stmt = lock.to_sea_query(py);

        let mut sqls = vec![crate::build_schema_statement!(&backend, stmt)?];

        for index in lock.indexes.iter() {
            let ix = unsafe { index.cast_bound_unchecked::<super::index::PyIndex>(py) };
            let ix_lock = ix.get().0.lock();

            if ix_lock.options & super::index::OPT_PRIMARY > 0
                || ix_lock.options & super::index::OPT_UNIQUE > 0
            {
                continue;
            }

            // Index name and table is necessary here
            if ix_lock.name.is_none() {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "You should always name indexes that aren't primary or unique",
                ));
            }

            let mut index_stmt = ix_lock.to_sea_query(py);
            index_stmt.table(lock.name.clone());

            sqls.push(crate::build_schema_statement!(&backend, index_stmt)?);
        }

        Ok(sqls.join(";\n"))
    }

    fn __repr__(&self) -> String {
        use std::io::Write;

        let lock = self.0.lock();
        let mut s = Vec::with_capacity(50);

        write!(s, "<Table name={} columns=[", lock.name.__repr__()).unwrap();

        let n = lock.columns.len();
        for (index, col) in lock.columns.iter().enumerate() {
            if index + 1 == n {
                write!(s, "{}", col).unwrap();
            } else {
                write!(s, "{}, ", col).unwrap();
            }
        }

        write!(s, "] indexes=[").unwrap();

        let n = lock.indexes.len();
        for (index, ix) in lock.indexes.iter().enumerate() {
            if index + 1 == n {
                write!(s, "{}", ix).unwrap();
            } else {
                write!(s, "{}, ", ix).unwrap();
            }
        }

        write!(s, "] foreign_keys=[").unwrap();

        let n = lock.foreign_keys.len();
        for (index, fk) in lock.foreign_keys.iter().enumerate() {
            if index + 1 == n {
                write!(s, "{}", fk).unwrap();
            } else {
                write!(s, "{}, ", fk).unwrap();
            }
        }

        write!(s, "] checks=[").unwrap();

        let n = lock.checks.len();
        for (index, expr) in lock.checks.iter().enumerate() {
            if index + 1 == n {
                write!(s, "{}", expr).unwrap();
            } else {
                write!(s, "{}, ", expr).unwrap();
            }
        }
        write!(s, "]").unwrap();

        if lock.options & OPT_IF_NOT_EXISTS > 0 {
            write!(s, " if_not_exists=True").unwrap();
        }
        if lock.options & OPT_TEMPORARY > 0 {
            write!(s, " temporary=True").unwrap();
        }

        if let Some(x) = &lock.comment {
            write!(s, " comment={x:?}").unwrap();
        }
        if let Some(x) = &lock.engine {
            write!(s, " engine={x:?}").unwrap();
        }
        if let Some(x) = &lock.collate {
            write!(s, " collate={x:?}").unwrap();
        }
        if let Some(x) = &lock.character_set {
            write!(s, " character_set={x:?}").unwrap();
        }

        write!(s, ">").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}
