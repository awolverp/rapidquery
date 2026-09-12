use pyo3::types::{PyAnyMethods, PyTupleMethods};

use super::ident::{Ident, IdentExtractor};
use crate::internal::alias;
use crate::sql::{PrepareResult, SqlWriter, WriteSql, write_separated};

use super::enum_::{DuplicateTreatment, ExprEnum};
use super::{ExprExtractor, PyExpr, subclass};

#[derive(Clone, Debug, PartialEq)]
pub enum FuncArguments {
    /// Used for special functions like `CURRENT_TIMESTAMP` that are invoked
    /// without parentheses.
    None,
    /// A normal function argument list, including `DISTINCT` / `ALL`.
    List {
        duplicate_treatment: Option<DuplicateTreatment>,
        args: Vec<ExprEnum>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Func {
    pub name: Ident,
    pub args: FuncArguments,
    pub filter: Option<Box<ExprEnum>>,
}

impl Func {
    pub(crate) fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = format!("Func({})", self.name.python_repr());
        match &self.args {
            FuncArguments::None => buf.push_str(", parentheses=False"),
            FuncArguments::List {
                duplicate_treatment,
                args,
            } => {
                for arg in args {
                    buf.push_str(", ");
                    buf.push_str(&arg.python_repr(py)?);
                }
                if let Some(dt) = duplicate_treatment {
                    match dt {
                        DuplicateTreatment::Distinct => buf.push_str(", distinct=True"),
                        DuplicateTreatment::All => buf.push_str(", distinct=False"),
                    }
                }
            }
        }
        if let Some(filter) = &self.filter {
            buf.push_str(", filter=");
            buf.push_str(&filter.python_repr(py)?);
        }
        buf.push(')');
        Ok(buf)
    }
}

impl WriteSql for Func {
    fn write_sql<W: SqlWriter>(
        &self,
        dialect: &crate::dialect::SqlDialect,
        out: &mut W,
    ) -> PrepareResult<()> {
        self.name.write_sql(dialect, out)?;
        match &self.args {
            FuncArguments::None => {}
            FuncArguments::List {
                duplicate_treatment,
                args,
            } => {
                out.push_char('(');
                if let Some(dt) = duplicate_treatment {
                    out.push_str(dt.as_sql());
                    if !args.is_empty() {
                        out.push_char(' ');
                    }
                }
                write_separated(dialect, out, args, ", ")?;
                out.push_char(')');
            }
        }
        if let Some(filter) = &self.filter {
            out.push_str(" FILTER (WHERE ");
            filter.write_sql(dialect, out)?;
            out.push_char(')');
        }
        Ok(())
    }
}

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
///     parentheses: `False` omits `()`.
#[rapidquery_macros::pyclass(
    "Func",
    skip_from_py_object,
    extends = PyExpr,
    module = "rapidquery._core.expr"
)]
#[derive(Debug, Default)]
pub struct PyFunc;

#[inline]
fn func_inner<'a>(slf: &'a pyo3::PyRef<'a, PyFunc>) -> &'a Func {
    match &slf.as_super().0 {
        ExprEnum::Function(func) => func,
        _ => unreachable!("Func instance always holds ExprEnum::Function"),
    }
}

#[pyo3::pymethods]
impl PyFunc {
    #[new]
    #[pyo3(signature = (name, *args, distinct = None, filter = None, parentheses = true))]
    fn __new__(
        name: IdentExtractor,
        args: alias::ArgsType<'_>,
        distinct: Option<bool>,
        filter: Option<ExprExtractor>,
        parentheses: bool,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let func_args = if !parentheses {
            if !args.is_empty() || distinct.is_some() {
                return rapidquery_macros::pyerr!(
                    Err(PyValueError),
                    "parentheses=False cannot be combined with arguments or distinct"
                );
            }
            FuncArguments::None
        } else {
            let mut extracted = Vec::with_capacity(args.len());
            for item in args.iter() {
                extracted.push(item.extract::<ExprExtractor>()?.0);
            }
            FuncArguments::List {
                duplicate_treatment: match distinct {
                    Some(true) => Some(DuplicateTreatment::Distinct),
                    Some(false) => Some(DuplicateTreatment::All),
                    None => None,
                },
                args: extracted,
            }
        };

        Ok(subclass::<PyFunc>(ExprEnum::Function(Func {
            name: name.0,
            args: func_args,
            filter: filter.map(|ExprExtractor(expr)| Box::new(expr)),
        })))
    }

    #[getter]
    fn get_name(
        slf: pyo3::PyRef<'_, Self>,
    ) -> pyo3::PyResult<alias::BoundObject<'_>> {
        PyExpr(ExprEnum::Ident(func_inner(&slf).name.clone())).into_subclasses(slf.py())
    }

    #[getter]
    fn get_args(
        slf: pyo3::PyRef<'_, Self>,
    ) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let py = slf.py();
        match &func_inner(&slf).args {
            FuncArguments::None => Ok(py.None().into_bound(py).into_any()),
            FuncArguments::List { args, .. } => {
                let mut items = Vec::with_capacity(args.len());
                for arg in args {
                    items.push(PyExpr(arg.clone()).into_subclasses(py)?);
                }
                Ok(pyo3::types::PyTuple::new(py, items)?.into_any())
            }
        }
    }

    #[getter]
    fn get_filter(
        slf: pyo3::PyRef<'_, Self>,
    ) -> pyo3::PyResult<Option<alias::BoundObject<'_>>> {
        match &func_inner(&slf).filter {
            Some(expr) => Ok(Some(PyExpr((**expr).clone()).into_subclasses(slf.py())?)),
            None => Ok(None),
        }
    }
}
