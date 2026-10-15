//! SQL expressions.
//!
//! [`PyExpr`] holds [`ExprInner`]. Concrete nodes (`Ident`, `Binary`, …) are
//! empty frozen subclasses. Shared methods live on the base.

use crate::internal::alias;
use crate::internal::sql;
use crate::internal::utils;

use super::enum_;

/// Base of every SQL expression. Construct a subclass, not this class.
#[rapidquery_macros::pyclass("rapidquery.expr.Expr", subclass, skip_from_py_object)]
#[derive(Debug, Clone)]
pub struct PyExpr(pub enum_::ExprInner);

#[inline(always)]
fn wrap<'py, T>(py: pyo3::Python<'py>, base: PyExpr) -> pyo3::PyResult<alias::BoundObject<'py>>
where
    T: pyo3::PyClass<BaseType = PyExpr> + Default,
{
    pyo3::Bound::new(py, pyo3::PyClassInitializer::from(base).add_subclass(T::default())).map(|x| x.into_any())
}

/// Wrap `expr` as the matching Python subclass.
///
/// Shorthand for `PyExpr(expr).into_subclasses(py)`
#[inline(always)]
pub(super) fn into_subclasses<'py>(
    py: pyo3::Python<'py>,
    expr: enum_::ExprInner,
) -> pyo3::PyResult<alias::BoundObject<'py>> {
    PyExpr(expr).into_subclasses(py)
}

impl PyExpr {
    /// Wrap `self` as the matching Python subclass.
    #[inline]
    pub fn into_subclasses<'py>(self, py: pyo3::Python<'py>) -> pyo3::PyResult<alias::BoundObject<'py>> {
        match &self.0 {
            enum_::ExprInner::Ident(_) => wrap::<super::ident::PyIdent>(py, self),
            enum_::ExprInner::Value(_) => wrap::<super::value::PyValue>(py, self),
            enum_::ExprInner::Tuple(_) => wrap::<super::nodes::PyTupleExpr>(py, self),
            enum_::ExprInner::Unary(_) => wrap::<super::nodes::PyUnaryExpr>(py, self),
            enum_::ExprInner::Binary(_) => wrap::<super::nodes::PyBinaryExpr>(py, self),
            enum_::ExprInner::Function(_) => wrap::<super::func::PyFunction>(py, self),
            enum_::ExprInner::Is(_) => wrap::<super::nodes::PyIsExpr>(py, self),
            enum_::ExprInner::IsDistinctFrom(_) => wrap::<super::nodes::PyIsDistinctFromExpr>(py, self),
            enum_::ExprInner::InList(_) => wrap::<super::nodes::PyInListExpr>(py, self),
            enum_::ExprInner::Between(_) => wrap::<super::nodes::PyBetweenExpr>(py, self),
            enum_::ExprInner::Like(_) => wrap::<super::nodes::PyLikeExpr>(py, self),
            enum_::ExprInner::SimilarTo(_) => wrap::<super::nodes::PySimilarToExpr>(py, self),
            enum_::ExprInner::RLike(_) => wrap::<super::nodes::PyRLikeExpr>(py, self),
            enum_::ExprInner::Cast(_) => wrap::<super::nodes::PyCastExpr>(py, self),
            enum_::ExprInner::Trim(_) => wrap::<super::nodes::PyTrimExpr>(py, self),
            enum_::ExprInner::Collate(_) => wrap::<super::nodes::PyCollateExpr>(py, self),
            enum_::ExprInner::Case(_) => wrap::<super::nodes::PyCaseExpr>(py, self),
            enum_::ExprInner::Custom(_) => wrap::<super::nodes::PyCustomExpr>(py, self),
        }
    }
}

/// `None`, exact `bool`, or exact `str` → an `IS` expression.
fn is_from_kind<'py>(expr: enum_::ExprInner, kind: &alias::BoundObject<'py>, negated: bool) -> pyo3::PyResult<PyExpr> {
    let ptr = kind.as_ptr();

    // SAFETY: `ptr` is a live borrowed object and this runs with the GIL held.
    // The checks only read the type tag. The string borrow lasts until `new_base` returns.
    unsafe {
        if pyo3::ffi::Py_IsNone(ptr) == 1 {
            return super::nodes::PyIsExpr::new_base(expr, "NULL", negated);
        }

        if pyo3::ffi::PyBool_Check(ptr) == 1 {
            let name = if ptr == pyo3::ffi::Py_True() { "TRUE" } else { "FALSE" };
            return super::nodes::PyIsExpr::new_base(expr, name, negated);
        }

        if pyo3::ffi::PyUnicode_CheckExact(ptr) == 1 {
            let text = utils::borrow_py_str_buffer(kind.py(), ptr)?;
            return super::nodes::PyIsExpr::new_base(expr, text, negated);
        }

        rapidquery_macros::pyerr!(
            Err(PyTypeError),
            "expected None, bool, or str, got {}",
            utils::get_type_name(kind.py(), ptr)
        )
    }
}

#[pyo3::pymethods]
impl PyExpr {
    // ===== Binary Expressions =====

    /// e.g. `a + b`
    fn __add__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(self.0.clone(), enum_::BinaryOp::Add, other.0).into_subclasses(py)
    }

    /// e.g. `a - b`
    fn __sub__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(self.0.clone(), enum_::BinaryOp::Sub, other.0).into_subclasses(py)
    }

    /// e.g. `a * b`
    fn __mul__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(self.0.clone(), enum_::BinaryOp::Mul, other.0).into_subclasses(py)
    }

    /// e.g. `a / b`
    fn __truediv__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(self.0.clone(), enum_::BinaryOp::Div, other.0).into_subclasses(py)
    }

    /// e.g. `a % b`
    fn __mod__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(self.0.clone(), enum_::BinaryOp::Mod, other.0).into_subclasses(py)
    }

    /// e.g. `a = b`
    fn __eq__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(self.0.clone(), enum_::BinaryOp::Eq, other.0).into_subclasses(py)
    }

    /// e.g. `a <> b`
    fn __ne__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(self.0.clone(), enum_::BinaryOp::NotEq, other.0).into_subclasses(py)
    }

    /// e.g. `a < b`
    fn __lt__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(self.0.clone(), enum_::BinaryOp::Lt, other.0).into_subclasses(py)
    }

    /// e.g. `a <= b`
    fn __le__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(self.0.clone(), enum_::BinaryOp::LtEq, other.0).into_subclasses(py)
    }

    /// e.g. `a > b`
    fn __gt__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(self.0.clone(), enum_::BinaryOp::Gt, other.0).into_subclasses(py)
    }

    /// e.g. `a >= b`
    fn __ge__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(self.0.clone(), enum_::BinaryOp::GtEq, other.0).into_subclasses(py)
    }

    /// e.g. `a AND b`
    fn __and__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(self.0.clone(), enum_::BinaryOp::And, other.0).into_subclasses(py)
    }

    /// e.g. `a OR b`
    fn __or__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(self.0.clone(), enum_::BinaryOp::Or, other.0).into_subclasses(py)
    }

    /// e.g. `a & b`
    fn bit_and<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(self.0.clone(), enum_::BinaryOp::BitAnd, other.0).into_subclasses(py)
    }

    /// e.g. `a | b`
    fn bit_or<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(self.0.clone(), enum_::BinaryOp::BitOr, other.0).into_subclasses(py)
    }

    /// e.g. `a ^ b`
    fn bit_xor<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(self.0.clone(), enum_::BinaryOp::BitXor, other.0).into_subclasses(py)
    }

    /// e.g. `a << b`
    fn __lshift__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(self.0.clone(), enum_::BinaryOp::LShift, other.0).into_subclasses(py)
    }

    /// e.g. `a >> b`
    fn __rshift__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(self.0.clone(), enum_::BinaryOp::RShift, other.0).into_subclasses(py)
    }

    /// e.g. `a || b`
    fn concat<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(self.0.clone(), enum_::BinaryOp::StringConcat, other.0).into_subclasses(py)
    }

    /// e.g. `b + a`
    fn __radd__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(other.0, enum_::BinaryOp::Add, self.0.clone()).into_subclasses(py)
    }

    /// e.g. `b - a`
    fn __rsub__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(other.0, enum_::BinaryOp::Sub, self.0.clone()).into_subclasses(py)
    }

    /// e.g. `b * a`
    fn __rmul__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(other.0, enum_::BinaryOp::Mul, self.0.clone()).into_subclasses(py)
    }

    /// e.g. `b / a`
    fn __rtruediv__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(other.0, enum_::BinaryOp::Div, self.0.clone()).into_subclasses(py)
    }

    /// e.g. `b % a`
    fn __rmod__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(other.0, enum_::BinaryOp::Mod, self.0.clone()).into_subclasses(py)
    }

    /// e.g. `b AND a`
    fn __rand__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(other.0, enum_::BinaryOp::And, self.0.clone()).into_subclasses(py)
    }

    /// e.g. `b OR a`
    fn __ror__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(other.0, enum_::BinaryOp::Or, self.0.clone()).into_subclasses(py)
    }

    /// e.g. `b << a`
    fn __rlshift__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(other.0, enum_::BinaryOp::LShift, self.0.clone()).into_subclasses(py)
    }

    /// e.g. `b >> a`
    fn __rrshift__<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBinaryExpr::new_base(other.0, enum_::BinaryOp::RShift, self.0.clone()).into_subclasses(py)
    }

    // ===== Unary =====

    /// e.g. `-a`
    fn __neg__<'py>(&self, py: pyo3::Python<'py>) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyUnaryExpr::new_base(enum_::UnaryOp::Minus, self.0.clone()).into_subclasses(py)
    }

    /// e.g. `+a`
    fn __pos__<'py>(&self, py: pyo3::Python<'py>) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyUnaryExpr::new_base(enum_::UnaryOp::Plus, self.0.clone()).into_subclasses(py)
    }

    /// e.g. `NOT a`
    fn not_<'py>(&self, py: pyo3::Python<'py>) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyUnaryExpr::new_base(enum_::UnaryOp::Not, self.0.clone()).into_subclasses(py)
    }

    /// e.g. `~a`
    fn bit_not<'py>(&self, py: pyo3::Python<'py>) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyUnaryExpr::new_base(enum_::UnaryOp::BitNot, self.0.clone()).into_subclasses(py)
    }

    // ===== Between ======

    /// e.g. `a BETWEEN b AND c`
    fn between<'py>(
        &self,
        py: pyo3::Python<'py>,
        low: super::extractors::LiteralExprExtractor,
        high: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBetweenExpr::new_base(self.0.clone(), low.0, high.0, false).into_subclasses(py)
    }

    /// e.g. `a NOT BETWEEN b AND c`
    fn not_between<'py>(
        &self,
        py: pyo3::Python<'py>,
        low: super::extractors::LiteralExprExtractor,
        high: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyBetweenExpr::new_base(self.0.clone(), low.0, high.0, true).into_subclasses(py)
    }

    // ===== Is =====

    /// e.g. `a IS [kind]`
    fn is_<'py>(
        &self,
        py: pyo3::Python<'py>,
        kind: alias::BoundObject<'py>,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        is_from_kind(self.0.clone(), &kind, false).and_then(|x| x.into_subclasses(py))
    }

    /// e.g. `a IS NOT [kind]`
    fn is_not<'py>(
        &self,
        py: pyo3::Python<'py>,
        kind: alias::BoundObject<'py>,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        is_from_kind(self.0.clone(), &kind, true).and_then(|x| x.into_subclasses(py))
    }

    // ===== IsDistinctFrom =====

    /// e.g. `a IS DISTINCT FROM b`
    fn is_distinct_from<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyIsDistinctFromExpr::new_base(self.0.clone(), other.0, false).into_subclasses(py)
    }

    /// e.g. `a IS NOT DISTINCT FROM b`
    fn is_not_distinct_from<'py>(
        &self,
        py: pyo3::Python<'py>,
        other: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyIsDistinctFromExpr::new_base(self.0.clone(), other.0, true).into_subclasses(py)
    }

    // ===== InList =====

    /// e.g. `a IN (b, c, ...)`
    fn in_<'py>(&self, list: alias::BoundObject<'py>) -> pyo3::PyResult<alias::BoundObject<'py>> {
        let py = list.py();

        super::nodes::PyInListExpr::new_base(self.0.clone(), list, false).and_then(|x| x.into_subclasses(py))
    }

    /// e.g. `a NOT IN (b, c, ...)`
    fn not_in<'py>(&self, list: alias::BoundObject<'py>) -> pyo3::PyResult<alias::BoundObject<'py>> {
        let py = list.py();

        super::nodes::PyInListExpr::new_base(self.0.clone(), list, true).and_then(|x| x.into_subclasses(py))
    }

    // // ===== Like =====

    /// e.g. `a LIKE 'a%'`
    #[pyo3(signature = (pattern, /, *, escape = None))]
    fn like<'py>(
        &self,
        pattern: alias::BoundObject<'py>,
        escape: Option<char>,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        let py = pattern.py();

        super::nodes::PyLikeExpr::new_base(self.0.clone(), pattern, false, true, escape)
            .and_then(|x| x.into_subclasses(py))
    }

    /// e.g. `a NOT LIKE 'a%'`
    #[pyo3(signature = (pattern, /, *, escape = None))]
    fn not_like<'py>(
        &self,
        pattern: alias::BoundObject<'py>,
        escape: Option<char>,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        let py = pattern.py();

        super::nodes::PyLikeExpr::new_base(self.0.clone(), pattern, true, true, escape)
            .and_then(|x| x.into_subclasses(py))
    }

    /// e.g. `a ILIKE 'a%'`
    #[pyo3(signature = (pattern, /, *, escape = None))]
    fn ilike<'py>(
        &self,
        pattern: alias::BoundObject<'py>,
        escape: Option<char>,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        let py = pattern.py();

        super::nodes::PyLikeExpr::new_base(self.0.clone(), pattern, false, false, escape)
            .and_then(|x| x.into_subclasses(py))
    }

    /// e.g. `a NOT ILIKE 'a%'`
    #[pyo3(signature = (pattern, /, *, escape = None))]
    fn not_ilike<'py>(
        &self,
        pattern: alias::BoundObject<'py>,
        escape: Option<char>,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        let py = pattern.py();

        super::nodes::PyLikeExpr::new_base(self.0.clone(), pattern, true, false, escape)
            .and_then(|x| x.into_subclasses(py))
    }

    // ===== SimilarTo / RLike =====

    /// e.g. `a SIMILAR TO b`
    fn similar_to<'py>(
        &self,
        py: pyo3::Python<'py>,
        pattern: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PySimilarToExpr::new_base(self.0.clone(), pattern.0, false).into_subclasses(py)
    }

    /// e.g. `a NOT SIMILAR TO b`
    fn not_similar_to<'py>(
        &self,
        py: pyo3::Python<'py>,
        pattern: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PySimilarToExpr::new_base(self.0.clone(), pattern.0, true).into_subclasses(py)
    }

    /// e.g. `a RLIKE b`
    fn rlike<'py>(
        &self,
        py: pyo3::Python<'py>,
        pattern: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyRLikeExpr::new_base(self.0.clone(), pattern.0, false).into_subclasses(py)
    }

    /// e.g. `a NOT RLIKE b`
    fn not_rlike<'py>(
        &self,
        py: pyo3::Python<'py>,
        pattern: super::extractors::LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyRLikeExpr::new_base(self.0.clone(), pattern.0, true).into_subclasses(py)
    }

    // ===== Cast =====

    /// e.g. `CAST(a AS int)`
    fn cast_as<'py>(&self, py: pyo3::Python<'py>, data_type_name: &str) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyCastExpr::new_base(self.0.clone(), data_type_name, "CAST").and_then(|x| x.into_subclasses(py))
    }

    /// e.g. `TRY_CAST(a AS int)`
    fn try_cast_as<'py>(&self, py: pyo3::Python<'py>, data_type_name: &str) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyCastExpr::new_base(self.0.clone(), data_type_name, "TRY_CAST")
            .and_then(|x| x.into_subclasses(py))
    }

    // ===== Collate / Trim =====

    /// e.g. `a COLLATE coll`
    fn collate<'py>(
        &self,
        py: pyo3::Python<'py>,
        collation: super::extractors::IdentExtractor,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyCollateExpr::new_base(self.0.clone(), collation.0).into_subclasses(py)
    }

    /// e.g. `TRIM(a)`
    #[pyo3(signature = (*, r#where = "BOTH", characters = None))]
    fn trim<'py>(
        &self,
        py: pyo3::Python<'py>,
        r#where: &str,
        characters: Option<super::extractors::LiteralExprExtractor>,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        super::nodes::PyTrimExpr::new_base(self.0.clone(), r#where, characters.map(|x| x.0))
            .and_then(|x| x.into_subclasses(py))
    }

    /// Render this expression as a parameterized statement.
    fn prepare<'py>(
        slf: pyo3::PyRef<'py, Self>,
        py: pyo3::Python<'py>,
        dialect: crate::dialect::PyDialect,
    ) -> pyo3::PyResult<(String, alias::BoundObject<'py>)> {
        let (sql, params) = sql::prepare(py, &slf.0, &dialect)?;

        let params = unsafe {
            utils::iterator_to_tuple(
                slf.py(),
                params.len() as isize,
                params.into_iter().map(|x| Ok(x.into_ptr())),
            )?
        };
        Ok((sql, params))
    }

    /// Render this expression as raw SQL (values inlined as literals).
    fn to_sql(
        slf: pyo3::PyRef<'_, Self>,
        py: pyo3::Python<'_>,
        dialect: crate::dialect::PyDialect,
    ) -> pyo3::PyResult<String> {
        sql::write_sql(py, &slf.0, &dialect).map_err(Into::into)
    }

    fn __repr__(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        Ok(self.0.python_repr(py)?.into_string())
    }
}
