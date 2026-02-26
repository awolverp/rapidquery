pub mod alter;
pub mod operations;

use pyo3::types::PyTupleMethods;

use crate::column::PyColumn;
use crate::common::PySchemaStatement;
use crate::common::PyTableName;
use crate::expression::PyExpr;
use crate::foreign_key::PyForeignKey;
use crate::index::PyIndex;
use crate::utils::ToSeaQuery;

implement_state_pyclass! {
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
    ///     name: TableName | str,
    ///     *args: Column | Index | ForeignKey | Expr,
    ///     options: int = 0,
    ///     comment: str | None = None,
    ///     engine: str | None = None,
    ///     collate: str | None = None,
    ///     character_set: str | None = None,
    ///     extra: str | None = None,
    /// )
    pub struct [extends=PySchemaStatement] PyTable(TableState) as "Table" {
        pub name: PyTableName,
        pub columns: Vec<PyColumn>,
        pub indexes: Vec<PyIndex>,
        pub foreign_keys: Vec<PyForeignKey>,
        pub checks: Vec<PyExpr>,
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

        for col in self.columns.iter() {
            let column_def: sea_query::ColumnDef = col.0.lock().to_sea_query(py);
            stmt.col(column_def);
        }
        for ix in self.indexes.iter() {
            let lock = ix.0.lock();
            if lock.options & PyIndex::OPT_PRIMARY > 0 || lock.options & PyIndex::OPT_UNIQUE > 0 {
                let mut index_stmt = lock.to_sea_query(py);
                stmt.index(&mut index_stmt);
            }
        }
        for fk in self.foreign_keys.iter() {
            let mut fk_stmt = fk.0.lock().to_sea_query(py);
            stmt.foreign_key(&mut fk_stmt);
        }
        for check in self.checks.iter() {
            stmt.check(check.0.clone());
        }

        if self.options & PyTable::OPT_IF_NOT_EXISTS > 0 {
            stmt.if_not_exists();
        }
        if self.options & PyTable::OPT_TEMPORARY > 0 {
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
    #[classattr]
    pub const OPT_IF_NOT_EXISTS: u8 = 1 << 0;
    #[classattr]
    pub const OPT_TEMPORARY: u8 = 1 << 1;

    #[new]
    #[
        pyo3(
            signature=(
                name,
                *args,
                options=0,
                comment=None,
                engine=None,
                collate=None,
                character_set=None,
                extra=None,
            )
        )
    ]
    fn __new__(
        name: &pyo3::Bound<'_, pyo3::PyAny>,
        args: pyo3::Bound<'_, pyo3::types::PyTuple>,
        options: u8,
        comment: Option<String>,
        engine: Option<String>,
        collate: Option<String>,
        character_set: Option<String>,
        extra: Option<String>,
    ) -> pyo3::PyResult<(Self, PySchemaStatement)> {
        let name = PyTableName::try_from(name)?;

        let mut columns: Vec<PyColumn> = Vec::new();
        let mut indexes: Vec<PyIndex> = Vec::new();
        let mut foreign_keys: Vec<PyForeignKey> = Vec::new();
        let mut checks: Vec<PyExpr> = Vec::new();

        for object in args.iter() {
            unsafe {
                let type_ptr = pyo3::ffi::Py_TYPE(object.as_ptr());

                if type_ptr == crate::typeref::COLUMN_TYPE {
                    let item = object.cast_into_unchecked::<PyColumn>();
                    columns.push(PyColumn(std::sync::Arc::clone(&item.get().0)));
                } else if type_ptr == crate::typeref::INDEX_TYPE {
                    let item = object.cast_into_unchecked::<PyIndex>();
                    indexes.push(PyIndex(std::sync::Arc::clone(&item.get().0)));
                } else if type_ptr == crate::typeref::FOREIGN_KEY_TYPE {
                    let item = object.cast_into_unchecked::<PyForeignKey>();
                    foreign_keys.push(PyForeignKey(std::sync::Arc::clone(&item.get().0)));
                } else if type_ptr == crate::typeref::EXPR_TYPE {
                    let item = object.cast_into_unchecked::<PyExpr>();
                    checks.push(item.get().clone());
                } else {
                    return Err(typeerror!(
                        "expected Column, Index, ForeignKey, or Expr, got {:?}",
                        object.py(),
                        object.as_ptr()
                    ));
                }
            }
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
        Ok((state.into(), PySchemaStatement))
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
    fn columns(&self) -> Vec<PyColumn> {
        let lock = self.0.lock();
        lock.columns
            .iter()
            .map(|x| PyColumn(std::sync::Arc::clone(&x.0)))
            .collect()
    }

    #[setter]
    fn set_columns(&self, val: Vec<pyo3::Bound<'_, pyo3::PyAny>>) -> pyo3::PyResult<()> {
        let mut columns: Vec<PyColumn> = Vec::new();

        for object in val.iter() {
            unsafe {
                let type_ptr = pyo3::ffi::Py_TYPE(object.as_ptr());

                if type_ptr == crate::typeref::COLUMN_TYPE {
                    let item = object.cast_unchecked::<PyColumn>();
                    columns.push(PyColumn(std::sync::Arc::clone(&item.get().0)));
                } else {
                    return Err(typeerror!(
                        "expected typing.Iterable[Column], got typing.Iterable[{:?}]",
                        object.py(),
                        object.as_ptr()
                    ));
                }
            }
        }

        self.0.lock().columns = columns;
        Ok(())
    }

    /// Table indexes.
    ///
    /// @signature (self) -> typing.Sequence[Index]
    /// @setter typing.Iterable[Index]
    #[getter]
    fn indexes(&self, py: pyo3::Python) -> Vec<pyo3::Py<PyIndex>> {
        let lock = self.0.lock();
        lock.indexes
            .iter()
            .map(|x| PyIndex(std::sync::Arc::clone(&x.0)))
            .map(|x| pyo3::Py::new(py, (x, PySchemaStatement)).unwrap())
            .collect()
    }

    #[setter]
    fn set_indexes(&self, val: Vec<pyo3::Bound<'_, pyo3::PyAny>>) -> pyo3::PyResult<()> {
        let mut indexes: Vec<PyIndex> = Vec::new();

        for object in val.iter() {
            unsafe {
                let type_ptr = pyo3::ffi::Py_TYPE(object.as_ptr());

                if type_ptr == crate::typeref::INDEX_TYPE {
                    let item = object.cast_unchecked::<PyIndex>();
                    indexes.push(PyIndex(std::sync::Arc::clone(&item.get().0)));
                } else {
                    return Err(typeerror!(
                        "expected typing.Iterable[Index], got typing.Iterable[{:?}]",
                        object.py(),
                        object.as_ptr()
                    ));
                }
            }
        }

        self.0.lock().indexes = indexes;
        Ok(())
    }

    /// Table foreign keys.
    ///
    /// @signature (self) -> typing.Sequence[ForeignKey]
    /// @setter typing.Iterable[ForeignKey]
    #[getter]
    fn foreign_keys(&self) -> Vec<PyForeignKey> {
        let lock = self.0.lock();
        lock.foreign_keys
            .iter()
            .map(|x| PyForeignKey(std::sync::Arc::clone(&x.0)))
            .collect()
    }

    #[setter]
    fn set_foreign_keys(&self, val: Vec<pyo3::Bound<'_, pyo3::PyAny>>) -> pyo3::PyResult<()> {
        let mut foreign_keys: Vec<PyForeignKey> = Vec::new();

        for object in val.iter() {
            unsafe {
                let type_ptr = pyo3::ffi::Py_TYPE(object.as_ptr());

                if type_ptr == crate::typeref::FOREIGN_KEY_TYPE {
                    let item = object.cast_unchecked::<PyForeignKey>();
                    foreign_keys.push(PyForeignKey(std::sync::Arc::clone(&item.get().0)));
                } else {
                    return Err(typeerror!(
                        "expected typing.Iterable[ForeignKey], got typing.Iterable[{:?}]",
                        object.py(),
                        object.as_ptr()
                    ));
                }
            }
        }

        self.0.lock().foreign_keys = foreign_keys;
        Ok(())
    }

    /// Table check constraints.
    ///
    /// @signature (self) -> typing.Sequence[Expr]
    /// @setter typing.Iterable[Expr]
    #[getter]
    fn checks(&self) -> Vec<PyExpr> {
        let lock = self.0.lock();
        lock.checks.clone()
    }

    #[setter]
    fn set_checks(&self, val: Vec<pyo3::Bound<'_, pyo3::PyAny>>) -> pyo3::PyResult<()> {
        let mut checks: Vec<PyExpr> = Vec::new();

        for object in val.iter() {
            unsafe {
                let type_ptr = pyo3::ffi::Py_TYPE(object.as_ptr());

                if type_ptr == crate::typeref::FOREIGN_KEY_TYPE {
                    let item = object.cast_unchecked::<PyExpr>();
                    checks.push(item.get().clone());
                } else {
                    return Err(typeerror!(
                        "expected typing.Iterable[Expr], got typing.Iterable[{:?}]",
                        object.py(),
                        object.as_ptr()
                    ));
                }
            }
        }

        self.0.lock().checks = checks;
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
        self.0.lock().options = val;
    }

    /// Whether to use IF NOT EXISTS clause to avoid errors if table exists.
    /// Shorthand for `self.options & self.OPT_IF_NOT_EXISTS > 0`.
    ///
    /// @signature (self) -> bool
    #[getter]
    fn if_not_exists(&self) -> bool {
        self.0.lock().options & Self::OPT_IF_NOT_EXISTS > 0
    }

    /// Whether this is a temporary table that exists only for the session.
    /// Shorthand for `self.options & self.OPT_TEMPORARY > 0`.
    ///
    /// @signature (self) -> bool
    #[getter]
    fn temporary(&self) -> bool {
        self.0.lock().options & Self::OPT_TEMPORARY > 0
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

    fn to_sql(&self, py: pyo3::Python<'_>, backend: String) -> pyo3::PyResult<String> {
        let lock = self.0.lock();
        let stmt = lock.to_sea_query(py);

        let mut sqls = vec![build_schema_statement!(&backend, stmt)?];

        for ix in lock.indexes.iter() {
            let ix_lock = ix.0.lock();

            if ix_lock.options & PyIndex::OPT_PRIMARY > 0
                || ix_lock.options & PyIndex::OPT_UNIQUE > 0
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

            sqls.push(build_schema_statement!(&backend, index_stmt)?);
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
                write!(s, "{}", col.__repr__()).unwrap();
            } else {
                write!(s, "{}, ", col.__repr__()).unwrap();
            }
        }

        write!(s, "] indexes=[").unwrap();

        let n = lock.indexes.len();
        for (index, ix) in lock.indexes.iter().enumerate() {
            if index + 1 == n {
                write!(s, "{}", ix.__repr__()).unwrap();
            } else {
                write!(s, "{}, ", ix.__repr__()).unwrap();
            }
        }

        write!(s, "] foreign_keys=[").unwrap();

        let n = lock.foreign_keys.len();
        for (index, fk) in lock.foreign_keys.iter().enumerate() {
            if index + 1 == n {
                write!(s, "{}", fk.__repr__()).unwrap();
            } else {
                write!(s, "{}, ", fk.__repr__()).unwrap();
            }
        }

        write!(s, "] checks=[").unwrap();

        let n = lock.checks.len();
        for (index, expr) in lock.checks.iter().enumerate() {
            if index + 1 == n {
                write!(s, "{}", expr.__repr__()).unwrap();
            } else {
                write!(s, "{}, ", expr.__repr__()).unwrap();
            }
        }
        write!(s, "]").unwrap();

        if lock.options & Self::OPT_IF_NOT_EXISTS > 0 {
            write!(s, " OPT_IF_NOT_EXISTS").unwrap();
        }
        if lock.options & Self::OPT_TEMPORARY > 0 {
            write!(s, " OPT_TEMPORARY").unwrap();
        }

        if let Some(x) = &lock.comment {
            write!(s, " comment={x}").unwrap();
        }
        if let Some(x) = &lock.engine {
            write!(s, " engine={x}").unwrap();
        }
        if let Some(x) = &lock.collate {
            write!(s, " collate={x}").unwrap();
        }
        if let Some(x) = &lock.character_set {
            write!(s, " character_set={x}").unwrap();
        }

        write!(s, ">").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}
