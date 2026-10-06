use std::str::FromStr;

use pyo3::types::PyAnyMethods;
use pyo3::types::PyListMethods;
use pyo3::types::PyTupleMethods;

use super::enum_::ExprInner;
use super::enum_::Ident;
use super::enum_::IdentPart;
use super::enum_::LikePattern;
use super::enum_::Tuple;
use crate::internal::utils;

/// `true` when `obj` is `Asterisk` or `type[Asterisk]`.
#[inline]
unsafe fn is_asterisk(obj: *mut pyo3::ffi::PyObject) -> bool {
    let type_ptr = if pyo3::ffi::PyType_CheckExact(obj) == 1 {
        obj.cast::<pyo3::ffi::PyTypeObject>()
    } else {
        pyo3::ffi::Py_TYPE(obj)
    };
    type_ptr == crate::typeref::ASTERISK_TYPE
}

/// Expression extracted from Python. A bare `str` becomes a value, not a name.
///
/// Use this for arguments where a string is data. Identifiers still pass through
/// as identifiers. The wrapped node is [`ExprInner`].
///
/// ```
/// use std::str::FromStr;
///
/// use rapidquery::expr::enum_::ExprInner;
/// use rapidquery::expr::enum_::Ident;
/// use rapidquery::expr::extractors::LiteralExprExtractor;
///
/// let extracted = LiteralExprExtractor(ExprInner::Ident(Ident::from_str("id").unwrap()));
/// assert!(matches!(extracted.0, ExprInner::Ident(_)));
/// ```
pub struct LiteralExprExtractor(pub ExprInner);

/// Native [`Ident`] extracted from a Python identifier argument.
///
/// Accepts `Ident`, one quoted `str` (including `"*"`), `Unquote`, `Asterisk`,
/// and `type[Asterisk]`. Sequences are rejected.
///
/// ```
/// use std::str::FromStr;
///
/// use rapidquery::expr::enum_::Ident;
/// use rapidquery::expr::extractors::IdentExtractor;
///
/// let extracted = IdentExtractor(Ident::from_str("users.id").unwrap());
/// assert_eq!(extracted.0.len(), 2);
/// ```
pub struct IdentExtractor(pub Ident);

fn extract_expr(
    obj: pyo3::prelude::Borrowed<'_, '_, pyo3::prelude::PyAny>,
    str_as_ident: bool,
) -> pyo3::PyResult<ExprInner> {
    let ptr = obj.as_ptr();
    unsafe {
        if crate::typeref::check(ptr, crate::typeref::EXPR_TYPE) {
            let expr = obj.cast_unchecked::<super::base::PyExpr>();
            return Ok(expr.get().0.clone());
        }

        if crate::typeref::check_exact(ptr, crate::typeref::UNQUOTE_TYPE) {
            let unquote = obj.cast_unchecked::<super::ident::PyUnquote>();
            let part = IdentPart::unquoted(unquote.get().0.as_str())
                .map_err(|err| rapidquery_macros::pyerr!(PyValueError, "{}", err))?;

            return Ok(ExprInner::Ident(Ident::from_one(part)));
        }

        if is_asterisk(ptr) {
            return Ok(ExprInner::Ident(Ident::from_one(IdentPart::Asterisk)));
        }

        if pyo3::ffi::PyUnicode_CheckExact(ptr) == 1 {
            let s = utils::borrow_py_str_buffer(obj.py(), ptr)?;

            // a bare `str` becomes [`ExprInner::Ident`] if `str_as_ident`
            if str_as_ident {
                let inner = Ident::from_str(s).map_err(|err| rapidquery_macros::pyerr!(PyValueError, "{}", err))?;

                return Ok(ExprInner::Ident(inner));
            }

            return Ok(ExprInner::Value(super::enum_::Value::new(
                obj.as_any().clone().unbind(),
                crate::serializers::SerializerType::String,
            )));
        }

        if pyo3::ffi::PyTuple_CheckExact(ptr) == 1 {
            let tuple = obj.cast_unchecked::<pyo3::types::PyTuple>();
            let mut items = Vec::with_capacity(tuple.len());

            for item in tuple.iter() {
                items.push(extract_expr(item.as_borrowed(), str_as_ident)?);
            }

            return Ok(Tuple { items }.into());
        }
    }

    Ok(ExprInner::Value(super::enum_::Value::infer_bound(obj.as_any().clone())))
}

/// Scalar `LIKE` pattern, or a list/tuple of patterns (`LIKE ANY`).
///
/// A Python `str` is never a sequence. An already-built `TupleExpr` is a
/// single pattern; a raw `tuple`/`list` is ANY.
pub fn extract_like_pattern(obj: pyo3::prelude::Borrowed<'_, '_, pyo3::prelude::PyAny>) -> pyo3::PyResult<LikePattern> {
    let ptr = obj.as_ptr();
    unsafe {
        if pyo3::ffi::PyUnicode_CheckExact(ptr) == 1 {
            return Ok(LikePattern::One(Box::new(extract_expr(obj, false)?)));
        }
        if pyo3::ffi::PyList_CheckExact(ptr) == 1 {
            let list = obj.cast_unchecked::<pyo3::types::PyList>();
            return extract_like_any(list.iter());
        }
        if pyo3::ffi::PyTuple_CheckExact(ptr) == 1 {
            let tuple = obj.cast_unchecked::<pyo3::types::PyTuple>();
            return extract_like_any(tuple.iter());
        }
    }
    Ok(LikePattern::One(Box::new(extract_expr(obj, false)?)))
}

fn extract_like_any<'py>(iter: impl IntoIterator<Item = pyo3::Bound<'py, pyo3::PyAny>>) -> pyo3::PyResult<LikePattern> {
    let mut items = Vec::new();
    for item in iter {
        items.push(item.extract::<LiteralExprExtractor>()?.0);
    }
    if items.is_empty() {
        return rapidquery_macros::pyerr!(Err(PyValueError), "LIKE ANY pattern list is empty");
    }
    Ok(LikePattern::Any(items))
}

pub(super) enum ExtractedIdentArgument {
    Ident(Ident),
    Part(IdentPart),
}

/// Hot-path extract: `str` first, then Ident / Unquote / Asterisk.
#[inline]
pub(super) fn extract_ident_argument(
    obj: pyo3::prelude::Borrowed<'_, '_, pyo3::prelude::PyAny>,
) -> pyo3::PyResult<ExtractedIdentArgument> {
    let ptr = obj.as_ptr();
    unsafe {
        // SAFETY: `ptr` is a live borrowed `PyObject`. Type pointers are
        // interned in `initialize_typeref` before any extractor runs.
        if pyo3::ffi::PyUnicode_CheckExact(ptr) == 1 {
            let s = utils::borrow_py_str_buffer(obj.py(), ptr)?;
            let part = IdentPart::quoted(s).map_err(|e| rapidquery_macros::pyerr!(PyValueError, "{}", e))?;

            return Ok(ExtractedIdentArgument::Part(part));
        }

        if crate::typeref::check_exact(ptr, crate::typeref::IDENT_TYPE) {
            let expr = obj.cast_unchecked::<super::base::PyExpr>();

            match &expr.get().0 {
                ExprInner::Ident(ident) => {
                    return Ok(ExtractedIdentArgument::Ident(ident.clone()));
                }
                _ => unreachable!("Ident instance always holds ExprInner::Ident"),
            }
        }

        if crate::typeref::check_exact(ptr, crate::typeref::UNQUOTE_TYPE) {
            let unquote = obj.cast_unchecked::<super::ident::PyUnquote>();
            let part = IdentPart::unquoted(unquote.get().0.as_str())
                .map_err(|e| rapidquery_macros::pyerr!(PyValueError, "{}", e))?;

            return Ok(ExtractedIdentArgument::Part(part));
        }

        if is_asterisk(ptr) {
            return Ok(ExtractedIdentArgument::Part(IdentPart::Asterisk));
        }
    }

    rapidquery_macros::pyerr!(
        Err(PyTypeError),
        "expected str, Ident, Unquote or Asterisk, got {}",
        unsafe { crate::internal::utils::get_type_name(obj.py(), obj.as_ptr()) }
    )
}

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for LiteralExprExtractor {
    type Error = pyo3::PyErr;

    #[inline]
    fn extract(obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>) -> Result<Self, Self::Error> {
        extract_expr(obj, false).map(Self)
    }
}

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for IdentExtractor {
    type Error = pyo3::PyErr;

    #[inline]
    fn extract(obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>) -> Result<Self, Self::Error> {
        unsafe {
            // SAFETY: `ptr` is a live borrowed `PyObject`.
            if pyo3::ffi::PyUnicode_CheckExact(obj.as_ptr()) == 1 {
                let s = utils::borrow_py_str_buffer(obj.py(), obj.as_ptr())?;
                let inner = Ident::from_str(s).map_err(|e| rapidquery_macros::pyerr!(PyValueError, "{}", e))?;

                return Ok(Self(inner));
            }
        }

        match extract_ident_argument(obj)? {
            ExtractedIdentArgument::Ident(ident) => Ok(Self(ident)),
            ExtractedIdentArgument::Part(part) => Ok(Self(Ident::from_one(part))),
        }
    }
}
