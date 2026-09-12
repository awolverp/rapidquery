//! SQL expressions: engine enum plus a Serializer-style Python class tree.
//!
//! [`PyExpr`] holds [`ExprEnum`]. Concrete nodes (`Ident`, `Binary`, …) are
//! empty frozen subclasses. Shared methods live on the base.
//!
//! Suggested future methods (not implemented): `to_sql` / `prepare`; `eq` /
//! `ne` / `lt` / `le` / `gt` / `ge`; `add` / `sub` / `mul` / `div` / `mod`;
//! `and_` / `or_` / `xor` / `not_`; `bit_and` / `bit_or`; `like` / `ilike` /
//! `between` / `in_` / `is_null` / `is_distinct_from` / `cast`; `case`;
//! `custom`. Prefer named methods over v0 dunders that return `Expr`.

mod enum_;
mod func;
mod ident;
mod nodes;
mod value;

#[cfg(test)]
mod tests;

pub use enum_::{BinaryOp, DuplicateTreatment, ExprEnum, IsKind, LikeKind, UnaryOp};
pub use func::{Func, FuncArguments, PyFunc};
pub use ident::{Ident, IdentExtractor, IdentPart, ParseIdentError, PyAsterisk, PyIdent, PyUnquote};
pub use nodes::{
    PyBetween, PyBinary, PyCase, PyCast, PyCustom, PyInList, PyIs, PyIsDistinctFrom, PyLike,
    PyTupleExpr, PyUnary,
};
pub use value::{PyValue, Value};

use pyo3::types::PyTupleMethods;

use crate::internal::alias;
use crate::internal::utils;

/// Abstract SQL expression.
///
/// Concrete nodes are frozen subclasses (`Ident`, `Value`, `Func`,
/// `Binary`, …). This base class cannot be instantiated.
///
/// Future methods (`to_sql`, `eq`, `like`, `between`, `in_`, `is_null`,
/// `cast`, `case`, `custom`, arithmetic/logic) will live here so every
/// subclass inherits them. Prefer named methods over v0-style dunders that
/// return `Expr`.
#[rapidquery_macros::pyclass(
    "Expr",
    subclass,
    skip_from_py_object,
    module = "rapidquery._core.expr"
)]
#[derive(Debug, Clone)]
pub struct PyExpr(pub ExprEnum);

fn wrap<'py, T>(
    py: pyo3::Python<'py>,
    base: PyExpr,
) -> pyo3::PyResult<alias::BoundObject<'py>>
where
    T: pyo3::PyClass<BaseType = PyExpr> + Default,
{
    pyo3::Bound::new(
        py,
        pyo3::PyClassInitializer::from(base).add_subclass(T::default()),
    )
    .map(|x| x.into_any())
}

pub(crate) fn subclass<T: Default + pyo3::PyClass<BaseType = PyExpr>>(
    inner: ExprEnum,
) -> pyo3::PyClassInitializer<T> {
    pyo3::PyClassInitializer::from(PyExpr(inner)).add_subclass(T::default())
}

impl PyExpr {
    /// Wrap `self` as the matching Python subclass.
    pub fn into_subclasses<'py>(
        self,
        py: pyo3::Python<'py>,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        match &self.0 {
            ExprEnum::Ident(_) => wrap::<PyIdent>(py, self),
            ExprEnum::Value(_) => wrap::<PyValue>(py, self),
            ExprEnum::Tuple(_) => wrap::<PyTupleExpr>(py, self),
            ExprEnum::Unary { .. } => wrap::<PyUnary>(py, self),
            ExprEnum::Binary { .. } => wrap::<PyBinary>(py, self),
            ExprEnum::Function(_) => wrap::<PyFunc>(py, self),
            ExprEnum::Is { .. } => wrap::<PyIs>(py, self),
            ExprEnum::IsDistinctFrom { .. } => wrap::<PyIsDistinctFrom>(py, self),
            ExprEnum::InList { .. } => wrap::<PyInList>(py, self),
            ExprEnum::Between { .. } => wrap::<PyBetween>(py, self),
            ExprEnum::Like { .. } => wrap::<PyLike>(py, self),
            ExprEnum::Cast { .. } => wrap::<PyCast>(py, self),
            ExprEnum::Case { .. } => wrap::<PyCase>(py, self),
            ExprEnum::Custom { .. } => wrap::<PyCustom>(py, self),
        }
    }
}

#[pyo3::pymethods]
impl PyExpr {
    fn __repr__(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        self.0.python_repr(py)
    }

    fn __eq__(&self, py: pyo3::Python<'_>, other: &pyo3::Bound<'_, Self>) -> pyo3::PyResult<bool> {
        self.0.py_eq(py, &other.get().0)
    }

    fn __hash__(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<u64> {
        self.0.py_hash(py)
    }
}

/// Ident-priority extractor: a bare `str` becomes [`ExprEnum::Ident`].
pub struct ExprExtractor(pub ExprEnum);

/// Literal-priority extractor: a bare `str` becomes [`ExprEnum::Value`].
pub struct LiteralExprExtractor(pub ExprEnum);

fn extract_expr(
    obj: pyo3::prelude::Borrowed<'_, '_, pyo3::prelude::PyAny>,
    str_as_ident: bool,
) -> pyo3::PyResult<ExprEnum> {
    let ptr = obj.as_ptr();
    unsafe {
        if crate::typeref::check(ptr, crate::typeref::EXPR_TYPE) {
            let expr = obj.cast_unchecked::<PyExpr>();
            return Ok(expr.get().0.clone());
        }

        if crate::typeref::check_exact(ptr, crate::typeref::UNQUOTE_TYPE) {
            let unquote = obj.cast_unchecked::<PyUnquote>();
            let part = IdentPart::unquoted(unquote.get().0.as_str()).map_err(|err| {
                rapidquery_macros::pyerr!(PyValueError, "{}", err)
            })?;
            return Ok(ExprEnum::Ident(Ident::from_one(part)));
        }

        if is_asterisk(ptr) {
            return Ok(ExprEnum::Ident(Ident::from_one(IdentPart::Asterisk)));
        }

        if pyo3::ffi::PyUnicode_CheckExact(ptr) == 1 {
            let s = utils::py_str(obj.py(), ptr)?;
            if str_as_ident {
                let part = IdentPart::quoted(s).map_err(|err| {
                    rapidquery_macros::pyerr!(PyValueError, "{}", err)
                })?;
                return Ok(ExprEnum::Ident(Ident::from_one(part)));
            }
            return Ok(ExprEnum::Value(Value::infer_bound(obj.as_any().clone())));
        }

        if pyo3::ffi::PyTuple_CheckExact(ptr) == 1 {
            let tuple = obj.cast_unchecked::<pyo3::types::PyTuple>();
            let mut items = Vec::with_capacity(tuple.len());
            for item in tuple.iter() {
                items.push(extract_expr(item.as_borrowed(), str_as_ident)?);
            }
            return Ok(ExprEnum::Tuple(items));
        }
    }

    Ok(ExprEnum::Value(Value::infer_bound(obj.as_any().clone())))
}

/// `true` when `obj` is `Asterisk` or `type[Asterisk]`.
unsafe fn is_asterisk(obj: *mut pyo3::ffi::PyObject) -> bool {
    let type_ptr = if pyo3::ffi::PyType_CheckExact(obj) == 1 {
        obj.cast::<pyo3::ffi::PyTypeObject>()
    } else {
        pyo3::ffi::Py_TYPE(obj)
    };
    type_ptr == crate::typeref::ASTERISK_TYPE
}

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for ExprExtractor {
    type Error = pyo3::PyErr;

    #[inline]
    fn extract(
        obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>,
    ) -> Result<Self, Self::Error> {
        extract_expr(obj, true).map(Self)
    }
}

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for LiteralExprExtractor {
    type Error = pyo3::PyErr;

    #[inline]
    fn extract(
        obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>,
    ) -> Result<Self, Self::Error> {
        extract_expr(obj, false).map(Self)
    }
}

#[pyo3::pymodule(name = "expr", gil_used = false)]
pub mod expr_mod {
    #[pymodule_export]
    use super::{
        PyAsterisk, PyBetween, PyBinary, PyCase, PyCast, PyCustom, PyExpr, PyFunc, PyIdent,
        PyInList, PyIs, PyIsDistinctFrom, PyLike, PyTupleExpr, PyUnary, PyUnquote, PyValue,
    };
}
