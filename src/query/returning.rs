use pyo3::types::PyTupleMethods;

use crate::common::column_ref::PyColumnRef;
use crate::internal::repr::ReprFormatter;
use crate::internal::{BoundArgs, ToSeaQuery};

#[derive(Debug, Clone)]
pub enum ReturningState {
    All,
    Columns(Vec<sea_query::DynIden>),
}

crate::implement_pyclass! {
    // NOTE: It's a very simple clause, so I think it's OK to be a final type.

    /// RETURNING clause.
    ///
    /// Works on PostgreSQL and SQLite>=3.35.0.
    ///
    /// Use `.all()` or `.columns()` classmethod to use this type.
    #[derive(Debug, Clone)]
    [] PyReturning as "Returning" (pub ReturningState);
}

impl ToSeaQuery<sea_query::ReturningClause> for ReturningState {
    #[cfg_attr(feature = "optimize", optimize(speed))]
    fn to_sea_query<'a>(&self, _py: pyo3::Python<'a>) -> sea_query::ReturningClause {
        match self {
            Self::All => sea_query::ReturningClause::All,
            Self::Columns(x) => sea_query::ReturningClause::Columns(
                x.iter()
                    .map(|x| sea_query::ColumnRef::Column(x.clone()))
                    .collect(),
            ),
        }
    }
}

#[pyo3::pymethods]
impl PyReturning {
    /// Specify columns you need to return.
    ///
    /// @signature (cls, *args: Column | ColumnRef | str) -> typing.Self
    #[new]
    #[pyo3(signature=(*args))]
    pub fn __new__(args: BoundArgs<'_>) -> pyo3::PyResult<Self> {
        let mut columns = Vec::with_capacity(args.len());

        for col in args.iter() {
            let column_ref = PyColumnRef::try_from(&col)?;

            match column_ref.name {
                Some(x) => columns.push(x),
                None => {
                    return Ok(Self(ReturningState::All));
                }
            }
        }

        Ok(Self(ReturningState::Columns(columns)))
    }

    /// Return all columns. Same as `self.columns("*")`.
    ///
    /// @signature (cls) -> typing.Self
    #[classmethod]
    fn all(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self(ReturningState::All)
    }

    pub fn __repr__(&self) -> String {
        let mut fmt = ReprFormatter::new("Returning");

        match &self.0 {
            ReturningState::All => {
                fmt.pair("", "*");
            }
            ReturningState::Columns(x) => {
                fmt.vec("", false)
                    .quote_iter(x.iter().map(|x| x.to_string()))
                    .finish(&mut fmt);
            }
        }

        fmt.finish()
    }
}
