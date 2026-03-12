use pyo3::types::PyTupleMethods;

use crate::internal::{BoundArgs, RefBoundObject};

crate::implement_pyclass! {
    // NOTE: SQLTypes, PyExpr, PyFunc, PyTableName & PyColumnRef could never mark as subclass.
    // these should be immutable and final types.

    /// Represents a SQL function call that can be used in expressions.
    ///
    /// This class provides a type-safe way to construct SQL function calls
    /// with proper argument handling and database dialect support.
    ///
    /// @signature (name: str, *args: object)
    #[derive(Debug, Clone)]
    [] PyFunc as "Func" (pub sea_query::FunctionCall);
}

#[pyo3::pymethods]
impl PyFunc {
    #[new]
    #[pyo3(signature=(name, *args))]
    pub fn __new__(name: String, args: BoundArgs<'_>) -> pyo3::PyResult<Self> {
        let mut function_call = sea_query::Func::cust(sea_query::Alias::new(name));

        for item in args.iter() {
            let expr = super::expr::PyExpr::try_from(&item)?;
            function_call = function_call.arg(expr.0);
        }

        Ok(Self(function_call))
    }

    /// Create a NOW() function call.
    ///
    /// @signature (cls) -> typing.Self
    #[classmethod]
    fn now(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self(sea_query::Func::cust(sea_query::Alias::new("NOW")))
    }

    /// Create a SUM(expr) function call.
    ///
    /// @signature (cls, /, expr: object) -> typing.Self
    #[classmethod]
    fn sum(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::sum(expr.0)))
    }

    /// Create a MIN(expr) function call.
    ///
    /// @signature (cls, /, expr: object) -> typing.Self
    #[classmethod]
    fn min(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::min(expr.0)))
    }

    /// Create a MAX(expr) function call.
    ///
    /// @signature (cls, /, expr: object) -> typing.Self
    #[classmethod]
    fn max(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::max(expr.0)))
    }

    /// Create a ABS(expr) function call.
    ///
    /// @signature (cls, /, expr: object) -> typing.Self
    #[classmethod]
    fn abs(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::abs(expr.0)))
    }

    /// Create a AVG(expr) function call.
    ///
    /// @signature (cls, /, expr: object) -> typing.Self
    #[classmethod]
    fn avg(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::avg(expr.0)))
    }

    /// Create a COUNT(expr) function call.
    ///
    /// @signature (cls, /, expr: object) -> typing.Self
    #[classmethod]
    fn count(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::count(expr.0)))
    }

    /// Create a COUNT(DISTINCT expr) function call.
    ///
    /// @signature (cls, /, expr: object) -> typing.Self
    #[classmethod]
    fn count_distinct(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::count_distinct(expr.0)))
    }

    /// Create a IF_NULL(a, b) function call.
    ///
    /// @signature (cls, /, a: object, b: object) -> typing.Self
    #[classmethod]
    fn if_null(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        a: RefBoundObject<'_>,
        b: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let a = super::expr::PyExpr::try_from(a)?;
        let b = super::expr::PyExpr::try_from(b)?;

        Ok(Self(sea_query::Func::if_null(a.0, b.0)))
    }

    /// Create a GREATEST function call.
    ///
    /// @signature (cls, /, *exprs: object) -> typing.Self
    #[classmethod]
    #[pyo3(signature=(*exprs))]
    fn greatest(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        exprs: BoundArgs<'_>,
    ) -> pyo3::PyResult<Self> {
        let mut simple_exprs = Vec::with_capacity(exprs.len());

        for expr in exprs.iter() {
            let expr = super::expr::PyExpr::try_from(&expr)?;
            simple_exprs.push(expr.0);
        }

        Ok(Self(sea_query::Func::greatest(simple_exprs)))
    }

    /// Create a LEAST function call.
    ///
    /// @signature (cls, /, *exprs: object) -> typing.Self
    #[classmethod]
    #[pyo3(signature=(*exprs))]
    fn least(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        exprs: BoundArgs<'_>,
    ) -> pyo3::PyResult<Self> {
        let mut simple_exprs = Vec::with_capacity(exprs.len());

        for expr in exprs.iter() {
            let expr = super::expr::PyExpr::try_from(&expr)?;
            simple_exprs.push(expr.0);
        }

        Ok(Self(sea_query::Func::least(simple_exprs)))
    }

    /// Create a CHAR_LENGTH(expr) function call.
    ///
    /// @signature (cls, /, expr: object) -> typing.Self
    #[classmethod]
    fn char_length(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::char_length(expr.0)))
    }

    /// Create a COALESCE function call.
    ///
    /// @signature (cls, /, *exprs: object) -> typing.Self
    #[classmethod]
    #[pyo3(signature=(*exprs))]
    fn coalesce(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        exprs: BoundArgs<'_>,
    ) -> pyo3::PyResult<Self> {
        let mut simple_exprs = Vec::with_capacity(exprs.len());

        for expr in exprs.iter() {
            let expr = super::expr::PyExpr::try_from(&expr)?;
            simple_exprs.push(expr.0);
        }

        Ok(Self(sea_query::Func::coalesce(simple_exprs)))
    }

    /// Create a LOWER(expr) function call.
    ///
    /// @signature (cls, /, expr: object) -> typing.Self
    #[classmethod]
    fn lower(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::lower(expr.0)))
    }

    /// Create a UPPER(expr) function call.
    ///
    /// @signature (cls, /, expr: object) -> typing.Self
    #[classmethod]
    fn upper(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::upper(expr.0)))
    }

    /// Create a BIT_AND(expr) function call - this is not supported on SQLite.
    ///
    /// @signature (cls, /, expr: object) -> typing.Self
    #[classmethod]
    fn bit_and(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::bit_and(expr.0)))
    }

    /// Create a BIT_OR(expr) function call - this is not supported on SQLite.
    ///
    /// @signature (cls, /, expr: object) -> typing.Self
    #[classmethod]
    fn bit_or(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::bit_or(expr.0)))
    }

    /// Create a RANDOM() function call.
    ///
    /// @signature (cls, /) -> typing.Self
    #[classmethod]
    fn random(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self(sea_query::Func::random())
    }

    /// Create a RANK() function call.
    ///
    /// @signature (cls, /) -> typing.Self
    #[classmethod]
    fn rank(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self(sea_query::Func::cust("RANK"))
    }

    /// Create a DENSE_RANK() function call.
    ///
    /// @signature (cls, /) -> typing.Self
    #[classmethod]
    fn dense_rank(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self(sea_query::Func::cust("DENSE_RANK"))
    }

    /// Create a PERCENT_RANK() function call.
    ///
    /// @signature (cls, /) -> typing.Self
    #[classmethod]
    fn percent_rank(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self(sea_query::Func::cust("PERCENT_RANK"))
    }

    /// Create a ROUND(expr) function call.
    ///
    /// @signature (cls, /, expr: object) -> typing.Self
    #[classmethod]
    fn round(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::round(expr.0)))
    }

    /// Create a ROUND(a, b) function call.
    ///
    /// @signature (cls, /, a: object, b: object) -> typing.Self
    #[classmethod]
    fn round_with_precision(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        a: RefBoundObject<'_>,
        b: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let a = super::expr::PyExpr::try_from(a)?;
        let b = super::expr::PyExpr::try_from(b)?;

        Ok(Self(sea_query::Func::round_with_precision(a.0, b.0)))
    }

    /// Create a MD5(expr) function call - this is only available in Postgres and MySQL.
    ///
    /// @signature (cls, /, expr: object) -> typing.Self
    #[classmethod]
    fn md5(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::md5(expr.0)))
    }

    fn __repr__(&self) -> String {
        format!("<Func {:?}>", self.0)
    }
}
