use crate::{
    common::{PyQueryStatement, PyTableName},
    expression::PyExpr,
    query::{ordering::PyOrderingClause, returning::PyReturningClause},
    utils::ToSeaQuery,
};

implement_state_pyclass! {
    /// Builds DELETE SQL statements with a fluent interface.
    ///
    /// Provides a chainable API for constructing DELETE queries with support for:
    /// - WHERE conditions for filtering
    /// - LIMIT for restricting deletion count
    /// - ORDER BY for determining deletion order
    /// - RETURNING clauses for getting deleted data
    ///
    /// @signature (table: Table | TableName | str)
    pub struct [extends=PyQueryStatement] PyDelete(DeleteState) as "Delete" {
        pub table: PyTableName,
        pub r#where: Option<PyExpr>,
        pub limit: Option<u64>,
        pub returning_clause: Option<PyReturningClause>,
        pub orders: Vec<PyOrderingClause>,
    }
}

impl ToSeaQuery<sea_query::DeleteStatement> for DeleteState {
    fn to_sea_query<'a>(&self, py: pyo3::Python<'a>) -> sea_query::DeleteStatement {
        let mut stmt = sea_query::DeleteStatement::new();
        stmt.from_table(self.table.clone());

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
impl PyDelete {
    #[new]
    fn __new__(table: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<(Self, PyQueryStatement)> {
        let table = PyTableName::try_from(table)?;

        let state = DeleteState {
            table,
            r#where: None,
            limit: None,
            returning_clause: None,
            orders: vec![],
        };
        Ok((state.into(), PyQueryStatement))
    }

    /// Specify the table to delete from.
    ///
    /// @signature (self, table: Table | TableName | str) -> typing.Self
    #[allow(clippy::wrong_self_convention)]
    fn from_table<'a>(
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

    /// Limit the number of rows to delete.
    ///
    /// @signature (self, n: int) -> typing.Self
    fn limit(slf: pyo3::PyRef<'_, Self>, n: u64) -> pyo3::PyRef<'_, Self> {
        {
            let mut lock = slf.0.lock();
            lock.limit = Some(n);
        }

        slf
    }

    /// Specify columns to return from the inserted rows.
    ///
    /// @signature (self, clause: ReturningClause) -> typing.Self
    #[pyo3(signature=(clause))]
    fn returning<'a>(
        slf: pyo3::PyRef<'a, Self>,
        clause: pyo3::Bound<'_, PyReturningClause>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        {
            let mut lock = slf.0.lock();
            lock.returning_clause = Some(clause.get().clone());
        }
        Ok(slf)
    }

    /// Add a WHERE condition to filter rows to delete.
    ///
    /// @signature (self, condition: Expr) -> typing.Self
    fn r#where<'a>(
        slf: pyo3::PyRef<'a, Self>,
        condition: &'a pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        unsafe {
            if pyo3::ffi::Py_TYPE(condition.as_ptr()) != crate::typeref::EXPR_TYPE {
                return Err(typeerror!(
                    "expected Expr, got {:?}",
                    condition.py(),
                    condition.as_ptr()
                ));
            }

            let condition = condition
                .cast_unchecked::<crate::expression::PyExpr>()
                .get()
                .clone();

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
    ///
    /// @signature (self) -> typing.Self
    fn clear_where(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        slf.0.lock().r#where = None;
        slf
    }

    /// Specify the order in which to delete rows. Typically used with
    /// `.limit` method to delete specific rows.
    ///
    /// @signature (self, clause: OrderingClause) -> typing.Self
    #[pyo3(signature=(clause))]
    fn order_by<'a>(
        slf: pyo3::PyRef<'a, Self>,
        clause: pyo3::Bound<'_, PyOrderingClause>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        {
            let mut lock = slf.0.lock();
            lock.orders.push(clause.get().clone());
        }

        Ok(slf)
    }

    /// Remove orders from statement.
    ///
    /// @signature (self) -> typing.Self
    fn clear_order_by(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        slf.0.lock().orders.clear();
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

        write!(s, "<Delete {}", lock.table.__repr__()).unwrap();

        if let Some(x) = lock.limit {
            write!(s, " limit={x}").unwrap();
        }

        if let Some(x) = &lock.r#where {
            write!(s, " where={}", x.__repr__()).unwrap();
        }

        write!(s, " orders=[").unwrap();

        let n = lock.orders.len();
        for (index, expr) in lock.orders.iter().enumerate() {
            if index + 1 == n {
                write!(s, "{}]", expr.__repr__()).unwrap();
            } else {
                write!(s, "{}, ", expr.__repr__()).unwrap();
            }
        }

        if let Some(x) = &lock.returning_clause {
            write!(s, " returning={}", x.__repr__()).unwrap();
        }

        write!(s, ">").unwrap();
        unsafe { String::from_utf8_unchecked(s) }
    }
}
