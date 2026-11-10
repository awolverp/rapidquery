use crate::ddl::common;
use crate::exc;

use super::column::PyIndexColumn;

/// IndexColumn extracted from Python.
///
/// Accepts `str` (becomes an identifier), `Expr` and exact `IndexColumn`.
pub struct IndexColumnExtractor<'py>(pub pyo3::Bound<'py, PyIndexColumn>);

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for IndexColumnExtractor<'py> {
    type Error = exc::Error;

    #[inline]
    fn extract(obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>) -> Result<Self, Self::Error> {
        unsafe {
            if crate::typeref::check_exact(obj.as_ptr(), crate::typeref::DDL_INDEX_COLUMN_TYPE) {
                let obj = obj.cast_unchecked::<PyIndexColumn>();
                return Ok(Self(obj.to_owned()));
            }
        }

        let expr = crate::expr::extractors::NameExprExtractor::extract(obj)?;
        let inner = PyIndexColumn {
            column: expr.0,
            order: common::Order::Asc,
        };

        pyo3::Bound::new(obj.py(), inner).map(Self).map_err(Into::into)
    }
}
