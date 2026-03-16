use super::base::PySchemaStatement;
use crate::common::table_ref::PyTableName;
use crate::internal::repr::ReprFormatter;
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
    #[getter]
    fn name(&self) -> PyTableName {
        let lock = self.0.lock();
        lock.name.clone()
    }

    /// Table columns.
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
    #[getter]
    fn if_not_exists(&self) -> bool {
        self.0.lock().options & OPT_IF_NOT_EXISTS > 0
    }

    /// Whether this is a temporary table that exists only for the session.
    #[getter]
    fn temporary(&self) -> bool {
        self.0.lock().options & OPT_TEMPORARY > 0
    }

    /// Comment describing the purpose of this table.
    #[getter]
    fn comment(&self) -> Option<String> {
        self.0.lock().comment.clone()
    }

    #[setter]
    fn set_comment(&self, val: Option<String>) {
        self.0.lock().comment = val;
    }

    /// Storage engine for the table (e.g., InnoDB, MyISAM for MySQL).
    #[getter]
    fn engine(&self) -> Option<String> {
        self.0.lock().engine.clone()
    }

    #[setter]
    fn set_engine(&self, val: Option<String>) {
        self.0.lock().engine = val;
    }

    /// Collation for string comparisons and sorting in this table.
    #[getter]
    fn collate(&self) -> Option<String> {
        self.0.lock().collate.clone()
    }

    #[setter]
    fn set_collate(&self, val: Option<String>) {
        self.0.lock().collate = val;
    }

    /// Character set encoding for text data in this table.
    #[getter]
    fn character_set(&self) -> Option<String> {
        self.0.lock().character_set.clone()
    }

    #[setter]
    fn set_character_set(&self, val: Option<String>) {
        self.0.lock().character_set = val;
    }

    /// Additional table-specific options for the CREATE TABLE statement.
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
                    "You should always name for indexes that aren't primary or unique",
                ));
            }

            let mut index_stmt = ix_lock.to_sea_query(py);
            index_stmt.table(lock.name.clone());

            sqls.push(crate::build_schema_statement!(&backend, index_stmt)?);
        }

        Ok(sqls.join(";\n"))
    }

    #[getter]
    fn __table_name__(&self) -> PyTableName {
        self.0.lock().name.clone()
    }

    fn __repr__(slf: pyo3::PyRef<'_, Self>) -> String {
        let lock = slf.0.lock();

        let mut fmt = ReprFormatter::new_with_pyref(&slf)
            .map("name", &lock.name, |x| x.__repr__())
            .take();

        fmt.vec("columns", true)
            .display_iter(lock.columns.iter())
            .finish(&mut fmt);

        fmt.vec("indexes", true)
            .display_iter(lock.indexes.iter())
            .finish(&mut fmt);

        fmt.vec("foreign_keys", true)
            .display_iter(lock.foreign_keys.iter())
            .finish(&mut fmt);

        fmt.vec("checks", true)
            .display_iter(lock.checks.iter())
            .finish(&mut fmt);

        fmt.optional_boolean("if_not_exists", lock.options & OPT_IF_NOT_EXISTS > 0)
            .optional_boolean("temporary", lock.options & OPT_TEMPORARY > 0)
            .optional_quote("comment", lock.comment.as_ref())
            .optional_quote("engine", lock.engine.as_ref())
            .optional_quote("collate", lock.collate.as_ref())
            .optional_quote("character_set", lock.character_set.as_ref())
            .finish()
    }
}
