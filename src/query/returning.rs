use pyo3::types::PyTupleMethods;

use crate::utils::ToSeaQuery;

#[derive(Debug, Clone)]
pub enum ReturningClauseState {
    All,
    Columns(Vec<sea_query::DynIden>),
}

implement_pyclass! {
    /// RETURNING clause.
    ///
    /// Works on PostgreSQL and SQLite>=3.35.0.
    ///
    /// Use `.all()` or `.columns()` classmethod to use this type.
    #[derive(Debug, Clone)]
    pub struct [] PyReturningClause as "ReturningClause" (pub ReturningClauseState);
}

impl ToSeaQuery<sea_query::ReturningClause> for ReturningClauseState {
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
impl PyReturningClause {
    /// Return all columns. Same as `self.columns("*")`.
    ///
    /// @signature (cls) -> typing.Self
    #[classmethod]
    fn all(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self(ReturningClauseState::All)
    }

    /// Specify columns you need to return.
    ///
    /// @signature (cls, *args: Column | ColumnRef | str) -> typing.Self
    #[classmethod]
    #[pyo3(signature=(*args))]
    fn columns(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        args: &pyo3::Bound<'_, pyo3::types::PyTuple>,
    ) -> pyo3::PyResult<Self> {
        let mut columns = Vec::with_capacity(args.len());

        for col in args.iter() {
            let column_ref = crate::common::PyColumnRef::try_from(&col)?;

            match column_ref.name {
                Some(x) => columns.push(x),
                None => {
                    return Ok(Self(ReturningClauseState::All));
                }
            }
        }

        Ok(Self(ReturningClauseState::Columns(columns)))
    }

    pub fn __repr__(&self) -> String {
        use std::io::Write;

        let mut s = Vec::<u8>::with_capacity(30);

        write!(s, "<Returning").unwrap();

        match &self.0 {
            ReturningClauseState::All => write!(s, " ALL").unwrap(),
            ReturningClauseState::Columns(x) => {
                write!(s, " [").unwrap();

                let n = x.len();
                for (index, ix) in x.iter().enumerate() {
                    if index + 1 == n {
                        write!(s, "{:?}", ix.to_string()).unwrap();
                    } else {
                        write!(s, "{:?}, ", ix.to_string()).unwrap();
                    }
                }
                write!(s, "]").unwrap();
            }
        }

        write!(s, ">").unwrap();
        unsafe { String::from_utf8_unchecked(s) }
    }
}
