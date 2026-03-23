use pyo3::types::PyAnyMethods;

use crate::internal::repr::ReprFormatter;
use crate::internal::type_engine::TypeEngine;
use crate::internal::{BoundObject, RefBoundObject, ToSeaQuery};

crate::implement_pyclass! {
    // NOTE: SQLTypes, PyExpr, PyFunc, PyTableName & PyColumnRef could never mark as subclass.
    // these should be immutable and final types.

    /// Represents a SQL expression that can be built into SQL code.
    ///
    /// This class provides a fluent interface for constructing complex SQL expressions
    /// in a database-agnostic way. It supports arithmetic operations, comparisons,
    /// logical operations, and database-specific functions.
    ///
    /// The class automatically handles SQL injection protection and proper quoting
    /// when building the final SQL statement.
    ///
    /// NOTE: `Expr` is immutable, so by calling each method you will give a new instance
    /// of it which includes new change(s).
    #[derive(Debug, Clone)]
    [] PyExpr as "Expr" (pub sea_query::SimpleExpr);
}

impl PyExpr {
    pub fn try_from_specific_type(
        value: RefBoundObject<'_>,
        type_engine: Option<TypeEngine>,
    ) -> pyo3::PyResult<Self> {
        unsafe {
            let py = value.py();
            let type_ptr = pyo3::ffi::Py_TYPE(value.as_ptr());

            if type_ptr == crate::typeref::EXPR_TYPE {
                let casted_value = value.cast_unchecked::<Self>();

                return Ok(casted_value.get().clone());
            }

            if pyo3::ffi::PyObject_TypeCheck(value.as_ptr(), crate::typeref::VALUE_TYPE) == 1 {
                let casted_value = value.cast_unchecked::<crate::common::value::PyValue>();
                let unbound = casted_value.get();

                return Ok(Self(unbound.0.lock().simple_expr(py)?));
            }

            if type_ptr == crate::typeref::COLUMN_REF_TYPE {
                let casted_value = value.cast_unchecked::<crate::common::column_ref::PyColumnRef>();
                let cloned = casted_value.get().clone();

                return Ok(Self(sea_query::Expr::column(cloned)));
            }

            if type_ptr == crate::typeref::FUNC_TYPE {
                let casted_value = value.cast_unchecked::<super::func::PyFunc>();
                let cloned = casted_value.get().clone();

                return Ok(Self(sea_query::SimpleExpr::FunctionCall(cloned.0)));
            }

            if pyo3::ffi::PyObject_TypeCheck(value.as_ptr(), crate::typeref::SELECT_STATEMENT_TYPE)
                == 1
            {
                let casted_value =
                    value.cast_unchecked::<crate::query::select::PySelectStatement>();

                let inner_value = casted_value.get();
                let result = sea_query::SimpleExpr::SubQuery(
                    None,
                    Box::new(sea_query::SubQueryStatement::SelectStatement(
                        inner_value.0.lock().to_sea_query(py),
                    )),
                );

                return Ok(Self(result));
            }

            if pyo3::ffi::PyObject_TypeCheck(value.as_ptr(), crate::typeref::CASE_STATEMENT_TYPE)
                == 1
            {
                let casted_value = value.cast_unchecked::<crate::query::case::PyCaseStatement>();

                let inner_value = casted_value.get();
                let result =
                    sea_query::SimpleExpr::Case(Box::new(inner_value.0.lock().to_sea_query(py)));

                return Ok(Self(result));
            }

            if pyo3::ffi::PyTuple_Check(value.as_ptr()) == 1 {
                use pyo3::types::PyTupleMethods;

                let casted_value = value.cast_unchecked::<pyo3::types::PyTuple>();
                let mut arr = Vec::with_capacity(casted_value.len());

                for item in casted_value.iter() {
                    arr.push(Self::try_from(&item)?);
                }

                let result = sea_query::Expr::tuple(arr.into_iter().map(|x| x.0));
                return Ok(Self(result.into()));
            }

            if let Some(result) = Self::try_from_property(value)? {
                return Ok(result);
            }

            if let Some(result) = crate::common::column_ref::PyColumnRef::try_from_property(value)?
            {
                return Ok(Self(sea_query::Expr::column(result)));
            }

            let type_engine = match type_engine {
                Some(x) => x,
                None => TypeEngine::infer_pyobject(value)?,
            };

            let result =
                crate::common::value::ValueState::from_pyobject(type_engine, value.clone())?
                    .simple_expr(py)?;

            Ok(Self(result))
        }
    }

    #[inline]
    pub fn try_from_property(value: RefBoundObject) -> pyo3::PyResult<Option<Self>> {
        const PROPERTY_NAME: &std::ffi::CStr = c"__expr__";

        let property = match value.getattr(PROPERTY_NAME) {
            Ok(x) => Ok(Some(x)),
            Err(err) if err.is_instance_of::<pyo3::exceptions::PyAttributeError>(value.py()) => {
                Ok(None)
            }
            Err(err) => Err(err),
        };
        let property = property?;

        if property.is_none() {
            return Ok(None);
        }

        unsafe {
            let property = property.unwrap_unchecked();

            if pyo3::ffi::Py_TYPE(property.as_ptr()) == crate::typeref::EXPR_TYPE {
                let casted_value = property.cast_into_unchecked::<Self>();
                return Ok(Some(casted_value.get().clone()));
            }

            crate::new_error!(
                PyTypeError,
                "__expr__ property returns something other than Expr; returns {}",
                crate::internal::get_type_name(property.py(), property.as_ptr())
            )
        }
    }
}

impl From<sea_query::SimpleExpr> for PyExpr {
    fn from(value: sea_query::SimpleExpr) -> Self {
        Self(value)
    }
}

impl TryFrom<RefBoundObject<'_>> for PyExpr {
    type Error = pyo3::PyErr;

    fn try_from(value: RefBoundObject<'_>) -> Result<Self, Self::Error> {
        Self::try_from_specific_type(value, None)
    }
}

#[pyo3::pymethods]
impl PyExpr {
    #[new]
    #[pyo3(signature = (value, /))]
    fn __new__(value: RefBoundObject<'_>) -> pyo3::PyResult<Self> {
        Self::try_from(value)
    }

    /// Shorthand for `Expr(Value(value, sql_type))`
    #[classmethod]
    #[pyo3(signature=(value, sql_type=None))]
    fn val(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        value: RefBoundObject<'_>,
        sql_type: Option<BoundObject<'_>>,
    ) -> pyo3::PyResult<Self> {
        unsafe {
            if pyo3::ffi::Py_TYPE(value.as_ptr()) == crate::typeref::VALUE_TYPE {
                let casted_value = value.cast_unchecked::<crate::common::value::PyValue>();
                let unbound = casted_value.get();

                return Ok(Self(unbound.0.lock().simple_expr(value.py())?));
            }

            let type_engine = {
                if let Some(sql_type) = sql_type {
                    TypeEngine::new(&sql_type)?
                } else {
                    TypeEngine::infer_pyobject(value)?
                }
            };

            let result =
                crate::common::value::ValueState::from_pyobject(type_engine, value.clone())?
                    .simple_expr(value.py())?;

            Ok(Self(result))
        }
    }

    /// Tries to convert the `value` into `ColumnRef`, and then converts it to `Expr`.
    #[classmethod]
    fn col(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        value: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        let column_ref = crate::common::column_ref::PyColumnRef::try_from(value)?;
        Ok(Self(sea_query::Expr::column(column_ref)))
    }

    /// Returns asterisk '*' expression.
    #[classmethod]
    fn asterisk(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        sea_query::Expr::column(sea_query::Asterisk).into()
    }

    /// Create an expression from a custom SQL string.
    ///
    /// Warning: This method does not escape the input, so it should only
    /// be used with trusted strings to avoid SQL injection vulnerabilities.
    #[classmethod]
    fn custom(_cls: &pyo3::Bound<'_, pyo3::types::PyType>, value: String) -> Self {
        sea_query::SimpleExpr::Custom(value).into()
    }

    /// Create an expression for the CURRENT_DATE SQL function.
    #[classmethod]
    fn current_date(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        sea_query::SimpleExpr::Keyword(sea_query::Keyword::CurrentDate).into()
    }

    /// Create an expression for the CURRENT_TIME SQL function.
    #[classmethod]
    fn current_time(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        sea_query::SimpleExpr::Keyword(sea_query::Keyword::CurrentTime).into()
    }

    /// Create an expression for the CURRENT_TIMESTAMP SQL function.
    #[classmethod]
    fn current_timestamp(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        sea_query::SimpleExpr::Keyword(sea_query::Keyword::CurrentTimestamp).into()
    }

    /// Create an expression representing the NULL value.
    #[classmethod]
    fn null(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        sea_query::SimpleExpr::Keyword(sea_query::Keyword::Null).into()
    }

    /// Express a `EXISTS` sub-query expression.
    #[classmethod]
    fn exists(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        statement: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        unsafe {
            if pyo3::ffi::PyObject_TypeCheck(
                statement.as_ptr(),
                crate::typeref::SELECT_STATEMENT_TYPE,
            ) != 1
            {
                return crate::new_error!(
                    PyTypeError,
                    "expected SelectStatement, got {}",
                    crate::internal::get_type_name(statement.py(), statement.as_ptr())
                );
            }

            let casted_statement =
                statement.cast_unchecked::<crate::query::select::PySelectStatement>();

            let inner_statement = casted_statement.get();

            let result = inner_statement.0.lock().to_sea_query(_cls.py());

            Ok(Self(sea_query::Expr::exists(result)))
        }
    }

    /// Express a `ANY` sub-query expression.
    #[classmethod]
    fn any(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        statement: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        unsafe {
            if pyo3::ffi::PyObject_TypeCheck(
                statement.as_ptr(),
                crate::typeref::SELECT_STATEMENT_TYPE,
            ) != 1
            {
                return crate::new_error!(
                    PyTypeError,
                    "expected SelectStatement, got {}",
                    crate::internal::get_type_name(statement.py(), statement.as_ptr())
                );
            }

            let casted_statement =
                statement.cast_unchecked::<crate::query::select::PySelectStatement>();

            let inner_statement = casted_statement.get();

            let result = inner_statement.0.lock().to_sea_query(_cls.py());

            Ok(Self(sea_query::Expr::any(result)))
        }
    }

    /// Express a `SOME` sub-query expression.
    #[classmethod]
    fn some(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        statement: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        unsafe {
            if pyo3::ffi::PyObject_TypeCheck(
                statement.as_ptr(),
                crate::typeref::SELECT_STATEMENT_TYPE,
            ) != 1
            {
                return crate::new_error!(
                    PyTypeError,
                    "expected SelectStatement, got {}",
                    crate::internal::get_type_name(statement.py(), statement.as_ptr())
                );
            }

            let casted_statement =
                statement.cast_unchecked::<crate::query::select::PySelectStatement>();

            let inner_statement = casted_statement.get();

            let result = inner_statement.0.lock().to_sea_query(_cls.py());

            Ok(Self(sea_query::Expr::some(result)))
        }
    }

    /// Express a `ALL` sub-query expression.
    #[classmethod]
    fn all(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        statement: RefBoundObject<'_>,
    ) -> pyo3::PyResult<Self> {
        unsafe {
            if pyo3::ffi::PyObject_TypeCheck(
                statement.as_ptr(),
                crate::typeref::SELECT_STATEMENT_TYPE,
            ) != 1
            {
                return crate::new_error!(
                    PyTypeError,
                    "expected SelectStatement, got {}",
                    crate::internal::get_type_name(statement.py(), statement.as_ptr())
                );
            }

            let casted_statement =
                statement.cast_unchecked::<crate::query::select::PySelectStatement>();

            let inner_statement = casted_statement.get();
            let result = inner_statement.0.lock().to_sea_query(_cls.py());

            Ok(Self(sea_query::Expr::all(result)))
        }
    }

    /// Express a `IN` expression.
    fn in_(slf: pyo3::PyRef<'_, Self>, other: RefBoundObject<'_>) -> pyo3::PyResult<Self> {
        unsafe {
            if pyo3::ffi::PyObject_TypeCheck(other.as_ptr(), crate::typeref::SELECT_STATEMENT_TYPE)
                == 1
            {
                let casted_statement =
                    other.cast_unchecked::<crate::query::select::PySelectStatement>();

                let inner_statement = casted_statement.get();
                let result = inner_statement.0.lock().to_sea_query(slf.py());
                let result = sea_query::ExprTrait::in_subquery(slf.0.clone(), result);

                return Ok(result.into());
            }
        }

        let mut exprs: Vec<sea_query::SimpleExpr> = Vec::new();

        for item in other.try_iter()? {
            let item = item?;
            let item = Self::try_from(&item)?;
            exprs.push(item.0);
        }

        Ok(sea_query::ExprTrait::is_in(slf.0.clone(), exprs).into())
    }

    /// Express a `NOT IN` expression.
    fn not_in(slf: pyo3::PyRef<'_, Self>, other: RefBoundObject<'_>) -> pyo3::PyResult<Self> {
        unsafe {
            if pyo3::ffi::PyObject_TypeCheck(other.as_ptr(), crate::typeref::SELECT_STATEMENT_TYPE)
                == 1
            {
                let casted_statement =
                    other.cast_unchecked::<crate::query::select::PySelectStatement>();

                let inner_statement = casted_statement.get();
                let result = inner_statement.0.lock().to_sea_query(slf.py());
                let result = sea_query::ExprTrait::not_in_subquery(slf.0.clone(), result);

                return Ok(result.into());
            }
        }

        let mut exprs: Vec<sea_query::SimpleExpr> = Vec::new();

        for item in other.try_iter()? {
            let item = item?;
            let item = Self::try_from(&item)?;
            exprs.push(item.0);
        }

        Ok(sea_query::ExprTrait::is_not_in(slf.0.clone(), exprs).into())
    }

    /// Create a `CAST` expression to convert to a specific SQL type.
    fn cast_as(slf: pyo3::PyRef<'_, Self>, value: String) -> Self {
        slf.0.clone().cast_as(sea_query::Alias::new(value)).into()
    }

    /// Create a `LIKE` pattern matching expression.
    #[pyo3(signature=(pattern, escape=None))]
    fn like(slf: pyo3::PyRef<'_, Self>, pattern: String, escape: Option<char>) -> Self {
        let e = sea_query::LikeExpr::new(pattern);

        if let Some(x) = escape {
            sea_query::ExprTrait::like(slf.0.clone(), e.escape(x)).into()
        } else {
            sea_query::ExprTrait::like(slf.0.clone(), e).into()
        }
    }

    /// Create a NOT LIKE pattern matching expression.
    #[pyo3(signature=(pattern, escape=None))]
    fn not_like(slf: pyo3::PyRef<'_, Self>, pattern: String, escape: Option<char>) -> Self {
        let e = sea_query::LikeExpr::new(pattern);

        if let Some(x) = escape {
            sea_query::ExprTrait::not_like(slf.0.clone(), e.escape(x)).into()
        } else {
            sea_query::ExprTrait::not_like(slf.0.clone(), e).into()
        }
    }

    /// Create an equality comparison expression.
    fn __eq__<'a>(slf: pyo3::PyRef<'a, Self>, other: RefBoundObject<'a>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::eq(slf.0.clone(), other.0).into())
    }

    /// Create an inequality comparison expression.
    fn __ne__<'a>(slf: pyo3::PyRef<'a, Self>, other: RefBoundObject<'a>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::ne(slf.0.clone(), other.0).into())
    }

    /// Create a greater-than comparison expression.
    fn __gt__<'a>(slf: pyo3::PyRef<'a, Self>, other: RefBoundObject<'a>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::gt(slf.0.clone(), other.0).into())
    }

    /// Create a greater-than-or-equal comparison expression.
    fn __ge__<'a>(slf: pyo3::PyRef<'a, Self>, other: RefBoundObject<'a>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::gte(slf.0.clone(), other.0).into())
    }

    /// Create a less-than comparison expression.
    fn __lt__<'a>(slf: pyo3::PyRef<'a, Self>, other: RefBoundObject<'a>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::lt(slf.0.clone(), other.0).into())
    }

    /// Create a less-than-or-equal comparison expression.
    fn __le__<'a>(slf: pyo3::PyRef<'a, Self>, other: RefBoundObject<'a>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::lte(slf.0.clone(), other.0).into())
    }

    /// Create an addition expression.
    fn __add__<'a>(slf: pyo3::PyRef<'a, Self>, other: RefBoundObject<'a>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::add(slf.0.clone(), other.0).into())
    }

    /// Create an subtraction expression.
    fn __sub__<'a>(slf: pyo3::PyRef<'a, Self>, other: RefBoundObject<'a>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::sub(slf.0.clone(), other.0).into())
    }

    /// Create a logical AND expression.
    fn __and__<'a>(slf: pyo3::PyRef<'a, Self>, other: RefBoundObject<'a>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::and(slf.0.clone(), other.0).into())
    }

    fn __neg__<'a>(slf: pyo3::PyRef<'a, Self>) -> pyo3::PyResult<Self> {
        Ok(sea_query::ExprTrait::mul(slf.0.clone(), -1).into())
    }

    /// Create a logical OR expression.
    fn __or__<'a>(slf: pyo3::PyRef<'a, Self>, other: RefBoundObject<'a>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::or(slf.0.clone(), other.0).into())
    }

    fn bit_and<'a>(slf: pyo3::PyRef<'a, Self>, other: RefBoundObject<'a>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::bit_and(slf.0.clone(), other.0).into())
    }

    fn bit_or<'a>(slf: pyo3::PyRef<'a, Self>, other: RefBoundObject<'a>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::bit_or(slf.0.clone(), other.0).into())
    }

    /// Create a division expression.
    fn __truediv__<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: RefBoundObject<'a>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::div(slf.0.clone(), other.0).into())
    }

    /// Create an IS comparison expression.
    fn is_<'a>(slf: pyo3::PyRef<'a, Self>, other: RefBoundObject<'a>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::is(slf.0.clone(), other.0).into())
    }

    /// Create an IS NOT comparison expression.
    fn is_not<'a>(slf: pyo3::PyRef<'a, Self>, other: RefBoundObject<'a>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::is_not(slf.0.clone(), other.0).into())
    }

    /// Create an IS NULL expression.
    fn is_null(slf: pyo3::PyRef<'_, Self>) -> Self {
        sea_query::ExprTrait::is_null(slf.0.clone()).into()
    }

    /// Create an IS NOT NULL expression.
    fn is_not_null(slf: pyo3::PyRef<'_, Self>) -> Self {
        sea_query::ExprTrait::is_not_null(slf.0.clone()).into()
    }

    /// Create a bitwise left shift expression.
    fn __lshift__<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: RefBoundObject<'a>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::left_shift(slf.0.clone(), other.0).into())
    }

    /// Create a bitwise right shift expression.
    fn __rshift__<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: RefBoundObject<'a>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::right_shift(slf.0.clone(), other.0).into())
    }

    /// Create a modulo expression.
    fn __mod__<'a>(slf: pyo3::PyRef<'a, Self>, other: RefBoundObject<'a>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::modulo(slf.0.clone(), other.0).into())
    }

    /// Create a multiplication expression.
    fn __mul__<'a>(slf: pyo3::PyRef<'a, Self>, other: RefBoundObject<'a>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::mul(slf.0.clone(), other.0).into())
    }

    /// Create a BETWEEN range comparison expression.
    fn between<'a>(
        slf: pyo3::PyRef<'a, Self>,
        a: RefBoundObject<'a>,
        b: RefBoundObject<'a>,
    ) -> pyo3::PyResult<Self> {
        let a = Self::try_from(a)?;
        let b = Self::try_from(b)?;

        Ok(sea_query::ExprTrait::between(slf.0.clone(), a.0, b.0).into())
    }

    /// Create a NOT BETWEEN range comparison expression.
    fn not_between<'a>(
        slf: pyo3::PyRef<'a, Self>,
        a: RefBoundObject<'a>,
        b: RefBoundObject<'a>,
    ) -> pyo3::PyResult<Self> {
        let a = Self::try_from(a)?;
        let b = Self::try_from(b)?;

        Ok(sea_query::ExprTrait::not_between(slf.0.clone(), a.0, b.0).into())
    }

    #[pyo3(signature = (backend, /))]
    #[allow(clippy::wrong_self_convention)]
    fn _to_sql(&self, _py: pyo3::Python<'_>, backend: String) -> pyo3::PyResult<String> {
        let builder = crate::internal::get_schema_builder(backend)?;
        let mut sql = String::new();

        let assert_unwind = std::panic::AssertUnwindSafe(|| {
            sea_query::QueryBuilder::prepare_simple_expr(&*builder, &self.0, &mut sql)
        });

        std::panic::catch_unwind(assert_unwind)
            .map_err(|_| pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("build failed"))?;

        Ok(sql)
    }

    // TODO: sqlite_*, pg_*, mysql_*
    // TODO: get_column_ref and is_column_ref
    // TODO: get_value and is_value
    // TODO: get_func and is_func
    // TODO: max and min and abs

    pub fn __repr__(&self) -> String {
        #[cfg(not(debug_assertions))]
        {
            ReprFormatter::new("Expr").pair("", "...").finish()
        }

        #[cfg(debug_assertions)]
        {
            ReprFormatter::new("Expr").debug("", &self.0).finish()
        }
    }
}
