use pyo3::types::{PyAnyMethods, PyTupleMethods};

use super::base::PyExpr;
use super::enum_::ExprEnum;
use super::extractors::{ConditionExtractor, ExprExtractor, LiteralExprExtractor};
use crate::data_type::DataTypeName;
use crate::internal::{alias, utils};

macro_rules! empty_node {
    (
        $(
            $(#[$meta:meta])*
            $name:ident($variant:ident($inner:ty)) => $pyname:literal,
        )+
    ) => {
        $(
            $(#[$meta])*
            #[rapidquery_macros::pyclass(
                $pyname,
                skip_from_py_object,
                extends = PyExpr,
                module = "rapidquery._core.expr"
            )]
            #[derive(Debug, Default)]
            pub struct $name;

            impl $name {
                #[inline]
                fn inner<'a>(slf: &'a pyo3::PyRef<'a, $name>) -> &'a $inner {
                    debug_assert!(
                        matches!(&slf.as_super().0, ExprEnum::$variant(..)),
                        concat!(stringify!($name), " instance always holds ExprEnum::", stringify!($variant))

                    );

                    match &slf.as_super().0 {
                        ExprEnum::$variant(v) => v,
                        _ => unsafe { std::hint::unreachable_unchecked() },
                    }
                }
            }
        )+
    };
}

empty_node! {
    /// Parenthesized row / tuple expression `(a, b, …)`.
    PyTupleExpr(Tuple([ExprEnum])) => "TupleExpr",

    /// Unary expression (`NOT x`, `-x`, `+x`, `~x`).
    PyUnaryExpr(Unary(super::enum_::Unary)) => "UnaryExpr",

    /// Binary expression (`a + b`, `a AND b`, custom operators).
    PyBinaryExpr(Binary(super::enum_::Binary)) => "BinaryExpr",

    /// `expr IS [NOT] {NULL|TRUE|FALSE|UNKNOWN}`.
    PyIsExpr(Is(super::enum_::Is)) => "IsExpr",

    /// `a IS [NOT] DISTINCT FROM b`.
    PyIsDistinctFromExpr(IsDistinctFrom(super::enum_::IsDistinctFrom)) => "IsDistinctFromExpr",

    // /// `expr [NOT] IN (… )`.
    PyInListExpr(InList(super::enum_::InList)) => "InListExpr",

    // /// `expr [NOT] BETWEEN low AND high`.
    PyBetweenExpr(Between(super::enum_::Between)) => "BetweenExpr",

    /// `expr [NOT] LIKE / ILIKE / SIMILAR TO / RLIKE pattern`.
    PyLikeExpr(Like(super::enum_::Like)) => "LikeExpr",

    /// `CAST(expr AS type)`.
    PyCastExpr(Cast(super::enum_::Cast)) => "CastExpr",

    /// `CASE [operand] WHEN … THEN … [ELSE …] END`.
    PyCaseExpr(Case(super::enum_::Case)) => "CaseExpr",

    /// Escape-hatch SQL template. `{}` slots are filled by argument
    /// expressions. `{{` / `}}` emit literal braces.
    PyCustomExpr(Custom(super::enum_::Custom)) => "CustomExpr",
}

#[pyo3::pymethods]
impl PyTupleExpr {
    #[new]
    #[pyo3(signature = (*items))]
    fn __new__(items: alias::ArgsType<'_>) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        use pyo3::types::PyTupleMethods;

        let mut exprs = Vec::with_capacity(items.len());
        for item in items.iter() {
            exprs.push(item.extract::<ExprExtractor>()?.0);
        }

        Ok(pyo3::PyClassInitializer::from(PyExpr(ExprEnum::Tuple(exprs))).add_subclass(Self))
    }

    fn get_items(
        slf: pyo3::PyRef<'_, Self>,
    ) -> pyo3::PyResult<pyo3::Bound<'_, pyo3::types::PyTuple>> {
        let py = slf.py();
        let items = Self::inner(&slf);
        let mut out = Vec::with_capacity(items.len());

        for item in items {
            out.push(super::base::into_subclasses(py, item.clone())?);
        }
        pyo3::types::PyTuple::new(py, out)
    }
}

#[pyo3::pymethods]
impl PyUnaryExpr {
    #[new]
    #[pyo3(signature = (operator, expr, /))]
    fn __new__(
        operator: &str,
        expr: ExprExtractor,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let Some(op) = super::enum_::UnaryOp::parse(operator) else {
            return rapidquery_macros::pyerr!(
                Err(PyValueError),
                "unknown unary operator {:?}",
                operator
            );
        };

        let inner = super::enum_::Unary {
            op,
            expr: Box::new(expr.0),
        };
        Ok(pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self))
    }

    fn get_operator(slf: pyo3::PyRef<'_, Self>) -> &'static str {
        let inner = Self::inner(&slf);
        inner.op.as_sql()
    }

    fn get_expr(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.expr).clone())
    }
}

#[pyo3::pymethods]
impl PyBinaryExpr {
    #[new]
    #[pyo3(signature = (left, operator, right, /))]
    fn __new__(
        left: ExprExtractor,
        operator: &str,
        right: ExprExtractor,
    ) -> pyo3::PyClassInitializer<Self> {
        let inner = super::enum_::Binary {
            left: Box::new(left.0),
            op: super::enum_::BinaryOp::parse(operator),
            right: Box::new(right.0),
        };
        pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self)
    }

    fn get_left(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.left).clone())
    }

    fn get_operator(slf: pyo3::PyRef<'_, Self>) -> String {
        let inner = Self::inner(&slf);
        inner.op.as_sql().to_owned()
    }

    fn get_right(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.right).clone())
    }
}

#[pyo3::pymethods]
impl PyIsExpr {
    #[new]
    #[pyo3(signature = (expr, kind, /, *, negated = false))]
    fn __new__(
        expr: ExprExtractor,
        kind: &str,
        negated: bool,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let Some(kind) = super::enum_::IsKind::parse(kind) else {
            return rapidquery_macros::pyerr!(
                Err(PyValueError),
                "unknown IS kind {:?}; expected NULL, TRUE, FALSE, or UNKNOWN",
                kind
            );
        };

        let inner = super::enum_::Is {
            expr: Box::new(expr.0),
            negated,
            kind,
        };
        Ok(pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self))
    }

    fn get_expr(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.expr).clone())
    }

    fn get_kind(slf: pyo3::PyRef<'_, Self>) -> &'static str {
        let inner = Self::inner(&slf);
        inner.kind.as_sql()
    }

    fn get_negated(slf: pyo3::PyRef<'_, Self>) -> bool {
        let inner = Self::inner(&slf);
        inner.negated
    }
}

#[pyo3::pymethods]
impl PyIsDistinctFromExpr {
    #[new]
    #[pyo3(signature = (left, right, /, *, negated = false))]
    fn __new__(
        left: ExprExtractor,
        right: ExprExtractor,
        negated: bool,
    ) -> pyo3::PyClassInitializer<Self> {
        let inner = super::enum_::IsDistinctFrom {
            left: Box::new(left.0),
            right: Box::new(right.0),
            negated,
        };
        pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self)
    }

    fn get_left(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.left).clone())
    }

    fn get_right(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.right).clone())
    }

    fn get_negated(slf: pyo3::PyRef<'_, Self>) -> bool {
        let inner = Self::inner(&slf);
        inner.negated
    }
}

#[pyo3::pymethods]
impl PyInListExpr {
    #[new]
    #[pyo3(signature = (expr, list, /, *, negated = false))]
    fn __new__(
        expr: ExprExtractor,
        list: alias::BoundObject<'_>,
        negated: bool,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let mut items = Vec::new();
        for item in list.try_iter()? {
            items.push(item?.extract::<ExprExtractor>()?.0);
        }

        let inner = super::enum_::InList {
            expr: Box::new(expr.0),
            list: items,
            negated,
        };
        Ok(pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self))
    }

    fn get_expr(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.expr).clone())
    }

    fn get_list(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<pyo3::Bound<'_, pyo3::PyAny>> {
        let py = slf.py();
        let inner = Self::inner(&slf);

        // instead of creating a Vec and then creating a PyTuple, which is `T(n) = 2n`
        // we can use unsafe tuple to create PyTuple directly, which is `T(n) = n`
        let tuple = unsafe { pyo3::ffi::PyTuple_New(inner.list.len() as isize) };

        for (ix, arg) in inner.list.iter().enumerate() {
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

    fn get_negated(slf: pyo3::PyRef<'_, Self>) -> bool {
        let inner = Self::inner(&slf);
        inner.negated
    }
}

#[pyo3::pymethods]
impl PyBetweenExpr {
    #[new]
    #[pyo3(signature = (expr, low, high, /, *, negated = false))]
    fn __new__(
        expr: ExprExtractor,
        low: ExprExtractor,
        high: ExprExtractor,
        negated: bool,
    ) -> pyo3::PyClassInitializer<Self> {
        let inner = super::enum_::Between {
            expr: Box::new(expr.0),
            negated,
            low: Box::new(low.0),
            high: Box::new(high.0),
        };
        pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self)
    }

    fn get_expr(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.expr).clone())
    }

    fn get_low(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.low).clone())
    }

    fn get_high(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.high).clone())
    }

    fn get_negated(slf: pyo3::PyRef<'_, Self>) -> bool {
        let inner = Self::inner(&slf);
        inner.negated
    }
}

#[pyo3::pymethods]
impl PyLikeExpr {
    #[new]
    #[pyo3(signature = (expr, pattern, /, *, kind = "LIKE", negated = false, escape = None))]
    fn __new__(
        expr: ExprExtractor,
        pattern: LiteralExprExtractor,
        kind: &str,
        negated: bool,
        escape: Option<char>,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let Some(kind) = super::enum_::LikeKind::parse(kind) else {
            return rapidquery_macros::pyerr!(
                Err(PyValueError),
                "unknown LIKE kind {kind:?}; expected LIKE, ILIKE, SIMILAR TO, or RLIKE"
            );
        };
        let inner = super::enum_::Like {
            kind,
            negated,
            expr: Box::new(expr.0),
            pattern: Box::new(pattern.0),
            escape,
        };
        Ok(pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self))
    }

    fn get_expr(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.expr).clone())
    }

    fn get_pattern(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.pattern).clone())
    }

    fn get_kind(slf: pyo3::PyRef<'_, Self>) -> &'static str {
        let inner = Self::inner(&slf);
        inner.kind.as_sql()
    }

    fn get_negated(slf: pyo3::PyRef<'_, Self>) -> bool {
        let inner = Self::inner(&slf);
        inner.negated
    }

    fn get_escape(slf: pyo3::PyRef<'_, Self>) -> Option<char> {
        let inner = Self::inner(&slf);
        inner.escape
    }
}

#[pyo3::pymethods]
impl PyCastExpr {
    #[new]
    #[pyo3(signature = (expr, to, /))]
    fn __new__(expr: ExprExtractor, to: String) -> pyo3::PyClassInitializer<Self> {
        let data_type_name = DataTypeName::from(to);
        let inner = super::enum_::Cast {
            expr: Box::new(expr.0),
            data_type: data_type_name,
        };
        pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self)
    }

    fn get_expr(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.expr).clone())
    }

    fn get_data_type(slf: pyo3::PyRef<'_, Self>) -> String {
        let inner = Self::inner(&slf);
        inner.data_type.to_string()
    }
}
#[pyo3::pymethods]
impl PyCaseExpr {
    #[new]
    #[pyo3(signature = (operand = None, /))]
    fn __new__(operand: Option<ExprExtractor>) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let inner = super::enum_::Case {
            operand: operand.map(|ExprExtractor(e)| Box::new(e)),
            when: vec![],
            else_result: None,
        };
        Ok(pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self))
    }

    fn when(
        slf: pyo3::PyRef<'_, Self>,
        when: ConditionExtractor,
        then: ExprExtractor,
    ) -> pyo3::PyResult<alias::PyObject> {
        let mut inner = Self::inner(&slf).clone();
        inner.when.push((when.0, then.0));

        pyo3::Py::new(
            slf.py(),
            pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self),
        )
        .map(pyo3::Py::into_any)
    }

    fn else_(slf: pyo3::PyRef<'_, Self>, result: ExprExtractor) -> pyo3::PyResult<alias::PyObject> {
        let mut inner = Self::inner(&slf).clone();
        inner.else_result = Some(Box::new(result.0));

        pyo3::Py::new(
            slf.py(),
            pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self),
        )
        .map(pyo3::Py::into_any)
    }

    fn get_operand(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);

        match inner.operand.as_ref() {
            Some(expr) => {
                let bound = super::base::into_subclasses(slf.py(), (**expr).clone())?;
                Ok(bound)
            }
            None => Ok(slf.py().None().into_any().into_bound(slf.py())),
        }
    }

    fn get_when_list(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let py = slf.py();
        let inner = Self::inner(&slf);

        let mut out = Vec::with_capacity(inner.when.len());
        for (when, then) in inner.when.iter() {
            let pair = pyo3::types::PyTuple::new(
                py,
                [
                    super::base::into_subclasses(py, when.clone())?,
                    super::base::into_subclasses(py, then.clone())?,
                ],
            )?;
            out.push(pair);
        }
        Ok(pyo3::types::PyTuple::new(py, out)?.into_any())
    }

    fn get_else(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        match inner.else_result.as_ref() {
            Some(expr) => super::base::into_subclasses(slf.py(), (**expr).clone()),
            None => Ok(utils::new_pynone(slf.py())),
        }
    }
}

#[pyo3::pymethods]
impl PyCustomExpr {
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

        let inner = super::enum_::Custom {
            sql: sql.into(),
            args: extracted,
        };
        Ok(pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self))
    }

    fn get_sql(slf: pyo3::PyRef<'_, Self>) -> String {
        let inner = Self::inner(&slf);
        inner.sql.to_string()
    }

    fn get_args(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let py = slf.py();
        let inner = Self::inner(&slf);

        // instead of creating a Vec and then creating a PyTuple, which is `T(n) = 2n`
        // we can use unsafe tuple to create PyTuple directly, which is `T(n) = n`
        let tuple = unsafe { pyo3::ffi::PyTuple_New(inner.args.len() as isize) };

        for (ix, arg) in inner.args.iter().enumerate() {
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
