use super::clauses::ReturningClause;
use pyo3::types::{PyAnyMethods, PyDictMethods, PyTupleMethods};

use crate::{
    common::{PyQueryStatement, PyTableName},
    expression::PyExpr,
    query::on_conflict::PyOnConflict,
    utils::ToSeaQuery,
};

#[derive(Debug, Default)]
pub enum InsertValueSource {
    #[default]
    None,
    Single(Vec<PyExpr>),
    Many(Vec<Vec<PyExpr>>),
    // TODO
    // Select(pyo3::Py<pyo3::PyAny>),
}

implement_state_pyclass! {
    /// Builds INSERT SQL statements with a fluent interface.
    ///
    /// Provides a chainable API for constructing INSERT queries with support for:
    /// - Single or multiple row insertion
    /// - Conflict resolution (UPSERT)
    /// - RETURNING clauses
    /// - REPLACE functionality
    /// - Default values
    ///
    /// @signature (table: Table | TableName | str)
    pub struct [extends=PyQueryStatement] PyInsert(InsertState) as "Insert" {
        pub replace: bool,
        pub table: PyTableName,
        pub columns: Vec<String>,
        pub source: InsertValueSource,
        pub on_conflict: Option<PyOnConflict>,
        pub returning_clause: ReturningClause,
        pub default_values: Option<u32>,
    }
}

impl ToSeaQuery<sea_query::InsertStatement> for InsertState {
    fn to_sea_query<'a>(&self, py: pyo3::Python<'a>) -> sea_query::InsertStatement {
        let mut stmt = sea_query::InsertStatement::new();
        stmt.into_table(self.table.clone());

        if self.replace {
            stmt.replace();
        }
        stmt.columns(self.columns.iter().map(sea_query::Alias::new));

        match &self.source {
            InsertValueSource::None => (),
            InsertValueSource::Single(x) => unsafe {
                stmt.values(x.iter().map(|x| x.0.clone())).unwrap();
            },
            InsertValueSource::Many(x) => unsafe {
                for y in x.iter() {
                    stmt.values(y.iter().map(|x| x.0.clone())).unwrap();
                }
            },
        }

        if let Some(x) = &self.on_conflict {
            let lock = x.0.lock();
            stmt.on_conflict(lock.to_sea_query(py));
        }

        if let Some(rows) = self.default_values {
            stmt.or_default_values_many(rows);
        }

        match &self.returning_clause {
            ReturningClause::None => (),
            ReturningClause::All => {
                stmt.returning_all();
            }
            ReturningClause::Columns(x) => {
                stmt.returning(sea_query::ReturningClause::Columns(
                    x.iter()
                        .map(|x| sea_query::ColumnRef::Column(x.clone()))
                        .collect(),
                ));
            }
        }
        stmt
    }
}

impl InsertState {
    #[inline]
    fn values_from_dictionary(
        &mut self,
        kwds: pyo3::Bound<'_, pyo3::types::PyDict>,
    ) -> pyo3::PyResult<()> {
        if !self.columns.is_empty() && self.columns.len() != kwds.len() {
            return Err(
                pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "values length isn't equal to columns length - this occurres when you're calling `.values()` method multiple times with different columns."
                )
            );
        }

        let mut cols = Vec::with_capacity(kwds.len());
        let mut vals = Vec::with_capacity(kwds.len());

        unsafe {
            for (key, value) in kwds.iter() {
                let key = key.extract::<String>().unwrap_unchecked();

                cols.push(key);
                vals.push(PyExpr::try_from(&value)?);
            }
        }

        match std::mem::take(&mut self.source) {
            InsertValueSource::None => {
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
    fn values_from_tuple(
        &mut self,
        args: pyo3::Bound<'_, pyo3::types::PyTuple>,
    ) -> pyo3::PyResult<()> {
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
            InsertValueSource::None => {
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
impl PyInsert {
    #[new]
    fn __new__(table: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<(Self, PyQueryStatement)> {
        let table = PyTableName::try_from(table)?;

        let state = InsertState {
            replace: false,
            table,
            columns: vec![],
            source: InsertValueSource::None,
            on_conflict: None,
            returning_clause: ReturningClause::None,
            default_values: None,
        };
        Ok((state.into(), PyQueryStatement))
    }

    /// Convert this INSERT to a REPLACE statement.
    ///
    /// REPLACE will delete existing rows that conflict with the new row
    /// before inserting.
    ///
    /// @signature (self) -> typing.Self
    fn replace(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        {
            let mut lock = slf.0.lock();
            lock.replace = true;
        }

        slf
    }

    /// Specify the target table for insertion.
    ///
    /// @signature (self, table: Table | TableName | str) -> typing.Self
    fn into<'a>(
        slf: pyo3::PyRef<'a, Self>,
        table: &'a pyo3::Bound<'_, pyo3::PyAny>,
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
    ///
    /// @signature (self, *args: Column | ColumnRef | str) -> typing.Self
    #[pyo3(signature=(*args))]
    fn columns<'a>(
        slf: pyo3::PyRef<'a, Self>,
        args: &'a pyo3::Bound<'_, pyo3::types::PyTuple>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let mut columns = Vec::with_capacity(args.len());

        for col in args.iter() {
            let column_ref = crate::common::PyColumnRef::try_from(&col)?;

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
    ///
    /// @overload (self, *args: object) -> typing.Self
    /// @overload (self, **kwds: object) -> typing.Self
    /// @signature (self, *args: object, **kwds: object) -> typing.Self
    #[pyo3(signature=(*args, **kwds))]
    fn values<'a>(
        slf: pyo3::PyRef<'a, Self>,
        args: &'a pyo3::Bound<'_, pyo3::types::PyTuple>,
        kwds: Option<&'a pyo3::Bound<'_, pyo3::types::PyDict>>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        if !PyTupleMethods::is_empty(args) && kwds.is_some() {
            return Err(typeerror!(
                "cannot use both args and kwargs at the same time",
            ));
        }

        if !PyTupleMethods::is_empty(args) {
            let mut lock = slf.0.lock();
            lock.values_from_tuple(args.clone())?;
        } else if kwds.is_some() {
            let mut lock = slf.0.lock();
            lock.values_from_dictionary(kwds.unwrap().clone())?;
        } else {
            return Err(typeerror!("no arguments provided"));
        }

        Ok(slf)
    }

    /// Use DEFAULT VALUES if no values were specified. The `rows`
    /// Specifies number of rows to insert with default values.
    ///
    /// @signature (self, rows: int = 1) -> typing.Self
    #[pyo3(signature=(rows=1))]
    fn or_default_values(slf: pyo3::PyRef<'_, Self>, rows: u32) -> pyo3::PyRef<'_, Self> {
        {
            let mut lock = slf.0.lock();
            lock.default_values = Some(rows);
        }

        slf
    }

    /// Specify conflict resolution behavior (UPSERT).
    ///
    /// @signature (self, action: OnConflict) -> typing.Self
    fn on_conflict<'a>(
        slf: pyo3::PyRef<'a, Self>,
        action: &'a pyo3::Bound<'a, PyOnConflict>,
    ) -> pyo3::PyRef<'a, Self> {
        let action_inner = std::sync::Arc::clone(&action.get().0);

        {
            let mut lock = slf.0.lock();
            lock.on_conflict = Some(PyOnConflict(action_inner));
        }

        slf
    }

    /// Specify columns to return from the inserted rows.
    ///
    /// @signature (self, *args: Column | ColumnRef | str) -> typing.Self
    #[pyo3(signature=(*args))]
    fn returning<'a>(
        slf: pyo3::PyRef<'a, Self>,
        args: pyo3::Bound<'_, pyo3::types::PyTuple>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        {
            let mut lock = slf.0.lock();
            lock.returning_clause = ReturningClause::new(args)?;
        }
        Ok(slf)
    }

    /// Return all columns from the inserted rows. Same as `self.returning("*")`.
    ///
    /// @signature (self) -> typing.Self
    fn returning_all(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        {
            let mut lock = slf.0.lock();
            lock.returning_clause = ReturningClause::All;
        }
        slf
    }

    #[pyo3(signature = (backend, /))]
    #[allow(clippy::wrong_self_convention)]
    fn to_sql(&self, py: pyo3::Python<'_>, backend: String) -> pyo3::PyResult<String> {
        let lock = self.0.lock();
        let stmt = lock.to_sea_query(py);
        drop(lock);

        build_query_statement!(backend, stmt)
    }

    #[pyo3(signature = (backend, /))]
    fn build<'a>(
        &self,
        py: pyo3::Python<'a>,
        backend: String,
    ) -> pyo3::PyResult<(String, pyo3::Bound<'a, pyo3::PyAny>)> {
        let lock = self.0.lock();
        let stmt = lock.to_sea_query(py);
        drop(lock);

        build_query_parts!(py, backend, stmt)
    }

    fn __repr__(&self) -> String {
        use std::io::Write;

        let lock = self.0.lock();
        let mut s = Vec::<u8>::with_capacity(30);

        write!(s, "<Insert").unwrap();
        if lock.replace {
            write!(s, " replace=True").unwrap();
        }
        write!(s, " into={}", lock.table.__repr__()).unwrap();

        if !lock.columns.is_empty() {
            write!(s, " columns={:?}", lock.columns).unwrap();
        }
        if let Some(x) = &lock.on_conflict {
            write!(s, " on_conflict={}", x.__repr__()).unwrap();
        }

        match &lock.source {
            InsertValueSource::None => {
                if let Some(x) = lock.default_values {
                    write!(s, " default_rows={x}").unwrap();
                }
            }
            InsertValueSource::Single(x) => {
                write!(s, " values=[").unwrap();

                let n = x.len();
                for (index, ix) in x.iter().enumerate() {
                    if index + 1 == n {
                        write!(s, "{}", ix.__repr__()).unwrap();
                    } else {
                        write!(s, "{}, ", ix.__repr__()).unwrap();
                    }
                }
                write!(s, "]").unwrap();
            }
            InsertValueSource::Many(x) => {
                write!(s, " values=[[").unwrap();

                let n = x.len();
                for (index_1, nested) in x.iter().enumerate() {
                    let j = nested.len();
                    for (index_2, val) in nested.iter().enumerate() {
                        if index_2 + 1 == j {
                            write!(s, "{}", val.__repr__()).unwrap();
                        } else {
                            write!(s, "{}, ", val.__repr__()).unwrap();
                        }
                    }

                    if index_1 + 1 < n {
                        write!(s, "], [").unwrap();
                    }
                }
                write!(s, "]]").unwrap();
            }
        }

        match &lock.returning_clause {
            ReturningClause::None => (),
            ReturningClause::All => {
                write!(s, " returning_all").unwrap();
            }
            ReturningClause::Columns(x) => {
                write!(s, " returning={x:?}").unwrap();
            }
        }

        write!(s, ">").unwrap();
        unsafe { String::from_utf8_unchecked(s) }
    }
}
