use pyo3::types::PyTupleMethods;
use sea_query::IntoIden;

use crate::internal::repr::ReprFormatter;
use crate::internal::{BoundArgs, RefBoundObject};

crate::implement_pyclass! {
    // NOTE: SQLTypes, PyExpr, PyFunc, PyTableName & PyColumnRef could never mark as subclass.
    // these should be immutable and final types.

    /// Represents a SQL function call that can be used in expressions.
    ///
    /// This class provides a type-safe way to construct SQL function calls
    /// with proper argument handling and database dialect support.
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
    #[classmethod]
    fn now(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self(sea_query::Func::cust(sea_query::Alias::new("NOW")))
    }

    /// Create a SUM(expr) function call.
    #[classmethod]
    fn sum(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::sum(expr.0)))
    }

    /// Create a MIN(expr) function call.
    #[classmethod]
    fn min(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::min(expr.0)))
    }

    /// Create a MAX(expr) function call.
    #[classmethod]
    fn max(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::max(expr.0)))
    }

    /// Create a ABS(expr) function call.
    #[classmethod]
    fn abs(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::abs(expr.0)))
    }

    /// Create a AVG(expr) function call.
    #[classmethod]
    fn avg(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::avg(expr.0)))
    }

    /// Create a COUNT(expr) function call.
    #[classmethod]
    fn count(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::count(expr.0)))
    }

    /// Create a COUNT(DISTINCT expr) function call.
    #[classmethod]
    fn count_distinct(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::count_distinct(expr.0)))
    }

    /// Create a IF_NULL(a, b) function call.
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
    #[classmethod]
    fn char_length(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::char_length(expr.0)))
    }

    /// Create a COALESCE function call.
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
    #[classmethod]
    fn lower(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::lower(expr.0)))
    }

    /// Create a UPPER(expr) function call.
    #[classmethod]
    fn upper(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::upper(expr.0)))
    }

    /// Create a BIT_AND(expr) function call - this is not supported on SQLite.
    #[classmethod]
    fn bit_and(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::bit_and(expr.0)))
    }

    /// Create a BIT_OR(expr) function call - this is not supported on SQLite.
    #[classmethod]
    fn bit_or(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::bit_or(expr.0)))
    }

    /// Create a RANDOM() function call.
    #[classmethod]
    fn random(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self(sea_query::Func::random())
    }

    /// Create a RANK() function call.
    #[classmethod]
    fn rank(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self(sea_query::Func::cust("RANK"))
    }

    /// Create a DENSE_RANK() function call.
    #[classmethod]
    fn dense_rank(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self(sea_query::Func::cust("DENSE_RANK"))
    }

    /// Create a PERCENT_RANK() function call.
    #[classmethod]
    fn percent_rank(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self(sea_query::Func::cust("PERCENT_RANK"))
    }

    /// Create a ROUND(expr) function call.
    #[classmethod]
    fn round(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::round(expr.0)))
    }

    /// Create a ROUND(a, b) function call.
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
    #[classmethod]
    fn md5(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::md5(expr.0)))
    }

    #[classmethod]
    fn cast_as(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: RefBoundObject<'_>,
        alias: String,
    ) -> pyo3::PyResult<Self> {
        let expr = super::expr::PyExpr::try_from(expr)?;
        Ok(Self(sea_query::Func::cast_as(
            expr.0,
            sea_query::Alias::new(alias).into_iden(),
        )))
    }

    /// Shorthand for `SelectLabel(self, alias, window)`
    #[pyo3(signature=(alias, window=None))]
    fn label(
        &self,
        alias: String,
        window: Option<RefBoundObject<'_>>,
    ) -> pyo3::PyResult<crate::query::select::PySelectLabel> {
        let window = match window {
            Some(x) => Some(crate::query::select::SelectLabelWindow::try_from(x)?),
            None => None,
        };
        let expr = self.to_expr();

        let state = crate::query::select::SelectLabelState {
            expr,
            alias: Some(alias),
            window,
        };
        Ok(state.into())
    }

    /// Shorthand for `Expr(self)`
    fn to_expr(&self) -> super::expr::PyExpr {
        super::expr::PyExpr(sea_query::SimpleExpr::FunctionCall(self.0.clone()))
    }

    pub fn __repr__(&self) -> String {
        #[cfg(not(debug_assertions))]
        {
            ReprFormatter::new("Func").pair("", "...").finish()
        }

        #[cfg(debug_assertions)]
        {
            ReprFormatter::new("Func").debug("", &self.0).finish()
        }
    }
}
