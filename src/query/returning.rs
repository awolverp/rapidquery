use pyo3::types::PyTupleMethods;
use sea_query::IntoColumnRef;

use crate::common::column_ref::PyColumnRef;
use crate::common::expression::PyExpr;
use crate::internal::repr::ReprFormatter;
use crate::internal::{BoundArgs, ToSeaQuery};

#[derive(Debug, Clone)]
pub enum ReturningState {
    All,
    Exprs(Vec<PyExpr>),
}

crate::implement_pyclass! {
    // NOTE: It's a very simple clause, so I think it's OK to be a final type.

    /// RETURNING clause.
    ///
    /// Works on PostgreSQL and SQLite>=3.35.0.
    #[derive(Debug, Clone)]
    [] PyReturning as "Returning" (pub ReturningState);
}

impl ToSeaQuery<sea_query::ReturningClause> for ReturningState {
    #[cfg_attr(feature = "optimize", optimize(speed))]
    fn to_sea_query<'a>(&self, _py: pyo3::Python<'a>) -> sea_query::ReturningClause {
        match self {
            Self::All => sea_query::ReturningClause::All,
            Self::Exprs(x) => {
                sea_query::ReturningClause::Exprs(x.iter().map(|x| x.0.clone()).collect())
            }
        }
    }
}

#[pyo3::pymethods]
impl PyReturning {
    /// Specify columns you need to return
    #[new]
    #[pyo3(signature=(*args))]
    pub fn __new__(args: BoundArgs<'_>) -> pyo3::PyResult<Self> {
        let mut columns = Vec::with_capacity(args.len());

        for col in args.iter() {
            unsafe {
                if pyo3::ffi::Py_TYPE(col.as_ptr()) == crate::typeref::EXPR_TYPE {
                    let value = col.cast_into_unchecked::<PyExpr>();

                    columns.push(value.get().clone());
                    continue;
                }
            }

            let column_ref = PyColumnRef::try_from(&col)?;
            if column_ref.name.is_none() {
                return Ok(Self(ReturningState::All));
            }

            columns.push(PyExpr(column_ref.into_column_ref().into()));
        }

        Ok(Self(ReturningState::Exprs(columns)))
    }

    /// Return all columns. Same as `self.columns("*")`.
    #[classmethod]
    fn all(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self(ReturningState::All)
    }

    fn __copy__(&self) -> Self {
        self.clone()
    }

    pub fn __repr__(&self) -> String {
        let mut fmt = ReprFormatter::new("Returning");

        match &self.0 {
            ReturningState::All => {
                fmt.pair("", "*");
            }
            ReturningState::Exprs(x) => {
                fmt.vec("", false)
                    .display_iter(x.iter().map(|x| x.__repr__()))
                    .finish(&mut fmt);
            }
        }

        fmt.finish()
    }
}
