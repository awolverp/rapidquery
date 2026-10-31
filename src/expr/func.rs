use crate::error;
use pyo3::types::PyAnyMethods;
use pyo3::types::PyTupleMethods;

use crate::internal::alias;
use crate::internal::utils;

use super::base::PyExpr;
use super::enum_::NullTreatment;
use super::extractors::IdentExtractor;
use super::extractors::LiteralExprExtractor;

/// SQL function call. The name is an identifier, and each method returns a new call.
#[rapidquery_macros::pyclass("rapidquery.expr.Function", extends = PyExpr)]
#[derive(Debug, Default)]
pub struct PyFunction;

impl PyFunction {
    #[inline(always)]
    fn inner<'a>(slf: &'a pyo3::PyRef<'a, PyFunction>) -> &'a super::enum_::Function {
        debug_assert!(
            matches!(&slf.as_super().0, super::enum_::ExprInner::Function(..)),
            "PyFunction instance always holds ExprInner::Function"
        );

        match &slf.as_super().0 {
            super::enum_::ExprInner::Function(v) => v,
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    fn wrap(py: pyo3::Python<'_>, inner: super::enum_::Function) -> error::Result<alias::PyObject> {
        pyo3::Py::new(
            py,
            pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self),
        )
        .map(pyo3::Py::into_any)
        .map_err(error::Error::from)
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
    ) -> error::Result<pyo3::PyClassInitializer<Self>> {
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

    /// Return a copy whose filter predicate is `predicate`.
    fn filter(slf: pyo3::PyRef<'_, Self>, predicate: LiteralExprExtractor) -> error::Result<alias::PyObject> {
        let mut inner = Self::inner(&slf).clone();
        inner.filter = Some(Box::new(predicate.0));
        Self::wrap(slf.py(), inner)
    }

    /// Return a copy that writes `IGNORE NULLS`.
    fn ignore_nulls(slf: pyo3::PyRef<'_, Self>) -> error::Result<alias::PyObject> {
        let mut inner = Self::inner(&slf).clone();
        inner.null_treatment = Some(NullTreatment::IgnoreNulls);
        Self::wrap(slf.py(), inner)
    }

    /// Return a copy that writes `RESPECT NULLS`.
    fn respect_nulls(slf: pyo3::PyRef<'_, Self>) -> error::Result<alias::PyObject> {
        let mut inner = Self::inner(&slf).clone();
        inner.null_treatment = Some(NullTreatment::RespectNulls);
        Self::wrap(slf.py(), inner)
    }

    /// The function name.
    fn get_name(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        PyExpr(super::enum_::ExprInner::Ident(inner.name.clone())).into_subclasses(slf.py())
    }

    /// Whether the call writes `DISTINCT`.
    fn get_distinct(slf: pyo3::PyRef<'_, Self>) -> bool {
        let inner = Self::inner(&slf);
        inner.distinct
    }

    /// The argument expressions.
    fn get_args(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let py = slf.py();
        let inner = Self::inner(&slf);

        unsafe {
            utils::iterator_to_tuple(
                slf.py(),
                inner.args.len() as isize,
                inner
                    .args
                    .iter()
                    .map(|x| PyExpr(x.clone()).into_subclasses(py).map(|x| x.into_ptr())),
            )
        }
    }

    /// The filter predicate, or `None`.
    fn get_filter(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let py = slf.py();
        let inner = Self::inner(&slf);

        match &inner.filter {
            Some(e) => super::base::into_subclasses(py, (**e).clone()),
            None => Ok(utils::new_pynone(py)),
        }
    }

    /// `IGNORE NULLS`, `RESPECT NULLS`, or `None`.
    fn get_null_treatment(slf: pyo3::PyRef<'_, Self>) -> Option<&'static str> {
        Self::inner(&slf).null_treatment.map(NullTreatment::as_sql)
    }
}
