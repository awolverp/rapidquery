use std::str::FromStr;

use pyo3::types::PyTupleMethods;

use super::enum_::{ExprEnum, Ident, IdentPart};
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

/// Literal-priority extractor: a bare `str` becomes [`ExprEnum::Value`].
pub struct LiteralExprExtractor(pub ExprEnum);

/// Extracts a native [`Ident`](crate::Ident) from a Python object.
///
/// Accepts `Ident`, `str` (one quoted part, including `"*"`), `Unquote`,
/// `Asterisk`, and `type[Asterisk]`. Sequences are rejected.
pub struct IdentExtractor(pub Ident);

/// Accepts `PyExpr` only when it is a condition, not a value or ident.
pub struct ConditionExtractor(pub ExprEnum);

fn extract_expr(
    obj: pyo3::prelude::Borrowed<'_, '_, pyo3::prelude::PyAny>,
    str_as_ident: bool,
) -> pyo3::PyResult<ExprEnum> {
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

            return Ok(ExprEnum::Ident(Ident::from_one(part)));
        }

        if is_asterisk(ptr) {
            return Ok(ExprEnum::Ident(Ident::from_one(IdentPart::Asterisk)));
        }

        if pyo3::ffi::PyUnicode_CheckExact(ptr) == 1 {
            let s = utils::borrow_py_str_buffer(obj.py(), ptr)?;

            // a bare `str` becomes [`ExprEnum::Ident`] if `str_as_ident`
            if str_as_ident {
                let inner = Ident::from_str(s)
                    .map_err(|err| rapidquery_macros::pyerr!(PyValueError, "{}", err))?;

                return Ok(ExprEnum::Ident(inner));
            }

            return Ok(ExprEnum::Value(super::enum_::Value::new(
                obj.as_any().clone().unbind(),
                crate::data_type::SerializerEnum::String,
            )));
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

    Ok(ExprEnum::Value(super::enum_::Value::infer_bound(
        obj.as_any().clone(),
    )))
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
            let part = IdentPart::quoted(s)
                .map_err(|e| rapidquery_macros::pyerr!(PyValueError, "{}", e))?;

            return Ok(ExtractedIdentArgument::Part(part));
        }

        if crate::typeref::check_exact(ptr, crate::typeref::IDENT_TYPE) {
            let expr = obj.cast_unchecked::<super::base::PyExpr>();

            match &expr.get().0 {
                ExprEnum::Ident(ident) => {
                    return Ok(ExtractedIdentArgument::Ident(ident.clone()));
                }
                _ => unreachable!("Ident instance always holds ExprEnum::Ident"),
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
    fn extract(
        obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>,
    ) -> Result<Self, Self::Error> {
        extract_expr(obj, false).map(Self)
    }
}

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for IdentExtractor {
    type Error = pyo3::PyErr;

    #[inline]
    fn extract(
        obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>,
    ) -> Result<Self, Self::Error> {
        unsafe {
            // SAFETY: `ptr` is a live borrowed `PyObject`.
            if pyo3::ffi::PyUnicode_CheckExact(obj.as_ptr()) == 1 {
                let s = utils::borrow_py_str_buffer(obj.py(), obj.as_ptr())?;
                let inner = Ident::from_str(s)
                    .map_err(|e| rapidquery_macros::pyerr!(PyValueError, "{}", e))?;

                return Ok(Self(inner));
            }
        }

        match extract_ident_argument(obj)? {
            ExtractedIdentArgument::Ident(ident) => Ok(Self(ident)),
            ExtractedIdentArgument::Part(part) => Ok(Self(Ident::from_one(part))),
        }
    }
}

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for ConditionExtractor {
    type Error = pyo3::PyErr;

    #[inline]
    fn extract(
        obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>,
    ) -> Result<Self, Self::Error> {
        let ptr = obj.as_ptr();
        unsafe {
            if crate::typeref::check(ptr, crate::typeref::EXPR_TYPE) {
                let expr = obj.cast_unchecked::<super::base::PyExpr>();
                let inner = &expr.get().0;

                if inner.is_condition() {
                    return Ok(Self(inner.clone()));
                }
            }
        }

        rapidquery_macros::pyerr!(
            Err(PyTypeError),
            "expected a condition expression, got {}",
            unsafe { crate::internal::utils::get_type_name(obj.py(), obj.as_ptr()) }
        )
    }
}
