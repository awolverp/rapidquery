use pyo3::types::{PyAnyMethods, PyTupleMethods};

use crate::data_type::PyDataType;
use crate::internal::alias;

use super::enum_::{BinaryOp, ExprEnum, IsKind, LikeKind, UnaryOp};
use super::{ExprExtractor, LiteralExprExtractor, PyExpr, subclass};

#[inline]
fn child<'py>(
    py: pyo3::Python<'py>,
    expr: &ExprEnum,
) -> pyo3::PyResult<alias::BoundObject<'py>> {
    PyExpr(expr.clone()).into_subclasses(py)
}

macro_rules! empty_node {
    ($(#[$meta:meta])* $name:ident => $pyname:literal) => {
        $(#[$meta])*
        #[rapidquery_macros::pyclass(
            $pyname,
            skip_from_py_object,
            extends = PyExpr,
            module = "rapidquery._core.expr"
        )]
        #[derive(Debug, Default)]
        pub struct $name;
    };
}

empty_node! {
    /// Parenthesized row / tuple expression `(a, b, …)`.
    PyTupleExpr => "TupleExpr"
}
empty_node! {
    /// Unary expression (`NOT x`, `-x`, `+x`, `~x`).
    PyUnary => "Unary"
}
empty_node! {
    /// Binary expression (`a + b`, `a AND b`, custom operators).
    PyBinary => "Binary"
}
empty_node! {
    /// `expr IS [NOT] {NULL|TRUE|FALSE|UNKNOWN}`.
    PyIs => "Is"
}
empty_node! {
    /// `a IS [NOT] DISTINCT FROM b`.
    PyIsDistinctFrom => "IsDistinctFrom"
}
empty_node! {
    /// `expr [NOT] IN (… )`.
    PyInList => "InList"
}
empty_node! {
    /// `expr [NOT] BETWEEN low AND high`.
    PyBetween => "Between"
}
empty_node! {
    /// `expr [NOT] LIKE / ILIKE / SIMILAR TO / RLIKE pattern`.
    PyLike => "Like"
}
empty_node! {
    /// `CAST(expr AS type)`.
    PyCast => "Cast"
}
empty_node! {
    /// `CASE [operand] WHEN … THEN … [ELSE …] END`.
    PyCase => "Case"
}
empty_node! {
    /// Escape-hatch SQL template. `{}` slots are filled by argument
    /// expressions. `{{` / `}}` emit literal braces.
    PyCustom => "Custom"
}

#[pyo3::pymethods]
impl PyTupleExpr {
    #[new]
    #[pyo3(signature = (*items))]
    fn __new__(items: alias::ArgsType<'_>) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let mut exprs = Vec::with_capacity(items.len());
        for item in items.iter() {
            exprs.push(item.extract::<ExprExtractor>()?.0);
        }
        Ok(subclass::<Self>(ExprEnum::Tuple(exprs)))
    }

    #[getter]
    fn get_items(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let py = slf.py();
        let ExprEnum::Tuple(items) = &slf.as_super().0 else {
            unreachable!("TupleExpr instance always holds ExprEnum::Tuple");
        };
        let mut out = Vec::with_capacity(items.len());
        for item in items {
            out.push(child(py, item)?);
        }
        Ok(pyo3::types::PyTuple::new(py, out)?.into_any())
    }
}

#[pyo3::pymethods]
impl PyUnary {
    #[new]
    #[pyo3(signature = (op, expr, /))]
    fn __new__(
        op: &str,
        expr: ExprExtractor,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let Some(op) = UnaryOp::parse(op) else {
            return rapidquery_macros::pyerr!(
                Err(PyValueError),
                "unknown unary operator {op:?}"
            );
        };
        Ok(subclass::<Self>(ExprEnum::Unary {
            op,
            expr: Box::new(expr.0),
        }))
    }

    #[getter]
    fn get_op(slf: pyo3::PyRef<'_, Self>) -> &'static str {
        let ExprEnum::Unary { op, .. } = &slf.as_super().0 else {
            unreachable!("Unary instance always holds ExprEnum::Unary");
        };
        op.as_sql()
    }

    #[getter]
    fn get_expr(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let ExprEnum::Unary { expr, .. } = &slf.as_super().0 else {
            unreachable!("Unary instance always holds ExprEnum::Unary");
        };
        child(slf.py(), expr)
    }
}

#[pyo3::pymethods]
impl PyBinary {
    #[new]
    #[pyo3(signature = (left, op, right, /))]
    fn __new__(
        left: ExprExtractor,
        op: &str,
        right: ExprExtractor,
    ) -> pyo3::PyClassInitializer<Self> {
        subclass::<Self>(ExprEnum::Binary {
            left: Box::new(left.0),
            op: BinaryOp::parse(op),
            right: Box::new(right.0),
        })
    }

    #[getter]
    fn get_left(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let ExprEnum::Binary { left, .. } = &slf.as_super().0 else {
            unreachable!("Binary instance always holds ExprEnum::Binary");
        };
        child(slf.py(), left)
    }

    #[getter]
    fn get_op(slf: pyo3::PyRef<'_, Self>) -> String {
        let ExprEnum::Binary { op, .. } = &slf.as_super().0 else {
            unreachable!("Binary instance always holds ExprEnum::Binary");
        };
        op.as_sql().to_owned()
    }

    #[getter]
    fn get_right(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let ExprEnum::Binary { right, .. } = &slf.as_super().0 else {
            unreachable!("Binary instance always holds ExprEnum::Binary");
        };
        child(slf.py(), right)
    }
}

#[pyo3::pymethods]
impl PyIs {
    #[new]
    #[pyo3(signature = (expr, kind, *, negated = false))]
    fn __new__(
        expr: ExprExtractor,
        kind: &str,
        negated: bool,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let Some(kind) = IsKind::parse(kind) else {
            return rapidquery_macros::pyerr!(
                Err(PyValueError),
                "unknown IS kind {kind:?}; expected NULL, TRUE, FALSE, or UNKNOWN"
            );
        };
        Ok(subclass::<Self>(ExprEnum::Is {
            expr: Box::new(expr.0),
            negated,
            kind,
        }))
    }

    #[getter]
    fn get_expr(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let ExprEnum::Is { expr, .. } = &slf.as_super().0 else {
            unreachable!("Is instance always holds ExprEnum::Is");
        };
        child(slf.py(), expr)
    }

    #[getter]
    fn get_kind(slf: pyo3::PyRef<'_, Self>) -> &'static str {
        let ExprEnum::Is { kind, .. } = &slf.as_super().0 else {
            unreachable!("Is instance always holds ExprEnum::Is");
        };
        kind.as_sql()
    }

    #[getter]
    fn get_negated(slf: pyo3::PyRef<'_, Self>) -> bool {
        let ExprEnum::Is { negated, .. } = &slf.as_super().0 else {
            unreachable!("Is instance always holds ExprEnum::Is");
        };
        *negated
    }
}

#[pyo3::pymethods]
impl PyIsDistinctFrom {
    #[new]
    #[pyo3(signature = (left, right, *, negated = false))]
    fn __new__(
        left: ExprExtractor,
        right: ExprExtractor,
        negated: bool,
    ) -> pyo3::PyClassInitializer<Self> {
        subclass::<Self>(ExprEnum::IsDistinctFrom {
            left: Box::new(left.0),
            right: Box::new(right.0),
            negated,
        })
    }

    #[getter]
    fn get_left(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let ExprEnum::IsDistinctFrom { left, .. } = &slf.as_super().0 else {
            unreachable!();
        };
        child(slf.py(), left)
    }

    #[getter]
    fn get_right(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let ExprEnum::IsDistinctFrom { right, .. } = &slf.as_super().0 else {
            unreachable!();
        };
        child(slf.py(), right)
    }

    #[getter]
    fn get_negated(slf: pyo3::PyRef<'_, Self>) -> bool {
        let ExprEnum::IsDistinctFrom { negated, .. } = &slf.as_super().0 else {
            unreachable!();
        };
        *negated
    }
}

#[pyo3::pymethods]
impl PyInList {
    #[new]
    #[pyo3(signature = (expr, list, *, negated = false))]
    fn __new__(
        expr: ExprExtractor,
        list: alias::BoundObject<'_>,
        negated: bool,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let mut items = Vec::new();
        for item in list.try_iter()? {
            items.push(item?.extract::<ExprExtractor>()?.0);
        }
        Ok(subclass::<Self>(ExprEnum::InList {
            expr: Box::new(expr.0),
            list: items,
            negated,
        }))
    }

    #[getter]
    fn get_expr(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let ExprEnum::InList { expr, .. } = &slf.as_super().0 else {
            unreachable!();
        };
        child(slf.py(), expr)
    }

    #[getter]
    fn get_list(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let py = slf.py();
        let ExprEnum::InList { list, .. } = &slf.as_super().0 else {
            unreachable!();
        };
        let mut out = Vec::with_capacity(list.len());
        for item in list {
            out.push(child(py, item)?);
        }
        Ok(pyo3::types::PyTuple::new(py, out)?.into_any())
    }

    #[getter]
    fn get_negated(slf: pyo3::PyRef<'_, Self>) -> bool {
        let ExprEnum::InList { negated, .. } = &slf.as_super().0 else {
            unreachable!();
        };
        *negated
    }
}

#[pyo3::pymethods]
impl PyBetween {
    #[new]
    #[pyo3(signature = (expr, low, high, *, negated = false))]
    fn __new__(
        expr: ExprExtractor,
        low: ExprExtractor,
        high: ExprExtractor,
        negated: bool,
    ) -> pyo3::PyClassInitializer<Self> {
        subclass::<Self>(ExprEnum::Between {
            expr: Box::new(expr.0),
            negated,
            low: Box::new(low.0),
            high: Box::new(high.0),
        })
    }

    #[getter]
    fn get_expr(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let ExprEnum::Between { expr, .. } = &slf.as_super().0 else {
            unreachable!();
        };
        child(slf.py(), expr)
    }

    #[getter]
    fn get_low(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let ExprEnum::Between { low, .. } = &slf.as_super().0 else {
            unreachable!();
        };
        child(slf.py(), low)
    }

    #[getter]
    fn get_high(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let ExprEnum::Between { high, .. } = &slf.as_super().0 else {
            unreachable!();
        };
        child(slf.py(), high)
    }

    #[getter]
    fn get_negated(slf: pyo3::PyRef<'_, Self>) -> bool {
        let ExprEnum::Between { negated, .. } = &slf.as_super().0 else {
            unreachable!();
        };
        *negated
    }
}

#[pyo3::pymethods]
impl PyLike {
    #[new]
    #[pyo3(signature = (expr, pattern, *, kind = "LIKE", negated = false, escape = None))]
    fn __new__(
        expr: ExprExtractor,
        pattern: LiteralExprExtractor,
        kind: &str,
        negated: bool,
        escape: Option<char>,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let Some(kind) = LikeKind::parse(kind) else {
            return rapidquery_macros::pyerr!(
                Err(PyValueError),
                "unknown LIKE kind {kind:?}; expected LIKE, ILIKE, SIMILAR TO, or RLIKE"
            );
        };
        Ok(subclass::<Self>(ExprEnum::Like {
            kind,
            negated,
            expr: Box::new(expr.0),
            pattern: Box::new(pattern.0),
            escape,
        }))
    }

    #[getter]
    fn get_expr(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let ExprEnum::Like { expr, .. } = &slf.as_super().0 else {
            unreachable!();
        };
        child(slf.py(), expr)
    }

    #[getter]
    fn get_pattern(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let ExprEnum::Like { pattern, .. } = &slf.as_super().0 else {
            unreachable!();
        };
        child(slf.py(), pattern)
    }

    #[getter]
    fn get_kind(slf: pyo3::PyRef<'_, Self>) -> &'static str {
        let ExprEnum::Like { kind, .. } = &slf.as_super().0 else {
            unreachable!();
        };
        kind.as_sql()
    }

    #[getter]
    fn get_negated(slf: pyo3::PyRef<'_, Self>) -> bool {
        let ExprEnum::Like { negated, .. } = &slf.as_super().0 else {
            unreachable!();
        };
        *negated
    }

    #[getter]
    fn get_escape(slf: pyo3::PyRef<'_, Self>) -> Option<char> {
        let ExprEnum::Like { escape, .. } = &slf.as_super().0 else {
            unreachable!();
        };
        *escape
    }
}

#[pyo3::pymethods]
impl PyCast {
    #[new]
    #[pyo3(signature = (expr, data_type, /))]
    fn __new__(expr: ExprExtractor, data_type: PyDataType) -> pyo3::PyClassInitializer<Self> {
        subclass::<Self>(ExprEnum::Cast {
            expr: Box::new(expr.0),
            data_type,
        })
    }

    #[getter]
    fn get_expr(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let ExprEnum::Cast { expr, .. } = &slf.as_super().0 else {
            unreachable!();
        };
        child(slf.py(), expr)
    }

    #[getter]
    fn get_data_type(slf: pyo3::PyRef<'_, Self>) -> PyDataType {
        let ExprEnum::Cast { data_type, .. } = &slf.as_super().0 else {
            unreachable!();
        };
        data_type.clone()
    }
}

#[pyo3::pymethods]
impl PyCase {
    #[new]
    #[pyo3(signature = (when_then, *, operand = None, else_ = None))]
    fn __new__(
        when_then: alias::BoundObject<'_>,
        operand: Option<ExprExtractor>,
        else_: Option<ExprExtractor>,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let mut pairs = Vec::new();
        for item in when_then.try_iter()? {
            let item = item?;
            let pair = item.cast::<pyo3::types::PyTuple>().map_err(|_| {
                rapidquery_macros::pyerr!(
                    PyTypeError,
                    "Case when_then items must be (when, then) pairs"
                )
            })?;
            if pair.len() != 2 {
                return rapidquery_macros::pyerr!(
                    Err(PyValueError),
                    "Case when_then items must be (when, then) pairs"
                );
            }
            let when = pair.get_item(0)?.extract::<ExprExtractor>()?.0;
            let then = pair.get_item(1)?.extract::<ExprExtractor>()?.0;
            pairs.push((when, then));
        }
        Ok(subclass::<Self>(ExprEnum::Case {
            operand: operand.map(|ExprExtractor(e)| Box::new(e)),
            when_then: pairs,
            else_result: else_.map(|ExprExtractor(e)| Box::new(e)),
        }))
    }

    #[getter]
    fn get_operand(
        slf: pyo3::PyRef<'_, Self>,
    ) -> pyo3::PyResult<Option<alias::BoundObject<'_>>> {
        let ExprEnum::Case { operand, .. } = &slf.as_super().0 else {
            unreachable!();
        };
        match operand {
            Some(expr) => Ok(Some(child(slf.py(), expr)?)),
            None => Ok(None),
        }
    }

    #[getter]
    fn get_when_then(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let py = slf.py();
        let ExprEnum::Case { when_then, .. } = &slf.as_super().0 else {
            unreachable!();
        };
        let mut out = Vec::with_capacity(when_then.len());
        for (when, then) in when_then {
            let pair = pyo3::types::PyTuple::new(
                py,
                [child(py, when)?, child(py, then)?],
            )?;
            out.push(pair);
        }
        Ok(pyo3::types::PyTuple::new(py, out)?.into_any())
    }

    #[getter]
    fn get_else_(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<Option<alias::BoundObject<'_>>> {
        let ExprEnum::Case { else_result, .. } = &slf.as_super().0 else {
            unreachable!();
        };
        match else_result {
            Some(expr) => Ok(Some(child(slf.py(), expr)?)),
            None => Ok(None),
        }
    }
}

#[pyo3::pymethods]
impl PyCustom {
    #[new]
    #[pyo3(signature = (sql, *args))]
    fn __new__(
        sql: String,
        args: alias::ArgsType<'_>,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let mut extracted = Vec::with_capacity(args.len());
        for item in args.iter() {
            extracted.push(item.extract::<ExprExtractor>()?.0);
        }
        Ok(subclass::<Self>(ExprEnum::Custom {
            sql,
            args: extracted,
        }))
    }

    #[getter]
    fn get_sql(slf: pyo3::PyRef<'_, Self>) -> String {
        let ExprEnum::Custom { sql, .. } = &slf.as_super().0 else {
            unreachable!();
        };
        sql.clone()
    }

    #[getter]
    fn get_args(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let py = slf.py();
        let ExprEnum::Custom { args, .. } = &slf.as_super().0 else {
            unreachable!();
        };
        let mut out = Vec::with_capacity(args.len());
        for arg in args {
            out.push(child(py, arg)?);
        }
        Ok(pyo3::types::PyTuple::new(py, out)?.into_any())
    }
}
