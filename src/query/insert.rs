use super::base::PyQueryStatement;
use super::on_conflict::PyOnConflict;
use super::returning::PyReturning;
use crate::common::column_ref::PyColumnRef;
use crate::common::expression::PyExpr;
use crate::common::table_ref::PyTableName;
use crate::internal::repr::ReprFormatter;
use crate::internal::{BoundArgs, BoundKwargs, BoundObject, PyObject, RefBoundObject, ToSeaQuery};

#[derive(Debug, Default)]
pub enum InsertValueSource {
    #[default]
    None,
    Single(Vec<PyExpr>),
    Many(Vec<Vec<PyExpr>>),
    Select(
        /// Always is `PySelectStatement`
        PyObject,
    ),
}

crate::implement_pyclass! {
    /// Builds INSERT SQL statements with a fluent interface.
    ///
    /// Provides a chainable API for constructing INSERT queries with support for:
    /// - Single or multiple row insertion
    /// - Conflict resolution (UPSERT)
    /// - RETURNING clauses
    /// - REPLACE functionality
    /// - Default values
    mutable [subclass, extends=PyQueryStatement] PyInsertStatement(InsertStatementState) as "InsertStatement" {
        pub replace: bool,
        pub table: PyTableName,
        pub columns: Vec<String>,
        pub source: InsertValueSource,

        /// Always is `Option<PyOnConflict>`.
        pub on_conflict: Option<PyObject>,
        pub returning_clause: Option<PyReturning>,
        pub default_values: Option<u32>,
    }
}

impl ToSeaQuery<sea_query::InsertStatement> for InsertStatementState {
    #[cfg_attr(feature = "optimize", optimize(speed))]
    fn to_sea_query<'a>(&self, py: pyo3::Python<'a>) -> sea_query::InsertStatement {
        let mut stmt = sea_query::InsertStatement::new();
        stmt.into_table(self.table.clone());

        if self.replace {
            stmt.replace();
        }
        stmt.columns(self.columns.iter().map(sea_query::Alias::new));

        match &self.source {
            InsertValueSource::None => (),
            InsertValueSource::Single(x) => {
                stmt.values(x.iter().map(|x| x.0.clone())).unwrap();
            }
            InsertValueSource::Many(x) => {
                for y in x.iter() {
                    stmt.values(y.iter().map(|x| x.0.clone())).unwrap();
                }
            }
            InsertValueSource::Select(subquery) => unsafe {
                let subquery =
                    subquery.cast_bound_unchecked::<super::select::PySelectStatement>(py);

                stmt.select_from(subquery.get().0.lock().to_sea_query(py))
                    .unwrap();
            },
        }

        if let Some(x) = &self.on_conflict {
            let py_oc = unsafe { x.cast_bound_unchecked::<PyOnConflict>(py) };
            stmt.on_conflict(py_oc.get().0.lock().to_sea_query(py));
        }
        if let Some(rows) = self.default_values {
            stmt.or_default_values_many(rows);
        }
        if let Some(x) = &self.returning_clause {
            stmt.returning(x.0.to_sea_query(py));
        }

        stmt
    }
}

impl InsertStatementState {
    #[inline]
    fn values_from_dictionary(
        &mut self,
        kwds: pyo3::Bound<'_, pyo3::types::PyDict>,
    ) -> pyo3::PyResult<()> {
        use pyo3::types::{PyAnyMethods, PyDictMethods};

        if !self.columns.is_empty() && self.columns.len() != kwds.len() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "values length isn't equal to columns length - this occurres when you're calling \
                 `.values()` method multiple times with different columns.",
            ));
        }

        let mut cols = Vec::with_capacity(kwds.len());
        let mut vals = Vec::with_capacity(kwds.len());

        unsafe {
            for (key, value) in kwds.into_iter() {
                let key = key.extract::<String>().unwrap_unchecked();

                cols.push(key);
                vals.push(PyExpr::try_from(&value)?);
            }
        }

        match std::mem::take(&mut self.source) {
            InsertValueSource::None | InsertValueSource::Select(_) => {
                self.source = InsertValueSource::Single(vals);
            }
            InsertValueSource::Single(oldvals) => {
                self.source = InsertValueSource::Many(vec![oldvals, vals]);
            }
            InsertValueSource::Many(mut arr_of_vals) => {
                arr_of_vals.push(vals);
                self.source = InsertValueSource::Many(arr_of_vals);
            }
        }
        self.columns = cols;

        Ok(())
    }

    #[inline]
    fn values_from_tuple(&mut self, args: BoundArgs<'_>) -> pyo3::PyResult<()> {
        use pyo3::types::PyTupleMethods;

        if !self.columns.is_empty() && self.columns.len() != args.len() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "values length isn't equal to columns length",
            ));
        }

        let mut vals = Vec::with_capacity(args.len());
        for value in args.iter() {
            vals.push(PyExpr::try_from(&value)?);
        }

        match std::mem::take(&mut self.source) {
            InsertValueSource::None | InsertValueSource::Select(_) => {
                self.source = InsertValueSource::Single(vals);
            }
            InsertValueSource::Single(oldvals) => {
                self.source = InsertValueSource::Many(vec![oldvals, vals]);
            }
            InsertValueSource::Many(mut arr_of_vals) => {
                arr_of_vals.push(vals);
                self.source = InsertValueSource::Many(arr_of_vals);
            }
        }

        Ok(())
    }
}

#[pyo3::pymethods]
impl PyInsertStatement {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(args: BoundArgs<'_>, kwds: Option<BoundKwargs<'_>>) -> (Self, PyQueryStatement) {
        (Self::uninit(), PyQueryStatement)
    }

    pub fn __init__(&self, table: RefBoundObject<'_>) -> pyo3::PyResult<()> {
        let table = PyTableName::try_from(table)?;

        let state = InsertStatementState {
            replace: false,
            table,
            columns: vec![],
            source: InsertValueSource::None,
            on_conflict: None,
            returning_clause: None,
            default_values: None,
        };
        self.0.set(state);
        Ok(())
    }

    /// Convert this INSERT to a REPLACE statement.
    ///
    /// REPLACE will delete existing rows that conflict with the new row
    /// before inserting.
    fn replace(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        {
            let mut lock = slf.0.lock();
            lock.replace = true;
        }

        slf
    }

    /// Specify the target table for insertion.
    fn into<'a>(
        slf: pyo3::PyRef<'a, Self>,
        table: RefBoundObject<'a>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let table = PyTableName::try_from(table)?;

        {
            let mut lock = slf.0.lock();
            lock.table = table;
        }
        Ok(slf)
    }

    /// Specify the columns for insertion.
    ///
    /// There's no need to use this method when you're specifying column
    /// names in `.values` method.
    #[pyo3(signature=(*args))]
    fn columns<'a>(
        slf: pyo3::PyRef<'a, Self>,
        args: BoundArgs<'a>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        use pyo3::types::PyTupleMethods;

        let mut columns = Vec::with_capacity(args.len());

        for col in args.iter() {
            let column_ref = PyColumnRef::try_from(&col)?;

            match column_ref.name {
                Some(x) => columns.push(x.to_string()),
                None => {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "Insert.columns cannot accept asterisk '*'",
                    ))
                }
            }
        }

        {
            let mut lock = slf.0.lock();
            lock.columns = columns;
        }
        Ok(slf)
    }

    /// Specify values to insert. Also you can specify columns using keyword arguments.
    #[pyo3(signature=(*args, **kwds))]
    fn values<'a>(
        slf: pyo3::PyRef<'a, Self>,
        args: BoundArgs<'a>,
        kwds: Option<BoundKwargs<'a>>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        use pyo3::types::PyTupleMethods;

        if !PyTupleMethods::is_empty(args) && kwds.is_some() {
            return crate::new_error!(
                PyTypeError,
                "cannot use both args and kwargs at the same time"
            );
        }

        if let Some(kwds) = kwds {
            let mut lock = slf.0.lock();
            lock.values_from_dictionary(kwds.clone())?;
        } else {
            let mut lock = slf.0.lock();
            lock.values_from_tuple(args)?;
        }

        Ok(slf)
    }

    /// Specify a select query whose values to be inserted.
    fn select_from<'a>(
        slf: pyo3::PyRef<'a, Self>,
        statement: BoundObject<'a>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        unsafe {
            if pyo3::ffi::PyObject_TypeCheck(
                statement.as_ptr(),
                crate::typeref::SELECT_STATEMENT_TYPE,
            ) == 0
            {
                return crate::new_error!(
                    PyTypeError,
                    "expected SelectStatement, got {}",
                    crate::internal::get_type_name(statement.py(), statement.as_ptr())
                );
            }

            // We have to return error if columns mismatch, like what SeaQuery does
            let cast = statement.cast_unchecked::<super::select::PySelectStatement>();

            let val_len = cast.get().0.lock().exprs.len();
            let col_len = slf.0.lock().columns.len();

            if col_len != val_len {
                return crate::new_error!(
                    PyValueError,
                    "columns and values length mismatch: {} != {}",
                    col_len,
                    val_len,
                );
            }
        }

        {
            let mut lock = slf.0.lock();
            lock.source = InsertValueSource::Select(statement.unbind());
        }
        Ok(slf)
    }

    /// Use DEFAULT VALUES if no values were specified. The `rows`
    /// Specifies number of rows to insert with default values.
    #[pyo3(signature=(rows=1))]
    fn or_default_values(slf: pyo3::PyRef<'_, Self>, rows: u32) -> pyo3::PyRef<'_, Self> {
        {
            let mut lock = slf.0.lock();
            lock.default_values = Some(rows);
        }

        slf
    }

    /// Specify conflict resolution behavior (UPSERT).
    fn on_conflict<'a>(
        slf: pyo3::PyRef<'a, Self>,
        action: RefBoundObject<'a>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        unsafe {
            if pyo3::ffi::PyObject_TypeCheck(action.as_ptr(), crate::typeref::ON_CONFLICT_TYPE) == 0
            {
                return crate::new_error!(
                    PyTypeError,
                    "expected OnConflict, got {}",
                    crate::internal::get_type_name(action.py(), action.as_ptr())
                );
            }

            let mut lock = slf.0.lock();
            lock.on_conflict = Some(action.clone().unbind());
        }

        Ok(slf)
    }

    /// Specify columns to return from the inserted rows.
    #[pyo3(signature=(clause))]
    fn returning<'a>(
        slf: pyo3::PyRef<'a, Self>,
        clause: pyo3::Bound<'_, PyReturning>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        {
            let mut lock = slf.0.lock();
            lock.returning_clause = Some(clause.get().clone());
        }
        Ok(slf)
    }

    #[pyo3(signature = (backend, /))]
    #[allow(clippy::wrong_self_convention)]
    fn to_sql(&self, py: pyo3::Python<'_>, backend: String) -> pyo3::PyResult<String> {
        let lock = self.0.lock();
        let stmt = lock.to_sea_query(py);
        drop(lock);

        crate::build_query_statement!(backend, stmt)
    }

    #[pyo3(signature = (backend, /))]
    fn build<'a>(
        &self,
        py: pyo3::Python<'a>,
        backend: String,
    ) -> pyo3::PyResult<(String, BoundObject<'a>)> {
        let lock = self.0.lock();
        let stmt = lock.to_sea_query(py);
        drop(lock);

        crate::build_query_parts!(py, backend, stmt)
    }

    fn __repr__(slf: pyo3::PyRef<'_, Self>) -> String {
        let lock = slf.0.lock();

        let mut fmt = ReprFormatter::new_with_pyref(&slf)
            .optional_boolean("replace", lock.replace)
            .map("into", &lock.table, |x| x.__repr__())
            .take();

        fmt.vec("columns", true)
            .quote_iter(lock.columns.iter())
            .finish(&mut fmt);

        fmt.optional_display("on_conflict", lock.on_conflict.as_ref());

        match &lock.source {
            InsertValueSource::None => {
                fmt.optional_display("default_rows", lock.default_values);
            }
            InsertValueSource::Select(x) => {
                fmt.display("select_from", x);
            }
            InsertValueSource::Single(x) => {
                fmt.vec("values", true)
                    .display_iter(x.iter().map(|x| x.__repr__()))
                    .finish(&mut fmt);
            }
            InsertValueSource::Many(x) => {
                let mut fmtvec = fmt.vec("values", true);

                for (index, nested) in x.iter().enumerate() {
                    fmtvec
                        .vec(index)
                        .display_iter(nested.iter().map(|x| x.__repr__()))
                        .finish(&mut fmtvec);
                }

                fmtvec.finish(&mut fmt);
            }
        }

        fmt.optional_map("returning", lock.returning_clause.as_ref(), |x| {
            x.__repr__()
        })
        .finish()
    }
}
