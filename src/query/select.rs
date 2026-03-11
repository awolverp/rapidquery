use pyo3::types::PyAnyMethods;
use sea_query::IntoIden;

// use pyo3::types::PyAnyMethods;
// use sea_query::IntoIden;
use super::base::PyQueryStatement;
// use crate::expression::PyFunc;
use super::ordering::PyOrdering;
use super::window::PyWindowStatement;
use crate::common::column_ref::PyColumnRef;
use crate::common::expression::PyExpr;
use crate::common::table_ref::PyTableName;

/// Window type in [`PySelectExpr`]
pub enum SelectExprWindow {
    Name(sea_query::DynIden),
    Window(
        /// Always is `PyWindowStatement`
        pyo3::Py<pyo3::PyAny>,
    ),
}

/// Select references in [`PySelectStatement`]
///
/// It exactly does what [`sea_query::TableRef`] does
pub enum SelectReference {
    // TODO: support from_values
    SubQuery(
        /// Always is `PySelectStatement`
        pyo3::Py<pyo3::PyAny>,
        /// Alias name
        sea_query::DynIden,
    ),
    Func(
        /// Always is `PyFunc`
        pyo3::Py<pyo3::PyAny>,
        /// Alias name
        sea_query::DynIden,
    ),
    TableName(PyTableName),
}

/// Distinct mode in [`PySelectStatement`]
#[derive(Debug, Clone)]
pub enum DistinctMode {
    None,
    Distinct,
    DistinctOn(Vec<PyColumnRef>),
}

/// Lock mode and options in [`PySelectStatement`]
#[derive(Debug)]
pub struct LockMode {
    pub r#type: sea_query::LockType,
    pub behavior: Option<sea_query::LockBehavior>,
    pub tables: Vec<PyTableName>,
}

/// Join mode and options in [`PySelectStatement`]
pub struct JoinMode {
    pub r#type: sea_query::JoinType,
    pub table: SelectReference,
    pub on: PyExpr,
    pub lateral: Option<sea_query::DynIden>,
}

crate::implement_pyclass! {
    /// Represents a column expression with an optional alias in a SELECT statement.
    ///
    /// Used to specify both the expression to select and an optional alias name
    /// for the result column.
    ///
    /// @signature (self, expr: object, alias: str | None = None, window: WindowStatement | str | None = None)
    immutable [subclass] PySelectExpr(SelectExprState) as "SelectExpr" {
        pub expr: PyExpr,
        pub alias: Option<String>,
        pub window: Option<SelectExprWindow>,
    }
}
crate::implement_pyclass! {
    /// Builds SELECT SQL statements with a fluent interface.
    ///
    /// Provides a chainable API for constructing SELECT queries with support for:
    /// - Column selection with expressions and aliases
    /// - Table and subquery sources
    /// - Filtering with WHERE and HAVING
    /// - Joins (inner, left, right, full, cross, lateral)
    /// - Grouping and aggregation
    /// - Ordering and pagination
    /// - Set operations (UNION, EXCEPT, INTERSECT)
    /// - Row locking for transactions
    /// - DISTINCT queries
    mutable [subclass, extends=PyQueryStatement] PySelectStatement(SelectStatementState) as "SelectStatement" {
        pub references: Vec<SelectReference>,
        pub columns: Vec<PySelectExpr>,
        pub r#where: Option<PyExpr>,
        pub groups: Vec<PyExpr>,

        /// Always is `Option<(_, PySelectStatement)>`
        pub unions: Vec<(sea_query::UnionType, pyo3::Py<pyo3::PyAny>)>,

        pub having: Option<PyExpr>,
        pub orders: Vec<PyOrdering>,
        pub distinct: DistinctMode,
        pub join: Vec<JoinMode>,
        pub lock: Option<LockMode>,
        pub limit: Option<u64>,
        pub offset: Option<u64>,

        /// Always is `Option<(_, PyWindowStatement)>`
        pub window: Option<(sea_query::DynIden, pyo3::Py<pyo3::PyAny>)>,

        // TODO
        // pub table_sample: Option<PyTableSample>,
        // pub index_hint: Option<PyIndexHint>,
    }
}

// #[inline]
// fn cast_into_select_expr<'a>(
//     value: pyo3::Bound<'a, pyo3::PyAny>,
// ) -> pyo3::PyResult<pyo3::Bound<'a, PySelectExpr>> {
// }

impl TryFrom<&pyo3::Bound<'_, pyo3::PyAny>> for SelectExprWindow {
    type Error = pyo3::PyErr;

    fn try_from(value: &pyo3::Bound<'_, pyo3::PyAny>) -> Result<Self, Self::Error> {
        unsafe {
            // Window statement
            if pyo3::ffi::PyObject_TypeCheck(value.as_ptr(), crate::typeref::WINDOW_STATEMENT_TYPE)
                == 1
            {
                Ok(Self::Window(value.clone().unbind()))
            }
            // String
            else if pyo3::ffi::PyUnicode_CheckExact(value.as_ptr()) == 1 {
                let window_name = value.extract::<String>().unwrap_unchecked();
                Ok(Self::Name(sea_query::Alias::new(window_name).into_iden()))
            }
            // Other types
            else {
                crate::new_error!(
                    PyTypeError,
                    "expected WindowStatement or str for SelectExpr window, got {}",
                    crate::internal::get_type_name(value.py(), value.as_ptr())
                )
            }
        }
    }
}

#[pyo3::pymethods]
impl PySelectExpr {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(
        args: &pyo3::Bound<'_, pyo3::types::PyTuple>,
        kwds: Option<&pyo3::Bound<'_, pyo3::types::PyDict>>,
    ) -> Self {
        Self::uninit()
    }

    #[pyo3(signature=(expr, alias=None, window=None))]
    fn __init__(
        &self,
        expr: &pyo3::Bound<'_, pyo3::PyAny>,
        alias: Option<String>,
        window: Option<&pyo3::Bound<'_, pyo3::PyAny>>,
    ) -> pyo3::PyResult<()> {
        let window = match window {
            Some(x) => Some(SelectExprWindow::try_from(x)?),
            None => None,
        };
        let expr = PyExpr::try_from(expr)?;

        let state = SelectExprState {
            expr,
            alias,
            window,
        };
        unsafe {
            self.0.set(state);
        }
        Ok(())
    }

    /// @signature (self) -> Expr
    #[getter]
    fn expr(&self) -> PyExpr {
        self.0.as_ref().expr.clone()
    }

    /// @signature (self) -> str | None
    #[getter]
    fn alias(&self) -> Option<String> {
        self.0.as_ref().alias.clone()
    }

    /// @signature (self) -> WindowStatement | str | None
    #[getter]
    fn window<'a>(&self, py: pyo3::Python<'a>) -> Option<pyo3::Bound<'a, pyo3::PyAny>> {
        use pyo3::IntoPyObjectExt;

        let inner = self.0.as_ref();

        match &inner.window {
            Some(ref select_window) => match select_window {
                SelectExprWindow::Name(name) => {
                    Some(name.to_string().into_bound_py_any(py).unwrap())
                }
                SelectExprWindow::Window(w) => Some(w.bind(py).clone()),
            },
            None => None,
        }
    }
}

    fn __repr__(&self) -> String {

    }
}
