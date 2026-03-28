use sea_query::IntoIden;

use crate::internal::repr::ReprFormatter;
use crate::internal::{BoundArgs, BoundKwargs, BoundObject, PyObject, RefBoundObject, ToSeaQuery};
use crate::query::base::PyQueryStatement;

pub enum CommonTableExpressionQuery {
    Select(
        /// Always is `SelectStatement`
        PyObject,
    ),
    Delete(
        /// Always is `DeleteStatement`
        PyObject,
    ),
    Update(
        /// Always is `UpdateStatement`
        PyObject,
    ),
    Insert(
        /// Always is `InsertStatement`
        PyObject,
    ),
}

pub struct CommonTableExpression {
    pub name: String,
    pub query: CommonTableExpressionQuery,
    pub columns: Vec<String>,
    pub materialized: Option<bool>,
}

crate::implement_pyclass! {
    /// A WITH clause can contain one or multiple common table expressions (CTEs).
    ///
    /// These named queries can act as a "query local table" that are materialized during execution and
    /// then can be used by the query prefixed with the WITH clause.
    ///
    /// A CTE is a name, column names and a query returning data for those columns.
    ///
    /// Some databases (like sqlite) restrict the acceptable kinds of queries inside of the WITH clause
    /// CTEs. These databases only allow `SelectStatement`s to form a CTE.
    ///
    /// Other databases like postgres allow modification queries (UPDATE, DELETE) inside of the WITH
    /// clause but they have to return a table. (They must have a RETURNING clause).
    ///
    /// RapidQuery doesn't check this or restrict the kind of CTE that you can create
    /// in rust. This means that you can put an UPDATE or DELETE queries into WITH clause and RapidQuery
    /// will succeed in generating that kind of sql query but the execution inside the database will
    /// fail because they are invalid.
    ///
    /// It is your responsibility to ensure that the kind of WITH clause that you put together makes
    /// sense and valid for that database that you are using.
    ///
    /// NOTE that for recursive WITH queries (in sql: "WITH RECURSIVE") you can only have a
    /// single CTE inside of the WITH clause. That query must match certain
    /// requirements:
    ///   * It is a query of UNION or UNION ALL of two queries.
    ///   * The first part of the query (the left side of the UNION) must be executable first in itself.
    ///     It must be non-recursive. (Cannot contain self reference)
    ///   * The self reference must appear in the right hand side of the UNION.
    ///   * The query can only have a single self-reference.
    ///   * Recursive data-modifying statements are not supported, but you can use the results of a
    ///     recursive SELECT query in a data-modifying statement. (like so: WITH RECURSIVE
    ///     cte_name(a,b,c,d) AS (SELECT ... UNION SELECT ... FROM ... JOIN cte_name ON ... WHERE ...)
    ///     DELETE FROM table WHERE table.a = cte_name.a)
    ///
    /// Recursive with query generation will raise `RuntimeError` if you specify more than one CTE.
    mutable [subclass] PyWithClause(WithClauseState) as "WithClause" {
        pub recursive: bool,
        pub cte_expressions: Vec<CommonTableExpression>,

        // TODO
        // search: PySearch,
        // cycle: PyCycle,
    }
}
crate::implement_pyclass! {
    /// A WITH query. A simple SQL query that has a WITH clause (`WithClause`).
    ///
    /// For full description, see `WithClause`'s documentation.
    mutable [subclass, extends=PyQueryStatement] PyWithQuery(WithQueryState) as "WithQuery" {
        /// Always is `PyWithClause`
        pub with_clause: PyObject,
        pub query: CommonTableExpressionQuery,
    }
}

impl TryFrom<RefBoundObject<'_>> for CommonTableExpressionQuery {
    type Error = pyo3::PyErr;

    fn try_from(value: RefBoundObject) -> Result<Self, Self::Error> {
        unsafe {
            let value_ptr = value.as_ptr();

            // SelectStatement
            if pyo3::ffi::PyObject_TypeCheck(value_ptr, crate::typeref::SELECT_STATEMENT_TYPE) == 1
            {
                Ok(Self::Select(value.clone().unbind()))
            }
            // DeleteStatement
            else if pyo3::ffi::PyObject_TypeCheck(
                value_ptr,
                crate::typeref::DELETE_STATEMENT_TYPE,
            ) == 1
            {
                Ok(Self::Delete(value.clone().unbind()))
            }
            // UpdateStatement
            else if pyo3::ffi::PyObject_TypeCheck(
                value_ptr,
                crate::typeref::UPDATE_STATEMENT_TYPE,
            ) == 1
            {
                Ok(Self::Update(value.clone().unbind()))
            }
            // InsertStatement
            else if pyo3::ffi::PyObject_TypeCheck(
                value_ptr,
                crate::typeref::INSERT_STATEMENT_TYPE,
            ) == 1
            {
                Ok(Self::Insert(value.clone().unbind()))
            }
            // Other types
            else {
                crate::new_error!(
                    PyTypeError,
                    "expected SelectStatement, DeleteStatement, UpdateStatement, or \
                     InsertStatement, got {}",
                    crate::internal::get_type_name(value.py(), value_ptr)
                )
            }
        }
    }
}

#[inline(always)]
fn cast_into_with_clause<'a, 'b>(py: pyo3::Python<'a>, value: &'b PyObject) -> &'b PyWithClause
where
    'a: 'b,
{
    let bound = value.bind(py);

    unsafe {
        let casted = bound.cast_unchecked::<PyWithClause>();
        casted.get()
    }
}

impl std::fmt::Display for CommonTableExpressionQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Select(x) => write!(f, "{x}"),
            Self::Update(x) => write!(f, "{x}"),
            Self::Delete(x) => write!(f, "{x}"),
            Self::Insert(x) => write!(f, "{x}"),
        }
    }
}

impl std::fmt::Display for CommonTableExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "('{}', {}", self.name, self.query)?;

        if !self.columns.is_empty() {
            write!(f, ", {:?}", self.columns)?;
        }
        match self.materialized {
            Some(true) => write!(f, ", materialized")?,
            Some(false) => write!(f, ", not materialized")?,
            None => {}
        }

        write!(f, ")")
    }
}

impl ToSeaQuery<sea_query::CommonTableExpression> for CommonTableExpression {
    #[cfg_attr(feature = "optimize", optimize(speed))]
    fn to_sea_query<'a>(&self, py: pyo3::Python<'a>) -> sea_query::CommonTableExpression {
        let mut stmt = {
            if let CommonTableExpressionQuery::Select(x) = &self.query {
                let select =
                    unsafe { x.cast_bound_unchecked::<super::select::PySelectStatement>(py) };

                let result = select.get().0.lock().to_sea_query(py);

                sea_query::CommonTableExpression::from_select(result)
            } else {
                sea_query::CommonTableExpression::new()
            }
        };
        stmt.table_name(sea_query::Alias::new(&self.name).into_iden());

        match &self.query {
            CommonTableExpressionQuery::Select(_) => {}
            CommonTableExpressionQuery::Delete(x) => {
                let delete =
                    unsafe { x.cast_bound_unchecked::<super::delete::PyDeleteStatement>(py) };

                let result = delete.get().0.lock().to_sea_query(py);
                stmt.query(result);
            }
            CommonTableExpressionQuery::Update(x) => {
                let update =
                    unsafe { x.cast_bound_unchecked::<super::update::PyUpdateStatement>(py) };

                let result = update.get().0.lock().to_sea_query(py);
                stmt.query(result);
            }
            CommonTableExpressionQuery::Insert(x) => {
                let insert =
                    unsafe { x.cast_bound_unchecked::<super::insert::PyInsertStatement>(py) };

                let result = insert.get().0.lock().to_sea_query(py);
                stmt.query(result);
            }
        }

        if !self.columns.is_empty() {
            stmt.columns(self.columns.iter().map(sea_query::Alias::new));
        }
        if let Some(x) = self.materialized {
            stmt.materialized(x);
        }

        stmt
    }
}

impl ToSeaQuery<sea_query::WithClause> for WithClauseState {
    #[cfg_attr(feature = "optimize", optimize(speed))]
    fn to_sea_query<'a>(&self, py: pyo3::Python<'a>) -> sea_query::WithClause {
        let mut stmt = sea_query::WithClause::new();

        if self.recursive {
            stmt.recursive(true);
        }

        for cte in self.cte_expressions.iter() {
            stmt.cte(cte.to_sea_query(py));
        }

        stmt
    }
}

impl ToSeaQuery<sea_query::WithQuery> for WithQueryState {
    #[cfg_attr(feature = "optimize", optimize(speed))]
    fn to_sea_query<'a>(&self, py: pyo3::Python<'a>) -> sea_query::WithQuery {
        let clause = unsafe {
            let x = cast_into_with_clause(py, &self.with_clause);
            x.0.lock().to_sea_query(py)
        };

        match &self.query {
            CommonTableExpressionQuery::Select(x) => {
                let select =
                    unsafe { x.cast_bound_unchecked::<super::select::PySelectStatement>(py) };

                let result = select.get().0.lock().to_sea_query(py);
                clause.query(result)
            }
            CommonTableExpressionQuery::Delete(x) => {
                let delete =
                    unsafe { x.cast_bound_unchecked::<super::delete::PyDeleteStatement>(py) };

                let result = delete.get().0.lock().to_sea_query(py);
                clause.query(result)
            }
            CommonTableExpressionQuery::Update(x) => {
                let update =
                    unsafe { x.cast_bound_unchecked::<super::update::PyUpdateStatement>(py) };

                let result = update.get().0.lock().to_sea_query(py);
                clause.query(result)
            }
            CommonTableExpressionQuery::Insert(x) => {
                let insert =
                    unsafe { x.cast_bound_unchecked::<super::insert::PyInsertStatement>(py) };

                let result = insert.get().0.lock().to_sea_query(py);
                clause.query(result)
            }
        }
    }
}

#[pyo3::pymethods]
impl PyWithClause {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(args: BoundArgs<'_>, kwds: Option<BoundKwargs<'_>>) -> Self {
        Self::uninit()
    }

    fn __init__(&self) -> pyo3::PyResult<()> {
        let state = WithClauseState {
            recursive: false,
            cte_expressions: vec![],
        };
        self.0.set(state);
        Ok(())
    }

    /// Sets whether this clause is a recursive with clause of not.
    /// If set to true it will generate a 'WITH RECURSIVE' query.
    ///
    /// You can only specify a single CTE containing a union query
    /// if this is set to true.
    fn recursive<'a>(slf: pyo3::PyRef<'a, Self>) -> pyo3::PyRef<'a, Self> {
        slf.0.lock().recursive = true;
        slf
    }

    /// Add a CTE to this with clause.
    #[pyo3(signature=(name, query, columns=Vec::new(), materialized=None))]
    fn cte<'a>(
        slf: pyo3::PyRef<'a, Self>,
        name: String,
        query: RefBoundObject<'a>,
        columns: Vec<String>,
        materialized: Option<bool>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let query = CommonTableExpressionQuery::try_from(query)?;

        slf.0.lock().cte_expressions.push(CommonTableExpression {
            name,
            query,
            columns,
            materialized,
        });
        Ok(slf)
    }

    /// You can turn this into a `WithQuery` using this function.
    /// The resulting WITH query will execute the argument query with this WITH clause.
    fn query<'a>(
        slf: pyo3::PyRef<'a, Self>,
        val: RefBoundObject<'a>,
    ) -> pyo3::PyResult<pyo3::Bound<'a, PyWithQuery>> {
        let query = CommonTableExpressionQuery::try_from(val)?;

        let state = WithQueryState {
            with_clause: unsafe { pyo3::Bound::from_borrowed_ptr(slf.py(), slf.as_ptr()).unbind() },
            query,
        };
        let result: PyWithQuery = state.into();

        pyo3::Bound::new(slf.py(), (result, PyQueryStatement))
    }

    fn __repr__(slf: pyo3::PyRef<'_, Self>) -> String {
        let lock = slf.0.lock();

        let mut fmt = ReprFormatter::new_with_pyref(&slf)
            .optional_boolean("recursive", lock.recursive)
            .take();

        fmt.vec("cte", false)
            .display_iter(lock.cte_expressions.iter())
            .finish(&mut fmt);

        fmt.finish()
    }
}

#[pyo3::pymethods]
impl PyWithQuery {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(args: BoundArgs<'_>, kwds: Option<BoundKwargs<'_>>) -> (Self, PyQueryStatement) {
        (Self::uninit(), PyQueryStatement)
    }

    fn __init__(&self, clause: BoundObject, query: RefBoundObject) -> pyo3::PyResult<()> {
        unsafe {
            if pyo3::ffi::PyObject_TypeCheck(clause.as_ptr(), crate::typeref::WITH_CLAUSE_TYPE) == 0
            {
                return crate::new_error!(
                    PyTypeError,
                    "expected WithClause, got {}",
                    crate::internal::get_type_name(clause.py(), clause.as_ptr())
                );
            }
        }
        let query = CommonTableExpressionQuery::try_from(query)?;

        let state = WithQueryState {
            with_clause: clause.unbind(),
            query,
        };
        self.0.set(state);
        Ok(())
    }

    /// Same as `WithClause.recursive` method.
    fn recursive<'a>(slf: pyo3::PyRef<'a, Self>) -> pyo3::PyRef<'a, Self> {
        {
            let lock = slf.0.lock();
            let clause = cast_into_with_clause(slf.py(), &lock.with_clause);

            clause.0.lock().recursive = true;
        }

        slf
    }

    /// Same as `WithClause.cte` method.
    ///
    /// Useful when you wanna add a new CTE to WITH clause.
    #[pyo3(signature=(name, query, columns=Vec::new(), materialized=None))]
    fn cte<'a>(
        slf: pyo3::PyRef<'a, Self>,
        name: String,
        query: RefBoundObject<'a>,
        columns: Vec<String>,
        materialized: Option<bool>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let query = CommonTableExpressionQuery::try_from(query)?;

        {
            let lock = slf.0.lock();
            let clause = cast_into_with_clause(slf.py(), &lock.with_clause);

            clause.0.lock().cte_expressions.push(CommonTableExpression {
                name,
                query,
                columns,
                materialized,
            });
        }

        Ok(slf)
    }

    #[pyo3(signature = (backend, /))]
    #[allow(clippy::wrong_self_convention)]
    fn to_sql(&self, py: pyo3::Python<'_>, backend: String) -> pyo3::PyResult<String> {
        use sea_query::QueryStatementBuilder;

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
        use sea_query::QueryStatementBuilder;

        let lock = self.0.lock();
        let stmt = lock.to_sea_query(py);
        drop(lock);

        crate::build_query_parts!(py, backend, stmt)
    }

    fn __repr__(slf: pyo3::PyRef<'_, Self>) -> String {
        let lock = slf.0.lock();

        ReprFormatter::new_with_pyref(&slf)
            .display("clause", &lock.with_clause)
            .display("query", &lock.query)
            .finish()
    }
}
