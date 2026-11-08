use std::str::FromStr;
use std::sync;

use crate::exc;
use crate::expr::enum_::ExprInner;
use crate::expr::extractors::NameExprExtractor;
use crate::internal::alias;
use crate::internal::repr;

/// Represents `ASC` and `DESC` keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, strum::EnumString, strum::IntoStaticStr)]
#[strum(ascii_case_insensitive)]
pub enum IndexOrder {
    /// Emits `ASC`.
    #[strum(serialize = "ASC")]
    Asc,
    /// Emits `DESC`.
    #[strum(serialize = "DESC")]
    Desc,
}

/// IndexColumn inner fields, will wrapped by [`std::sync::Arc`] to have cheap clones
#[derive(Debug, Clone)]
pub struct IndexColumnInner {
    /// Index column expression.
    pub column: ExprInner,

    /// Index column order.
    pub order: IndexOrder,
}

impl IndexColumnInner {
    /// Returns Python `__repr__` result
    #[inline]
    pub fn python_repr(&self, py: pyo3::Python<'_>) -> exc::Result<repr::ReprFormatter> {
        let mut fmt = repr::ReprFormatter::new_call("IndexColumn");

        fmt.positional(self.column.python_repr(py)?);
        if matches!(self.order, IndexOrder::Desc) {
            fmt.keyword("order", "DESC");
        }

        Ok(fmt)
    }
}

/// One key of a `CREATE INDEX` column list.
#[rapidquery_macros::pyclass("rapidquery.ddl.IndexColumn")]
#[derive(Debug, Clone)]
pub struct PyIndexColumn(pub sync::Arc<IndexColumnInner>);

impl std::ops::Deref for PyIndexColumn {
    type Target = IndexColumnInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[pyo3::pymethods]
impl PyIndexColumn {
    #[new]
    #[pyo3(signature = (column, *, order = "ASC"))]
    fn new(column: NameExprExtractor, order: &str) -> exc::Result<Self> {
        let order = IndexOrder::from_str(order)
            .map_err(|_| exc::Error::invalid("order", format!("expected 'ASC' or 'DESC', received {order}")))?;

        let inner = IndexColumnInner {
            column: column.0,
            order,
        };

        Ok(Self(sync::Arc::new(inner)))
    }

    #[getter]
    fn column<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<alias::BoundObject<'a>> {
        crate::expr::base::into_subclasses(py, self.column.clone())
    }

    #[getter]
    fn order(&self) -> &'static str {
        self.order.into()
    }

    fn __repr__(&self, py: pyo3::Python<'_>) -> exc::Result<String> {
        self.python_repr(py).map(repr::ReprFormatter::into_string)
    }
}
