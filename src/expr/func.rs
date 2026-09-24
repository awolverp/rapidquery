use pyo3::types::{PyAnyMethods, PyTupleMethods};

use crate::internal::{alias, utils};

use super::base::PyExpr;
use super::enum_::NullTreatment;
use super::extractors::{ConditionExtractor, IdentExtractor, LiteralExprExtractor};

/// SQL function call.
///
/// `name` is an identifier (`SUM`, `schema.fn`). The constructor emits a
/// bare name such as `CURRENT_TIMESTAMP` (no parentheses). `.call(...)`
/// adds an argument list; `.filter(...)` adds `FILTER (WHERE …)`;
/// `.ignore_nulls()` / `.respect_nulls()` add a postfix null-treatment
/// clause. Each method returns a new frozen `Function`.
///
/// Args:
///     name: Function name (`str`, `Ident`, `Unquote`).
#[rapidquery_macros::pyclass(
    "Function",
    skip_from_py_object,
    extends = super::base::PyExpr,
    module = "rapidquery._core.expr"
)]
#[derive(Debug, Default)]
pub struct PyFunction;

impl PyFunction {
    #[inline(always)]
    fn inner<'a>(slf: &'a pyo3::PyRef<'a, PyFunction>) -> &'a super::enum_::Function {
        debug_assert!(
            matches!(&slf.as_super().0, super::enum_::ExprEnum::Function(..)),
            "PyFunction instance always holds ExprEnum::Function"
        );

        match &slf.as_super().0 {
            super::enum_::ExprEnum::Function(v) => v,
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    fn wrap(
        py: pyo3::Python<'_>,
        inner: super::enum_::Function,
    ) -> pyo3::PyResult<alias::PyObject> {
        pyo3::Py::new(
            py,
            pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self),
        )
        .map(pyo3::Py::into_any)
    }
}

#[pyo3::pymethods]
impl PyFunction {
    #[new]
    #[pyo3(signature = (name, /, *args, distinct = false))]
    fn __new__(
        name: IdentExtractor,
        args: alias::ArgsType<'_>,
        distinct: bool,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let mut extracted = Vec::with_capacity(args.len());
        for item in args.iter() {
            extracted.push(item.extract::<LiteralExprExtractor>()?.0);
        }

        let inner = super::enum_::Function {
            name: name.0,
            args: extracted,
            distinct,
            filter: None,
            null_treatment: None,
        };
        Ok(pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self))
    }

    fn filter(
        slf: pyo3::PyRef<'_, Self>,
        predicate: ConditionExtractor,
    ) -> pyo3::PyResult<alias::PyObject> {
        let mut inner = Self::inner(&slf).clone();
        inner.filter = Some(Box::new(predicate.0));
        Self::wrap(slf.py(), inner)
    }

    fn ignore_nulls(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::PyObject> {
        let mut inner = Self::inner(&slf).clone();
        inner.null_treatment = Some(NullTreatment::IgnoreNulls);
        Self::wrap(slf.py(), inner)
    }

    fn respect_nulls(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::PyObject> {
        let mut inner = Self::inner(&slf).clone();
        inner.null_treatment = Some(NullTreatment::RespectNulls);
        Self::wrap(slf.py(), inner)
    }

    fn get_name(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        PyExpr(super::enum_::ExprEnum::Ident(inner.name.clone())).into_subclasses(slf.py())
    }

    fn get_distinct(slf: pyo3::PyRef<'_, Self>) -> bool {
        let inner = Self::inner(&slf);
        inner.distinct
    }

    fn get_args(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let py = slf.py();
        let inner = Self::inner(&slf);

        let len = inner.args.len() as isize;
        utils::iterator_to_tuple(
            slf.py(),
            len,
            inner
                .args
                .iter()
                .map(|x| PyExpr(x.clone()).into_subclasses(py).map(|x| x.as_ptr())),
        )
    }

    fn get_filter(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let py = slf.py();
        let inner = Self::inner(&slf);

        match &inner.filter {
            Some(e) => super::base::into_subclasses(py, (**e).clone()),
            None => Ok(utils::new_pynone(py)),
        }
    }

    fn get_null_treatment(slf: pyo3::PyRef<'_, Self>) -> Option<&'static str> {
        Self::inner(&slf).null_treatment.map(NullTreatment::as_sql)
    }
}
