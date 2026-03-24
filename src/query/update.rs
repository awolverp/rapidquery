use sea_query::IntoIden;

use super::base::PyQueryStatement;
use super::ordering::PyOrdering;
use super::returning::PyReturning;
use crate::common::expression::PyExpr;
use crate::common::table_ref::PyTableName;
use crate::internal::repr::ReprFormatter;
use crate::internal::{BoundArgs, BoundKwargs, BoundObject, RefBoundObject, ToSeaQuery};

crate::implement_pyclass! {
    /// Builds UPDATE SQL statements with a fluent interface.
    ///
    /// Provides a chainable API for constructing UPDATE queries with support for:
    /// - Setting column values
    /// - WHERE conditions for filtering
    /// - LIMIT for restricting update count
    /// - ORDER BY for determining update order
    /// - RETURNING clauses for getting updated data
    mutable [subclass, extends=PyQueryStatement] PyUpdateStatement(UpdateStatementState) as "UpdateStatement" {
        pub table: PyTableName,
        pub from_table: Option<PyTableName>,
        pub values: Vec<(sea_query::DynIden, PyExpr)>,
        pub r#where: Option<PyExpr>,
        pub limit: Option<u64>,
        pub returning_clause: Option<PyReturning>,
        pub orders: Vec<PyOrdering>,
    }
}

impl ToSeaQuery<sea_query::UpdateStatement> for UpdateStatementState {
    #[cfg_attr(feature = "optimize", optimize(speed))]
    fn to_sea_query<'a>(&self, py: pyo3::Python<'a>) -> sea_query::UpdateStatement {
        let mut stmt = sea_query::UpdateStatement::new();
        stmt.table(self.table.clone());

        if let Some(x) = &self.from_table {
            stmt.from(x.clone());
        }
        stmt.values(self.values.iter().map(|(x, y)| (x.clone(), y.0.clone())));

        if let Some(x) = &self.r#where {
            stmt.and_where(x.0.clone());
        }
        if let Some(x) = self.limit {
            stmt.limit(x);
        }
        if let Some(x) = &self.returning_clause {
            stmt.returning(x.0.to_sea_query(py));
        }

        for order in self.orders.iter() {
            if let Some(x) = order.null_order {
                stmt.order_by_expr_with_nulls(order.target.0.clone(), order.order.clone(), x);
            } else {
                stmt.order_by_expr(order.target.0.clone(), order.order.clone());
            }
        }

        stmt
    }
}

#[pyo3::pymethods]
impl PyUpdateStatement {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(args: BoundArgs<'_>, kwds: Option<BoundKwargs<'_>>) -> (Self, PyQueryStatement) {
        (Self::uninit(), PyQueryStatement)
    }

    pub fn __init__(&self, table: RefBoundObject<'_>) -> pyo3::PyResult<()> {
        let table = PyTableName::try_from(table)?;

        let state = UpdateStatementState {
            table,
            from_table: None,
            values: vec![],
            r#where: None,
            limit: None,
            returning_clause: None,
            orders: vec![],
        };
        self.0.set(state);
        Ok(())
    }

    /// Specify the table to update.
    fn table<'a>(
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

    /// Update using data from another table (`UPDATE .. FROM ..`).
    ///
    /// MySQL doesn't support the UPDATE FROM syntax. And the current implementation attempt to
    /// tranform it to the UPDATE JOIN syntax, which only works for one join target.
    #[allow(clippy::wrong_self_convention)]
    fn from_table<'a>(
        slf: pyo3::PyRef<'a, Self>,
        table: RefBoundObject<'a>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let table = PyTableName::try_from(table)?;

        {
            let mut lock = slf.0.lock();
            lock.from_table = Some(table);
        }
        Ok(slf)
    }

    /// Limit the number of rows to update.
    fn limit(slf: pyo3::PyRef<'_, Self>, n: u64) -> pyo3::PyRef<'_, Self> {
        {
            let mut lock = slf.0.lock();
            lock.limit = Some(n);
        }

        slf
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

    /// Add a WHERE condition to filter rows to update.
    fn r#where<'a>(
        slf: pyo3::PyRef<'a, Self>,
        condition: RefBoundObject<'a>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        unsafe {
            if pyo3::ffi::Py_TYPE(condition.as_ptr()) != crate::typeref::EXPR_TYPE {
                return crate::new_error!(
                    PyTypeError,
                    "expected Expr, got {}",
                    crate::internal::get_type_name(condition.py(), condition.as_ptr())
                );
            }

            let condition = condition.cast_unchecked::<PyExpr>().get().clone();
            let mut lock = slf.0.lock();

            match std::mem::take(&mut lock.r#where) {
                None => {
                    lock.r#where = Some(condition);
                }
                Some(x) => {
                    lock.r#where = Some(PyExpr(x.0.and(condition.0)));
                }
            }
        }

        Ok(slf)
    }

    /// Remove where conditions from statement.
    fn clear_where(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        slf.0.lock().r#where = None;
        slf
    }

    /// Specify the order in which to delete rows. Typically used with
    /// `.limit` method to delete specific rows.
    #[pyo3(signature=(clause))]
    fn order_by<'a>(
        slf: pyo3::PyRef<'a, Self>,
        clause: pyo3::Bound<'_, PyOrdering>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        {
            let mut lock = slf.0.lock();
            lock.orders.push(clause.get().clone());
        }

        Ok(slf)
    }

    /// Remove orders from statement.
    fn clear_order_by(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        slf.0.lock().orders.clear();
        slf
    }

    /// Specify columns and their new values.
    #[pyo3(signature=(**kwds))]
    fn values<'a>(
        slf: pyo3::PyRef<'a, Self>,
        kwds: Option<BoundKwargs<'a>>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        use pyo3::types::{PyAnyMethods, PyDictMethods};

        if kwds.is_none() {
            return Ok(slf);
        }

        let kwds = unsafe { kwds.unwrap_unchecked() };
        let mut values = Vec::with_capacity(kwds.len());

        unsafe {
            for (key, value) in kwds.iter() {
                let key = key.extract::<String>().unwrap_unchecked();
                values.push((
                    (sea_query::Alias::new(key).into_iden()),
                    PyExpr::try_from(&value)?,
                ));
            }
        }

        {
            let mut lock = slf.0.lock();
            lock.values.append(&mut values);
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
            .map("table", &lock.table, |x| x.__repr__())
            .optional_map("from_table", lock.from_table.as_ref(), |x| x.__repr__())
            .take();

        fmt.vec("values", false)
            .display_iter(
                lock.values
                    .iter()
                    .map(|x| format!("('{}', {})", x.0.to_string(), x.1.__repr__())),
            )
            .finish(&mut fmt);

        fmt.vec("orders", true)
            .display_iter(lock.orders.iter().map(|x| x.__repr__()))
            .finish(&mut fmt);

        fmt.optional_display("limit", lock.limit)
            .optional_map("where", lock.r#where.as_ref(), |x| x.__repr__())
            .optional_map("returning", lock.returning_clause.as_ref(), |x| {
                x.__repr__()
            })
            .finish()
    }
}
