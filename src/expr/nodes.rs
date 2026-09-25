use pyo3::types::{PyAnyMethods, PyTupleMethods};

use super::base::PyExpr;
use super::enum_::ExprInner;
use super::extractors::{
    ConditionExtractor, IdentExtractor, LiteralExprExtractor, extract_like_pattern,
    extract_trim_characters,
};
use crate::data_type::DataTypeNameExtractor;
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

    /// Unary expression (`NOT x`, `-x`, `|/x`, `x!`, …).
    PyUnaryExpr(Unary(super::enum_::Unary)) => "UnaryExpr",

    /// Binary expression (`a + b`, `a AND b`, custom operators).
    PyBinaryExpr(Binary(super::enum_::Binary)) => "BinaryExpr",

    /// `expr IS [NOT] {NULL|TRUE|FALSE|UNKNOWN}`.
    PyIsExpr(Is(super::enum_::Is)) => "IsExpr",

    /// `expr IS [NOT] [form] NORMALIZED`.
    PyIsNormalizedExpr(IsNormalized(super::enum_::IsNormalized)) => "IsNormalizedExpr",

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

    /// `expr [NOT] RLIKE pattern`.
    PyRLikeExpr(RLike(super::enum_::RLike)) => "RLikeExpr",

    /// `CAST(expr AS type [FORMAT …])`.
    PyCastExpr(Cast(super::enum_::Cast)) => "CastExpr",

    /// `TRIM([BOTH|LEADING|TRAILING] [what FROM] expr)`.
    PyTrimExpr(Trim(super::enum_::Trim)) => "TrimExpr",

    /// `expr COLLATE collation`.
    PyCollateExpr(Collate(super::enum_::Collate)) => "CollateExpr",

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
            exprs.push(item.extract::<LiteralExprExtractor>()?.0);
        }

        Ok(
            pyo3::PyClassInitializer::from(PyExpr(super::enum_::Tuple { items: exprs }.into()))
                .add_subclass(Self),
        )
    }

    fn get_items(
        slf: pyo3::PyRef<'_, Self>,
    ) -> pyo3::PyResult<pyo3::Bound<'_, pyo3::types::PyTuple>> {
        let py = slf.py();
        let items = &Self::inner(&slf).items;
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
        expr: LiteralExprExtractor,
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
        left: LiteralExprExtractor,
        operator: &str,
        right: LiteralExprExtractor,
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
        expr: LiteralExprExtractor,
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
impl PyIsNormalizedExpr {
    #[new]
    #[pyo3(signature = (expr, form = None, /, *, negated = false))]
    fn __new__(
        expr: LiteralExprExtractor,
        form: Option<&str>,
        negated: bool,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let form = match form {
            None => None,
            Some(s) => {
                let Some(parsed) = super::enum_::NormalizationForm::parse(s) else {
                    return rapidquery_macros::pyerr!(
                        Err(PyValueError),
                        "unknown normalization form {:?}; expected NFC, NFD, NFKC, or NFKD",
                        s
                    );
                };
                Some(parsed)
            }
        };

        let inner = super::enum_::IsNormalized {
            expr: Box::new(expr.0),
            negated,
            form,
        };
        Ok(pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self))
    }

    fn get_expr(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.expr).clone())
    }

    fn get_form(slf: pyo3::PyRef<'_, Self>) -> Option<&'static str> {
        let inner = Self::inner(&slf);
        inner.form.map(super::enum_::NormalizationForm::as_sql)
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
        left: LiteralExprExtractor,
        right: LiteralExprExtractor,
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
        expr: LiteralExprExtractor,
        list: alias::BoundObject<'_>,
        negated: bool,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let mut items = Vec::new();
        for item in list.try_iter()? {
            items.push(item?.extract::<LiteralExprExtractor>()?.0);
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

        let len = inner.list.len() as isize;
        utils::iterator_to_tuple(
            slf.py(),
            len,
            inner
                .list
                .iter()
                .map(|x| PyExpr(x.clone()).into_subclasses(py).map(|x| x.as_ptr())),
        )
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
        expr: LiteralExprExtractor,
        low: LiteralExprExtractor,
        high: LiteralExprExtractor,
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
    #[pyo3(signature = (expr, pattern, /, *, negated = false, case_sensitive = true, escape = None))]
    fn __new__(
        expr: LiteralExprExtractor,
        pattern: alias::BoundObject<'_>,
        negated: bool,
        case_sensitive: bool,
        escape: Option<char>,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let inner = super::enum_::Like {
            case_sensitive,
            negated,
            expr: Box::new(expr.0),
            pattern: extract_like_pattern(pattern.as_borrowed())?,
            escape,
        };
        Ok(pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self))
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
            super::enum_::LikePattern::Any(ps) => {
                let len = ps.len() as isize;
                utils::iterator_to_tuple(
                    slf.py(),
                    len,
                    ps.iter()
                        .map(|x| PyExpr(x.clone()).into_subclasses(py).map(|x| x.as_ptr())),
                )
            }
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

#[pyo3::pymethods]
impl PySimilarToExpr {
    #[new]
    #[pyo3(signature = (expr, pattern, /, *, negated = false, escape = None))]
    fn __new__(
        expr: LiteralExprExtractor,
        pattern: LiteralExprExtractor,
        negated: bool,
        escape: Option<char>,
    ) -> pyo3::PyClassInitializer<Self> {
        let inner = super::enum_::SimilarTo {
            negated,
            expr: Box::new(expr.0),
            pattern: Box::new(pattern.0),
            escape,
        };
        pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self)
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

    fn get_escape(slf: pyo3::PyRef<'_, Self>) -> Option<char> {
        let inner = Self::inner(&slf);
        inner.escape
    }
}

#[pyo3::pymethods]
impl PyRLikeExpr {
    #[new]
    #[pyo3(signature = (expr, pattern, /, *, negated = false, escape = None))]
    fn __new__(
        expr: LiteralExprExtractor,
        pattern: LiteralExprExtractor,
        negated: bool,
        escape: Option<char>,
    ) -> pyo3::PyClassInitializer<Self> {
        let inner = super::enum_::RLike {
            negated,
            expr: Box::new(expr.0),
            pattern: Box::new(pattern.0),
            escape,
        };
        pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self)
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

    fn get_escape(slf: pyo3::PyRef<'_, Self>) -> Option<char> {
        let inner = Self::inner(&slf);
        inner.escape
    }
}

#[pyo3::pymethods]
impl PyCastExpr {
    #[new]
    #[pyo3(signature = (expr, data_type_name, /, *, kind=String::from("CAST"), format=None, at_time_zone=None))]
    fn __new__(
        expr: LiteralExprExtractor,
        data_type_name: DataTypeNameExtractor,
        kind: String,
        format: Option<LiteralExprExtractor>,
        at_time_zone: Option<LiteralExprExtractor>,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let Some(kind) = super::enum_::CastKind::parse(&kind) else {
            return rapidquery_macros::pyerr!(Err(PyValueError), "unknown cast kind {:?}", kind);
        };
        if format.is_none() && at_time_zone.is_some() {
            return rapidquery_macros::pyerr!(Err(PyValueError), "AT TIME ZONE requires FORMAT");
        }

        let inner = super::enum_::Cast {
            kind,
            expr: Box::new(expr.0),
            data_type_name: data_type_name.0,
            format: format.map(|LiteralExprExtractor(e)| Box::new(e)),
            at_time_zone: at_time_zone.map(|LiteralExprExtractor(e)| Box::new(e)),
        };
        Ok(pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self))
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

    fn get_format(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        match inner.format.as_ref() {
            Some(expr) => super::base::into_subclasses(slf.py(), (**expr).clone()),
            None => Ok(utils::new_pynone(slf.py())),
        }
    }

    fn get_at_time_zone(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        match inner.at_time_zone.as_ref() {
            Some(expr) => super::base::into_subclasses(slf.py(), (**expr).clone()),
            None => Ok(utils::new_pynone(slf.py())),
        }
    }
}

#[pyo3::pymethods]
impl PyTrimExpr {
    #[new]
    #[pyo3(signature = (expr, /, *, r#where = None, what = None, characters = None))]
    fn __new__(
        expr: LiteralExprExtractor,
        r#where: Option<&str>,
        what: Option<LiteralExprExtractor>,
        characters: Option<alias::BoundObject<'_>>,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        if what.is_some() && characters.is_some() {
            return rapidquery_macros::pyerr!(
                Err(PyValueError),
                "TRIM what and characters cannot both be set"
            );
        }

        let where_ = match r#where {
            None => None,
            Some(s) => {
                let Some(parsed) = super::enum_::TrimWhere::parse(s) else {
                    return rapidquery_macros::pyerr!(
                        Err(PyValueError),
                        "unknown TRIM where {:?}; expected BOTH, LEADING, or TRAILING",
                        s
                    );
                };
                Some(parsed)
            }
        };

        let characters = match characters {
            None => None,
            Some(obj) => Some(extract_trim_characters(obj.as_borrowed())?),
        };

        let inner = super::enum_::Trim {
            expr: Box::new(expr.0),
            where_,
            what: what.map(|LiteralExprExtractor(e)| Box::new(e)),
            characters,
        };
        Ok(pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self))
    }

    fn get_expr(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        super::base::into_subclasses(slf.py(), (*inner.expr).clone())
    }

    fn get_where(slf: pyo3::PyRef<'_, Self>) -> Option<&'static str> {
        let inner = Self::inner(&slf);
        inner.where_.map(super::enum_::TrimWhere::as_sql)
    }

    fn get_what(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let inner = Self::inner(&slf);
        match inner.what.as_ref() {
            Some(expr) => super::base::into_subclasses(slf.py(), (**expr).clone()),
            None => Ok(utils::new_pynone(slf.py())),
        }
    }

    fn get_characters(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        let py = slf.py();
        let inner = Self::inner(&slf);
        match inner.characters.as_ref() {
            None => Ok(utils::new_pynone(py)),
            Some(chars) => {
                let mut out = Vec::with_capacity(chars.len());
                for c in chars {
                    out.push(super::base::into_subclasses(py, c.clone())?);
                }
                Ok(pyo3::types::PyTuple::new(py, out)?.into_any())
            }
        }
    }
}

#[pyo3::pymethods]
impl PyCollateExpr {
    #[new]
    #[pyo3(signature = (expr, collation, /))]
    fn __new__(
        expr: LiteralExprExtractor,
        collation: IdentExtractor,
    ) -> pyo3::PyClassInitializer<Self> {
        let inner = super::enum_::Collate {
            expr: Box::new(expr.0),
            collation: collation.0,
        };
        pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self)
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
    fn __new__(
        operand: Option<LiteralExprExtractor>,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let inner = super::enum_::Case {
            operand: operand.map(|LiteralExprExtractor(e)| Box::new(e)),
            when: vec![],
            else_result: None,
        };
        Ok(pyo3::PyClassInitializer::from(PyExpr(inner.into())).add_subclass(Self))
    }

    fn when(
        slf: pyo3::PyRef<'_, Self>,
        when: ConditionExtractor,
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

    fn else_(
        slf: pyo3::PyRef<'_, Self>,
        result: LiteralExprExtractor,
    ) -> pyo3::PyResult<alias::PyObject> {
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
}
