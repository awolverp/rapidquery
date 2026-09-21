//! SQL expressions.
//!
//! [`PyExpr`] holds [`ExprEnum`]. Concrete nodes (`Ident`, `Binary`, …) are
//! empty frozen subclasses. Shared methods live on the base.
//!
// TODO: `to_sql` / `prepare`; `eq` / `ne` / `lt` / `le` / `gt` / `ge`; `add` / `sub` / `mul` / `div` / `mod`;
// `and_` / `or_` / `xor` / `not_`; `bit_and` / `bit_or`; `like` / `ilike` /
// `between` / `in_` / `is_null` / `is_distinct_from` / `cast`; `case`;
// `custom`. Prefer named methods over v0 dunders that return `Expr`.

use crate::internal::alias;
use crate::sql;

use super::enum_::ExprEnum;

/// Abstract SQL expression.
///
/// Concrete nodes are frozen subclasses (`Ident`, `Value`, `Func`,
/// `Binary`, …). This base class cannot be instantiated.
#[rapidquery_macros::pyclass(
    "Expr",
    subclass,
    skip_from_py_object,
    module = "rapidquery._core.expr"
)]
#[derive(Debug, Clone)]
pub struct PyExpr(pub ExprEnum);

#[inline(always)]
fn wrap<'py, T>(py: pyo3::Python<'py>, base: PyExpr) -> pyo3::PyResult<alias::BoundObject<'py>>
where
    T: pyo3::PyClass<BaseType = PyExpr> + Default,
{
    pyo3::Bound::new(
        py,
        pyo3::PyClassInitializer::from(base).add_subclass(T::default()),
    )
    .map(|x| x.into_any())
}

/// Wrap `expr` as the matching Python subclass.
///
/// Shorthand for `PyExpr(expr).into_subclasses(py)`
#[inline]
pub(super) fn into_subclasses<'py>(
    py: pyo3::Python<'py>,
    expr: ExprEnum,
) -> pyo3::PyResult<alias::BoundObject<'py>> {
    PyExpr(expr).into_subclasses(py)
}

impl PyExpr {
    /// Wrap `self` as the matching Python subclass.
    #[inline]
    pub fn into_subclasses<'py>(
        self,
        py: pyo3::Python<'py>,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        match &self.0 {
            ExprEnum::Ident(_) => wrap::<super::ident::PyIdent>(py, self),
            ExprEnum::Value(_) => wrap::<super::value::PyValue>(py, self),
            ExprEnum::Tuple(_) => wrap::<super::nodes::PyTupleExpr>(py, self),
            ExprEnum::Unary(_) => wrap::<super::nodes::PyUnaryExpr>(py, self),
            ExprEnum::Binary(_) => wrap::<super::nodes::PyBinaryExpr>(py, self),
            ExprEnum::Function(_) => wrap::<super::func::PyFunction>(py, self),
            ExprEnum::Is(_) => wrap::<super::nodes::PyIsExpr>(py, self),
            ExprEnum::IsNormalized(_) => wrap::<super::nodes::PyIsNormalizedExpr>(py, self),
            ExprEnum::IsDistinctFrom(_) => wrap::<super::nodes::PyIsDistinctFromExpr>(py, self),
            ExprEnum::InList(_) => wrap::<super::nodes::PyInListExpr>(py, self),
            ExprEnum::Between(_) => wrap::<super::nodes::PyBetweenExpr>(py, self),
            ExprEnum::Like(_) => wrap::<super::nodes::PyLikeExpr>(py, self),
            ExprEnum::SimilarTo(_) => wrap::<super::nodes::PySimilarToExpr>(py, self),
            ExprEnum::RLike(_) => wrap::<super::nodes::PyRLikeExpr>(py, self),
            ExprEnum::Cast(_) => wrap::<super::nodes::PyCastExpr>(py, self),
            ExprEnum::Case(_) => wrap::<super::nodes::PyCaseExpr>(py, self),
            ExprEnum::Custom(_) => wrap::<super::nodes::PyCustomExpr>(py, self),
        }
    }
}

#[pyo3::pymethods]
impl PyExpr {
    /// Render this expression as a parameterized statement.
    fn prepare(
        slf: pyo3::PyRef<'_, Self>,
        py: pyo3::Python<'_>,
        dialect: crate::dialect::py::PySqlDialect,
    ) -> pyo3::PyResult<(String, sql::PrepareParams)> {
        sql::prepare(py, &slf.0, &dialect.0).map_err(Into::into)
    }

    /// Render this expression as raw SQL (values inlined as literals).
    fn to_sql(
        slf: pyo3::PyRef<'_, Self>,
        py: pyo3::Python<'_>,
        dialect: crate::dialect::py::PySqlDialect,
    ) -> pyo3::PyResult<String> {
        sql::write_sql(py, &slf.0, &dialect.0).map_err(Into::into)
    }

    fn __repr__(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        self.0.python_repr(py)
    }
}
