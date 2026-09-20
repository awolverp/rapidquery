use crate::internal::{alias, utils};

use super::base::PyExpr;
use super::enum_::{DuplicateTreatment, FunctionArguments};
use super::extractors::{ConditionExtractor, IdentExtractor, LiteralExprExtractor};

/// SQL function call.
///
/// `name` is an identifier (`SUM`, `schema.fn`). `parentheses=False` emits
/// `CURRENT_TIMESTAMP` with no argument list. `distinct=True` inserts
/// `DISTINCT` inside the parentheses.
///
/// Args:
///     name: Function name (`str`, `Ident`, `Unquote`).
///     args: Positional argument expressions. A bare `str` is a column.
///     distinct: `True` → `DISTINCT`, `False` → `ALL`, omitted → neither.
///     filter: Optional `FILTER (WHERE …)` predicate.
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
}

#[pyo3::pymethods]
impl PyFunction {
    #[new]
    #[pyo3(signature = (name, args = None, *, distinct = None, filter = None))]
    fn __new__(
        name: IdentExtractor,
        args: Option<Vec<LiteralExprExtractor>>,
        distinct: Option<bool>,
        filter: Option<ConditionExtractor>,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let func_args = match args {
            None => FunctionArguments::None,
            Some(x) => FunctionArguments::List {
                duplicate_treatment: match distinct {
                    Some(true) => Some(DuplicateTreatment::Distinct),
                    Some(false) => Some(DuplicateTreatment::All),
                    None => None,
                },
                args: x.into_iter().map(|LiteralExprExtractor(x)| x).collect(),
            },
        };

        let inner = super::enum_::Function {
            name: name.0,
            args: func_args,
            filter: filter.map(|ConditionExtractor(expr)| Box::new(expr)),
        };
        Ok(pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self))
    }

    fn get_name(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        PyExpr(super::enum_::ExprEnum::Ident(inner.name.clone())).into_subclasses(slf.py())
    }

    fn get_distinct(slf: pyo3::PyRef<'_, Self>) -> Option<bool> {
        let inner = Self::inner(&slf);

        match inner.args {
            FunctionArguments::None => None,
            FunctionArguments::List {
                duplicate_treatment: None,
                ..
            } => None,
            FunctionArguments::List {
                duplicate_treatment: Some(x),
                ..
            } => Some(matches!(x, DuplicateTreatment::Distinct)),
        }
    }

    fn get_args(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let py = slf.py();
        let inner = Self::inner(&slf);

        match &inner.args {
            FunctionArguments::None => Ok(utils::new_pynone(py)),
            FunctionArguments::List { args, .. } => {
                // instead of creating a Vec and then creating a PyTuple, which is `T(n) = 2n`
                // we can use unsafe tuple to create PyTuple directly, which is `T(n) = n`
                let tuple = unsafe { pyo3::ffi::PyTuple_New(args.len() as isize) };

                for (ix, arg) in args.iter().enumerate() {
                    match PyExpr(arg.clone()).into_subclasses(py) {
                        Ok(bound) => unsafe {
                            pyo3::ffi::PyTuple_SetItem(tuple, ix as isize, bound.into_ptr());
                        },
                        Err(err) => unsafe {
                            pyo3::ffi::Py_DECREF(tuple);
                            return Err(err);
                        },
                    }
                }

                Ok(unsafe { pyo3::Bound::from_owned_ptr(py, tuple) })
            }
        }
    }

    fn get_filter(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let py = slf.py();
        let inner = Self::inner(&slf);

        match &inner.filter {
            Some(e) => super::base::into_subclasses(py, (**e).clone()),
            None => Ok(utils::new_pynone(py)),
        }
    }
}
