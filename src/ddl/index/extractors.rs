use std::sync;

use crate::exc;

use super::column::IndexColumnInner;
use super::column::IndexOrder;
use super::column::PyIndexColumn;

/// IndexColumn extracted from Python.
///
/// Accepts `str` (becomes an identifier), `Expr` and exact `IndexColumn`.
pub struct IndexColumnExtractor(pub sync::Arc<IndexColumnInner>);

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for IndexColumnExtractor {
    type Error = exc::Error;

    #[inline]
    fn extract(obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>) -> Result<Self, Self::Error> {
        unsafe {
            if crate::typeref::check_exact(obj.as_ptr(), crate::typeref::DDL_INDEX_COLUMN_TYPE) {
                let obj = obj.cast_unchecked::<PyIndexColumn>();
                return Ok(Self(obj.get().0.clone()));
            }
        }

        let expr = crate::expr::extractors::NameExprExtractor::extract(obj)?;
        let inner = IndexColumnInner {
            column: expr.0,
            order: IndexOrder::Asc,
        };

        Ok(Self(sync::Arc::new(inner)))
    }
}
