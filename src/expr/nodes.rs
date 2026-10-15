use pyo3::types::PyAnyMethods;
use pyo3::types::PyTupleMethods;

use super::base::PyExpr;
use super::enum_::ExprInner;
use super::extractors::IdentExtractor;
use super::extractors::LiteralExprExtractor;
use super::extractors::extract_like_pattern;
use crate::internal::alias;
use crate::internal::utils;

macro_rules! empty_node {
    (
        $(
            $(#[$meta:meta])*
            $name:ident($variant:ident($inner:ty)) => $pyname:literal,
        )+
    ) => {
        $(
            $(#[$meta])*
            #[rapidquery_macros::pyclass("rapidquery.expr" + $pyname, skip_from_py_object, extends = PyExpr)]
            #[derive(Debug, Default)]
            pub struct $name;

            impl $name {
                #[inline]
                fn inner<'a>(slf: &'a pyo3::PyRef<'a, $name>) -> &'a $inner {
                    debug_assert!(
                        matches!(&slf.as_super().0, ExprInner::$variant(..)),
                        concat!(stringify!($name), " instance always holds ExprInner::", stringify!($variant))
                    );

                    match &slf.as_super().0 {
                        ExprInner::$variant(v) => v,
                        _ => unsafe { std::hint::unreachable_unchecked() },
                    }
                }
            }
        )+
    };
}

empty_node! {
    /// Parenthesized row / tuple expression `(a, b, …)`.
    PyTupleExpr(Tuple(super::enum_::Tuple)) => "TupleExpr",

    /// Prefix expression: `NOT x`, `+x`, `-x`, or `~x`.
    PyUnaryExpr(Unary(super::enum_::Unary)) => "UnaryExpr",

    /// Binary expression (`a + b`, `a AND b`, custom operators).
    PyBinaryExpr(Binary(super::enum_::Binary)) => "BinaryExpr",

    /// `expr IS [NOT] {NULL|TRUE|FALSE|UNKNOWN}`.
    PyIsExpr(Is(super::enum_::Is)) => "IsExpr",

    /// `a IS [NOT] DISTINCT FROM b`.
    PyIsDistinctFromExpr(IsDistinctFrom(super::enum_::IsDistinctFrom)) => "IsDistinctFromExpr",

    /// `expr [NOT] IN (… )`.
    PyInListExpr(InList(super::enum_::InList)) => "InListExpr",

    /// `expr [NOT] BETWEEN low AND high`.
    PyBetweenExpr(Between(super::enum_::Between)) => "BetweenExpr",

    /// `expr [NOT] LIKE / ILIKE [ANY] pattern`.
    PyLikeExpr(Like(super::enum_::Like)) => "LikeExpr",

    /// `expr [NOT] SIMILAR TO pattern`.
    PySimilarToExpr(SimilarTo(super::enum_::SimilarTo)) => "SimilarToExpr",

    /// Regex match (`RLIKE`, `~`, `REGEXP_LIKE`, …).
    PyRLikeExpr(RLike(super::enum_::RLike)) => "RLikeExpr",

    /// `CAST(expr AS type)`, `TRY_CAST(expr AS type)`, or `(expr)::type`.
    PyCastExpr(Cast(super::enum_::Cast)) => "CastExpr",

    /// `TRIM` of `expr`, optionally from one side and optionally of `characters`.
    PyTrimExpr(Trim(super::enum_::Trim)) => "TrimExpr",

    /// `expr COLLATE collation`.
    PyCollateExpr(Collate(super::enum_::Collate)) => "CollateExpr",

    /// `CASE [operand] WHEN … THEN … [ELSE …] END`.
    PyCaseExpr(Case(super::enum_::Case)) => "CaseExpr",

    /// SQL template. `{}` slots are filled by arguments. `{{` and `}}` are braces.
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
            exprs.push(item.extract::<LiteralExprExtractor>()?.0);
        }

        Ok(pyo3::PyClassInitializer::from(PyExpr(super::enum_::Tuple { items: exprs }.into())).add_subclass(Self))
    }

    fn get_items(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<pyo3::Bound<'_, pyo3::PyAny>> {
        let py = slf.py();
        let items = &Self::inner(&slf).items;

        unsafe {
            utils::iterator_to_tuple(
                py,
                items.len() as isize,
                items
                    .iter()
                    .map(|x| super::base::into_subclasses(py, x.clone()).map(|x| x.into_ptr())),
            )
        }
    }
}

impl PyUnaryExpr {
    pub(super) fn new_base(op: super::enum_::UnaryOp, expr: ExprInner) -> PyExpr {
        let inner = super::enum_::Unary {
            op,
            expr: Box::new(expr),
        };
        PyExpr(inner.into())
    }
}

#[pyo3::pymethods]
impl PyUnaryExpr {
    #[new]
    #[pyo3(signature = (operator, expr, /))]
    fn __new__(operator: &str, expr: LiteralExprExtractor) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let Some(op) = super::enum_::UnaryOp::parse(operator) else {
            return rapidquery_macros::pyerr!(Err(PyValueError), "unknown unary operator {:?}", operator);
        };

        let base = Self::new_base(op, expr.0);
        Ok(pyo3::PyClassInitializer::from(base).add_subclass(Self))
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

impl PyBinaryExpr {
    pub(super) fn new_base(left: ExprInner, op: super::enum_::BinaryOp, right: ExprInner) -> PyExpr {
        let inner = super::enum_::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
        PyExpr(inner.into())
    }
}

#[pyo3::pymethods]
impl PyBinaryExpr {
    #[new]
    #[pyo3(signature = (left, operator, right, /))]
    fn __new__(
        left: LiteralExprExtractor,
        operator: &str,
        right: LiteralExprExtractor,
    ) -> pyo3::PyClassInitializer<Self> {
        let base = Self::new_base(left.0, super::enum_::BinaryOp::parse(operator), right.0);
        pyo3::PyClassInitializer::from(base).add_subclass(Self)
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

impl PyIsExpr {
    pub(super) fn new_base(expr: ExprInner, kind: &str, negated: bool) -> pyo3::PyResult<PyExpr> {
        let Some(kind) = super::enum_::IsKind::parse(kind) else {
            return rapidquery_macros::pyerr!(
                Err(PyValueError),
                "unknown IS kind {:?}; expected NULL, TRUE, FALSE, or UNKNOWN",
                kind
            );
        };

        let inner = super::enum_::Is {
            expr: Box::new(expr),
            negated,
            kind,
        };
        Ok(PyExpr(inner.into()))
    }
}

#[pyo3::pymethods]
impl PyIsExpr {
    #[new]
    #[pyo3(signature = (expr, kind, /, *, negated = false))]
    fn __new__(
        expr: LiteralExprExtractor,
        kind: &str,
        negated: bool,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let base = Self::new_base(expr.0, kind, negated)?;
        Ok(pyo3::PyClassInitializer::from(base).add_subclass(Self))
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

impl PyIsDistinctFromExpr {
    pub(super) fn new_base(left: ExprInner, right: ExprInner, negated: bool) -> PyExpr {
        let inner = super::enum_::IsDistinctFrom {
            left: Box::new(left),
            right: Box::new(right),
            negated,
        };
        PyExpr(inner.into())
    }
}

#[pyo3::pymethods]
impl PyIsDistinctFromExpr {
    #[new]
    #[pyo3(signature = (left, right, /, *, negated = false))]
    fn __new__(
        left: LiteralExprExtractor,
        right: LiteralExprExtractor,
        negated: bool,
    ) -> pyo3::PyClassInitializer<Self> {
        let base = Self::new_base(left.0, right.0, negated);
        pyo3::PyClassInitializer::from(base).add_subclass(Self)
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

impl PyInListExpr {
    pub(super) fn new_base(expr: ExprInner, list: alias::BoundObject<'_>, negated: bool) -> pyo3::PyResult<PyExpr> {
        let mut items = Vec::new();
        for item in list.try_iter()? {
            items.push(item?.extract::<LiteralExprExtractor>()?.0);
        }

        let inner = super::enum_::InList {
            expr: Box::new(expr),
            list: items,
            negated,
        };
        Ok(PyExpr(inner.into()))
    }
}

#[pyo3::pymethods]
impl PyInListExpr {
    #[new]
    #[pyo3(signature = (expr, list, /, *, negated = false))]
    fn __new__(
        expr: LiteralExprExtractor,
        list: alias::BoundObject<'_>,
        negated: bool,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let base = Self::new_base(expr.0, list, negated)?;
        Ok(pyo3::PyClassInitializer::from(base).add_subclass(Self))
    }

    fn get_expr(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.expr).clone())
    }

    fn get_list(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<pyo3::Bound<'_, pyo3::PyAny>> {
        let py = slf.py();
        let inner = Self::inner(&slf);

        unsafe {
            utils::iterator_to_tuple(
                slf.py(),
                inner.list.len() as isize,
                inner
                    .list
                    .iter()
                    .map(|x| PyExpr(x.clone()).into_subclasses(py).map(|x| x.into_ptr())),
            )
        }
    }

    fn get_negated(slf: pyo3::PyRef<'_, Self>) -> bool {
        let inner = Self::inner(&slf);
        inner.negated
    }
}

impl PyBetweenExpr {
    pub(super) fn new_base(expr: ExprInner, low: ExprInner, high: ExprInner, negated: bool) -> PyExpr {
        let inner = super::enum_::Between {
            expr: Box::new(expr),
            negated,
            low: Box::new(low),
            high: Box::new(high),
        };
        PyExpr(inner.into())
    }
}

#[pyo3::pymethods]
impl PyBetweenExpr {
    #[new]
    #[pyo3(signature = (expr, low, high, /, *, negated = false))]
    fn __new__(
        expr: LiteralExprExtractor,
        low: LiteralExprExtractor,
        high: LiteralExprExtractor,
        negated: bool,
    ) -> pyo3::PyClassInitializer<Self> {
        let base = Self::new_base(expr.0, low.0, high.0, negated);
        pyo3::PyClassInitializer::from(base).add_subclass(Self)
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

impl PyLikeExpr {
    pub(super) fn new_base(
        expr: ExprInner,
        pattern: alias::BoundObject<'_>,
        negated: bool,
        case_sensitive: bool,
        escape: Option<char>,
    ) -> pyo3::PyResult<PyExpr> {
        let inner = super::enum_::Like {
            case_sensitive,
            negated,
            expr: Box::new(expr),
            pattern: extract_like_pattern(pattern.as_borrowed())?,
            escape,
        };
        Ok(PyExpr(inner.into()))
    }
}

#[pyo3::pymethods]
impl PyLikeExpr {
    #[new]
    #[pyo3(signature = (expr, pattern, /, *, negated = false, case_sensitive = true, escape = None))]
    fn __new__(
        expr: LiteralExprExtractor,
        pattern: alias::BoundObject<'_>,
        negated: bool,
        case_sensitive: bool,
        escape: Option<char>,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let base = Self::new_base(expr.0, pattern, negated, case_sensitive, escape)?;
        Ok(pyo3::PyClassInitializer::from(base).add_subclass(Self))
    }

    fn get_expr(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.expr).clone())
    }

    fn get_pattern(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let py = slf.py();
        let inner = Self::inner(&slf);
        match &inner.pattern {
            super::enum_::LikePattern::One(p) => super::base::into_subclasses(py, (**p).clone()),
            super::enum_::LikePattern::Any(ps) => unsafe {
                utils::iterator_to_tuple(
                    slf.py(),
                    ps.len() as isize,
                    ps.iter()
                        .map(|x| PyExpr(x.clone()).into_subclasses(py).map(|x| x.into_ptr())),
                )
            },
        }
    }

    fn get_negated(slf: pyo3::PyRef<'_, Self>) -> bool {
        let inner = Self::inner(&slf);
        inner.negated
    }

    fn get_case_sensitive(slf: pyo3::PyRef<'_, Self>) -> bool {
        let inner = Self::inner(&slf);
        inner.case_sensitive
    }

    fn get_escape(slf: pyo3::PyRef<'_, Self>) -> Option<char> {
        let inner = Self::inner(&slf);
        inner.escape
    }
}

impl PySimilarToExpr {
    pub(super) fn new_base(expr: ExprInner, pattern: ExprInner, negated: bool) -> PyExpr {
        let inner = super::enum_::SimilarTo {
            negated,
            expr: Box::new(expr),
            pattern: Box::new(pattern),
        };
        PyExpr(inner.into())
    }
}

#[pyo3::pymethods]
impl PySimilarToExpr {
    #[new]
    #[pyo3(signature = (expr, pattern, /, *, negated = false))]
    fn __new__(
        expr: LiteralExprExtractor,
        pattern: LiteralExprExtractor,
        negated: bool,
    ) -> pyo3::PyClassInitializer<Self> {
        let base = Self::new_base(expr.0, pattern.0, negated);
        pyo3::PyClassInitializer::from(base).add_subclass(Self)
    }

    fn get_expr(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.expr).clone())
    }

    fn get_pattern(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.pattern).clone())
    }

    fn get_negated(slf: pyo3::PyRef<'_, Self>) -> bool {
        let inner = Self::inner(&slf);
        inner.negated
    }
}

impl PyRLikeExpr {
    pub(super) fn new_base(expr: ExprInner, pattern: ExprInner, negated: bool) -> PyExpr {
        let inner = super::enum_::RLike {
            negated,
            expr: Box::new(expr),
            pattern: Box::new(pattern),
        };
        PyExpr(inner.into())
    }
}

#[pyo3::pymethods]
impl PyRLikeExpr {
    #[new]
    #[pyo3(signature = (expr, pattern, /, *, negated = false))]
    fn __new__(
        expr: LiteralExprExtractor,
        pattern: LiteralExprExtractor,
        negated: bool,
    ) -> pyo3::PyClassInitializer<Self> {
        let base = Self::new_base(expr.0, pattern.0, negated);
        pyo3::PyClassInitializer::from(base).add_subclass(Self)
    }

    fn get_expr(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.expr).clone())
    }

    fn get_pattern(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.pattern).clone())
    }

    fn get_negated(slf: pyo3::PyRef<'_, Self>) -> bool {
        let inner = Self::inner(&slf);
        inner.negated
    }
}

impl PyCastExpr {
    pub(super) fn new_base(expr: ExprInner, data_type_name: &str, kind: &str) -> pyo3::PyResult<PyExpr> {
        let Some(kind) = super::enum_::CastKind::parse(kind) else {
            return rapidquery_macros::pyerr!(Err(PyValueError), "unknown cast kind {:?}", kind);
        };

        let inner = super::enum_::Cast {
            kind,
            expr: Box::new(expr),
            data_type_name: data_type_name.into(),
        };
        Ok(PyExpr(inner.into()))
    }
}

#[pyo3::pymethods]
impl PyCastExpr {
    #[new]
    #[pyo3(signature = (expr, data_type_name, /, *, kind = "CAST"))]
    fn __new__(
        expr: LiteralExprExtractor,
        data_type_name: &str,
        kind: &str,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let base = Self::new_base(expr.0, data_type_name, kind)?;
        Ok(pyo3::PyClassInitializer::from(base).add_subclass(Self))
    }

    fn get_expr(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.expr).clone())
    }

    fn get_data_type_name(slf: pyo3::PyRef<'_, Self>) -> String {
        let inner = Self::inner(&slf);
        inner.data_type_name.to_string()
    }

    fn get_kind(slf: pyo3::PyRef<'_, Self>) -> &'static str {
        let inner = Self::inner(&slf);
        inner.kind.as_sql()
    }
}

impl PyTrimExpr {
    pub(super) fn new_base(expr: ExprInner, r#where: &str, characters: Option<ExprInner>) -> pyo3::PyResult<PyExpr> {
        let Some(where_) = super::enum_::TrimWhere::parse(r#where) else {
            return rapidquery_macros::pyerr!(
                Err(PyValueError),
                "unknown TRIM where {:?}; expected BOTH, LEADING, or TRAILING",
                r#where
            );
        };

        let inner = super::enum_::Trim {
            expr: Box::new(expr),
            where_,
            characters: characters.map(Box::new),
        };
        Ok(PyExpr(inner.into()))
    }
}

#[pyo3::pymethods]
impl PyTrimExpr {
    #[new]
    #[pyo3(signature = (expr, /, *, r#where = "BOTH", characters = None))]
    fn __new__(
        expr: LiteralExprExtractor,
        r#where: &str,
        characters: Option<LiteralExprExtractor>,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let base = Self::new_base(expr.0, r#where, characters.map(|LiteralExprExtractor(e)| e))?;
        Ok(pyo3::PyClassInitializer::from(base).add_subclass(Self))
    }

    fn get_expr(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.expr).clone())
    }

    fn get_where(slf: pyo3::PyRef<'_, Self>) -> &'static str {
        let inner = Self::inner(&slf);
        inner.where_.as_sql()
    }

    fn get_characters(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        match inner.characters.as_ref() {
            Some(expr) => super::base::into_subclasses(slf.py(), (**expr).clone()),
            None => Ok(utils::new_pynone(slf.py())),
        }
    }
}

impl PyCollateExpr {
    pub(super) fn new_base(expr: ExprInner, collation: super::enum_::Ident) -> PyExpr {
        let inner = super::enum_::Collate {
            expr: Box::new(expr),
            collation,
        };
        PyExpr(inner.into())
    }
}

#[pyo3::pymethods]
impl PyCollateExpr {
    #[new]
    #[pyo3(signature = (expr, collation, /))]
    fn __new__(expr: LiteralExprExtractor, collation: IdentExtractor) -> pyo3::PyClassInitializer<Self> {
        let base = Self::new_base(expr.0, collation.0);
        pyo3::PyClassInitializer::from(base).add_subclass(Self)
    }

    fn get_expr(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.expr).clone())
    }

    fn get_collation(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), inner.collation.clone().into())
    }
}
#[pyo3::pymethods]
impl PyCaseExpr {
    #[new]
    #[pyo3(signature = (operand = None, /))]
    fn __new__(operand: Option<LiteralExprExtractor>) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let inner = super::enum_::Case {
            operand: operand.map(|LiteralExprExtractor(e)| Box::new(e)),
            when: vec![],
            else_result: None,
        };
        Ok(pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self))
    }

    fn when(
        slf: pyo3::PyRef<'_, Self>,
        when: LiteralExprExtractor,
        then: LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::PyObject> {
        let mut inner = Self::inner(&slf).clone();
        inner.when.push((when.0, then.0));

        pyo3::Py::new(
            slf.py(),
            pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self),
        )
        .map(pyo3::Py::into_any)
    }

    fn else_(slf: pyo3::PyRef<'_, Self>, result: LiteralExprExtractor) -> pyo3::PyResult<alias::PyObject> {
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
    fn __new__(sql: String, args: alias::ArgsType<'_>) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let mut extracted = Vec::with_capacity(args.len());

        for item in args.iter() {
            extracted.push(item.extract::<LiteralExprExtractor>()?.0);
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
}
