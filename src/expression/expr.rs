use crate::sqltypes::TypeEngine;

implement_pyclass! {
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
    pub struct [] PyExpr as "Expr" (pub sea_query::SimpleExpr);
}

impl PyExpr {
    pub fn try_from_specific_type(
        value: &pyo3::Bound<'_, pyo3::PyAny>,
        type_engine: Option<TypeEngine>,
    ) -> pyo3::PyResult<Self> {
        unsafe {
            let py = value.py();
            let type_ptr = pyo3::ffi::Py_TYPE(value.as_ptr());

            if type_ptr == crate::typeref::EXPR_TYPE {
                let casted_value = value.cast_unchecked::<Self>();

                return Ok(Self(casted_value.get().0.clone()));
            }

            if type_ptr == crate::typeref::VALUE_TYPE {
                let casted_value = value.cast_unchecked::<crate::value::PyValue>();
                let unbound = casted_value.get();

                return Ok(Self(unbound.0.lock().simple_expr(py)?));
            }

            if type_ptr == crate::typeref::ASTERISK_TYPE {
                return Ok(Self(sea_query::Expr::column(sea_query::Asterisk)));
            }

            if type_ptr == crate::typeref::COLUMN_REF_TYPE {
                let casted_value = value.cast_unchecked::<crate::common::PyColumnRef>();
                let cloned = casted_value.get().clone();

                return Ok(Self(sea_query::Expr::column(cloned)));
            }

            if type_ptr == crate::typeref::FUNC_TYPE {
                let casted_value = value.cast_unchecked::<super::func::PyFunc>();
                let cloned = casted_value.get().clone();

                return Ok(Self(sea_query::SimpleExpr::FunctionCall(cloned.0)));
            }

            if type_ptr == crate::typeref::COLUMN_TYPE {
                let casted_value = value.cast_unchecked::<crate::column::PyColumn>();
                let inner_value = casted_value.get();

                return Ok(Self(inner_value.0.lock().to_sea_query_column_ref().into()));
            }

            // TODO: PySelect
            // TODO: PyCase

            if pyo3::ffi::PyTuple_CheckExact(value.as_ptr()) == 1 {
                use pyo3::types::PyTupleMethods;

                let casted_value = value.cast_unchecked::<pyo3::types::PyTuple>();
                let mut arr: Vec<Self> = Vec::new();

                for item in casted_value.iter() {
                    arr.push(Self::try_from(&item)?);
                }

                let result = sea_query::Expr::tuple(arr.into_iter().map(|x| x.0));
                return Ok(Self(result.into()));
            }

            let type_engine = match type_engine {
                Some(x) => x,
                None => crate::sqltypes::TypeEngine::infer_pyobject(value)?,
            };

            let result = crate::value::ValueState::from_pyobject(type_engine, value.clone())?
                .simple_expr(py)?;

            Ok(Self(result))
        }
    }
}

impl From<sea_query::SimpleExpr> for PyExpr {
    fn from(value: sea_query::SimpleExpr) -> Self {
        Self(value)
    }
}

impl TryFrom<&pyo3::Bound<'_, pyo3::PyAny>> for PyExpr {
    type Error = pyo3::PyErr;

    fn try_from(value: &pyo3::Bound<'_, pyo3::PyAny>) -> Result<Self, Self::Error> {
        Self::try_from_specific_type(value, None)
    }
}

#[pyo3::pymethods]
impl PyExpr {
    #[new]
    #[pyo3(signature = (value, /))]
    fn __new__(value: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        Self::try_from(value)
    }

    /// Shorthand for `Expr(Value(value, sql_type))`
    ///
    /// @typevar I, O
    /// @signature (cls, /, value: I | None, sql_type: SQLTypeAbstract[I, O] | None = ...) -> typing.Self
    #[classmethod]
    #[pyo3(signature=(value, sql_type=None))]
    fn val(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        value: &pyo3::Bound<'_, pyo3::PyAny>,
        sql_type: Option<pyo3::Bound<'_, pyo3::PyAny>>,
    ) -> pyo3::PyResult<Self> {
        unsafe {
            if pyo3::ffi::Py_TYPE(value.as_ptr()) == crate::typeref::VALUE_TYPE {
                let casted_value = value.cast_unchecked::<crate::value::PyValue>();
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

            let result = crate::value::ValueState::from_pyobject(type_engine, value.clone())?
                .simple_expr(value.py())?;

            Ok(Self(result))
        }
    }

    /// Shorthand for `Expr(ColumnRef.parse(value))`
    ///
    /// @signature (cls, value: str | ColumnRef) -> typing.Self
    #[classmethod]
    fn col(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        value: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        use pyo3::types::PyAnyMethods;
        use std::str::FromStr;

        unsafe {
            if pyo3::ffi::Py_TYPE(value.as_ptr()) == crate::typeref::COLUMN_REF_TYPE {
                let casted_value = value.cast_unchecked::<crate::common::PyColumnRef>();
                let cloned = casted_value.get().clone();

                return Ok(Self(sea_query::Expr::column(cloned)));
            }

            if pyo3::ffi::PyUnicode_CheckExact(value.as_ptr()) == 1 {
                let extracted = value.extract::<&str>().unwrap_unchecked();
                let colref = crate::common::PyColumnRef::from_str(extracted)?;

                return Ok(Self(sea_query::Expr::column(colref)));
            }

            Err(typeerror!(
                "expected ColumnRef or str, got {}",
                value.py(),
                value.as_ptr()
            ))
        }
    }

    /// Shorthand for `Expr(ASTERISK)`
    ///
    /// @signature (cls) -> typing.Self
    #[classmethod]
    fn asterisk(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        sea_query::Expr::column(sea_query::Asterisk).into()
    }

    /// Create an expression from a custom SQL string.
    ///
    /// Warning: This method does not escape the input, so it should only
    /// be used with trusted strings to avoid SQL injection vulnerabilities.
    ///
    /// @signature (cls, value: str) -> typing.Self
    #[classmethod]
    fn custom(_cls: &pyo3::Bound<'_, pyo3::types::PyType>, value: String) -> Self {
        sea_query::SimpleExpr::Custom(value).into()
    }

    /// Create an expression for the CURRENT_DATE SQL function.
    ///
    /// @signature (cls) -> typing.Self
    #[classmethod]
    fn current_date(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        sea_query::SimpleExpr::Keyword(sea_query::Keyword::CurrentDate).into()
    }

    /// Create an expression for the CURRENT_TIME SQL function.
    ///
    /// @signature (cls) -> typing.Self
    #[classmethod]
    fn current_time(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        sea_query::SimpleExpr::Keyword(sea_query::Keyword::CurrentTime).into()
    }

    /// Create an expression for the CURRENT_TIMESTAMP SQL function.
    ///
    /// @signature (cls) -> typing.Self
    #[classmethod]
    fn current_timestamp(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        sea_query::SimpleExpr::Keyword(sea_query::Keyword::CurrentTimestamp).into()
    }

    /// Create an expression representing the NULL value.
    ///
    /// @signature (cls) -> typing.Self
    #[classmethod]
    fn null(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        sea_query::SimpleExpr::Keyword(sea_query::Keyword::Null).into()
    }

    // TODO: exists, any, some, all, in_subquery, not_in_subquery, in_, not_in

    /// Create a CAST expression to convert to a specific SQL type.
    ///
    /// @signature (self, value: str) -> typing.Self
    fn cast_as(slf: pyo3::PyRef<'_, Self>, value: String) -> Self {
        slf.0.clone().cast_as(sea_query::Alias::new(value)).into()
    }

    /// Create a LIKE pattern matching expression.
    ///
    /// @signature (self, pattern: str, escape: str | None = ...) -> typing.Self
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
    ///
    /// @signature (self, pattern: str, escape: str | None = ...) -> typing.Self
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
    ///
    /// @signature (self, other: object) -> typing.Self
    fn __eq__<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::eq(slf.0.clone(), other.0).into())
    }

    /// Create an inequality comparison expression.
    ///
    /// @signature (self, other: object) -> typing.Self
    fn __ne__<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::ne(slf.0.clone(), other.0).into())
    }

    /// Create a greater-than comparison expression.
    ///
    /// @signature (self, other: object) -> typing.Self
    fn __gt__<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::gt(slf.0.clone(), other.0).into())
    }

    /// Create a greater-than-or-equal comparison expression.
    ///
    /// @signature (self, other: object) -> typing.Self
    fn __ge__<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::gte(slf.0.clone(), other.0).into())
    }

    /// Create a less-than comparison expression.
    ///
    /// @signature (self, other: object) -> typing.Self
    fn __lt__<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::lt(slf.0.clone(), other.0).into())
    }

    /// Create a less-than-or-equal comparison expression.
    ///
    /// @signature (self, other: object) -> typing.Self
    fn __le__<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::lte(slf.0.clone(), other.0).into())
    }

    /// Create an addition expression.
    ///
    /// @signature (self, other: object) -> typing.Self
    fn __add__<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::add(slf.0.clone(), other.0).into())
    }

    /// Create an subtraction expression.
    ///
    /// @signature (self, other: object) -> typing.Self
    fn __sub__<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::sub(slf.0.clone(), other.0).into())
    }

    /// Create a logical AND expression.
    ///
    /// @signature (self, other: object) -> typing.Self
    fn __and__<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::and(slf.0.clone(), other.0).into())
    }

    /// @signature (self, other: object) -> typing.Self
    fn __neg__<'a>(slf: pyo3::PyRef<'a, Self>) -> pyo3::PyResult<Self> {
        Ok(sea_query::ExprTrait::mul(slf.0.clone(), -1).into())
    }

    /// Create a logical OR expression.
    ///
    /// @signature (self, other: object) -> typing.Self
    fn __or__<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::or(slf.0.clone(), other.0).into())
    }

    /// @signature (self, other: object) -> typing.Self
    fn bit_and<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::bit_and(slf.0.clone(), other.0).into())
    }

    /// @signature (self, other: object) -> typing.Self
    fn bit_or<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::bit_or(slf.0.clone(), other.0).into())
    }

    /// Create a division expression.
    ///
    /// @signature (self, other: object) -> typing.Self
    fn __truediv__<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::div(slf.0.clone(), other.0).into())
    }

    /// Create an IS comparison expression.
    ///
    /// @signature (self, other: object) -> typing.Self
    fn is_<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::is(slf.0.clone(), other.0).into())
    }

    /// Create an IS NOT comparison expression.
    ///
    /// @signature (self, other: object) -> typing.Self
    fn is_not<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::is_not(slf.0.clone(), other.0).into())
    }

    /// Create an IS NULL expression.
    ///
    /// @signature (self) -> typing.Self
    fn is_null(slf: pyo3::PyRef<'_, Self>) -> Self {
        sea_query::ExprTrait::is_null(slf.0.clone()).into()
    }

    /// Create an IS NOT NULL expression.
    ///
    /// @signature (self) -> typing.Self
    fn is_not_null(slf: pyo3::PyRef<'_, Self>) -> Self {
        sea_query::ExprTrait::is_not_null(slf.0.clone()).into()
    }

    /// Create a bitwise left shift expression.
    ///
    /// @signature (self, other: object) -> typing.Self
    fn __lshift__<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::left_shift(slf.0.clone(), other.0).into())
    }

    /// Create a bitwise right shift expression.
    ///
    /// @signature (self, other: object) -> typing.Self
    fn __rshift__<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::right_shift(slf.0.clone(), other.0).into())
    }

    /// Create a modulo expression.
    ///
    /// @signature (self, other: object) -> typing.Self
    fn __mod__<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::modulo(slf.0.clone(), other.0).into())
    }

    /// Create a multiplication expression.
    ///
    /// @signature (self, other: object) -> typing.Self
    fn __mul__<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other)?;
        Ok(sea_query::ExprTrait::mul(slf.0.clone(), other.0).into())
    }

    // TODO: sqlite_*, pg_*, mysql_*

    /// Create a BETWEEN range comparison expression.
    ///
    /// @signature (self, a: object, b: object) -> typing.Self
    fn between<'a>(
        slf: pyo3::PyRef<'a, Self>,
        a: &pyo3::Bound<'a, pyo3::PyAny>,
        b: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let a = Self::try_from(a)?;
        let b = Self::try_from(b)?;

        Ok(sea_query::ExprTrait::between(slf.0.clone(), a.0, b.0).into())
    }

    /// Create a NOT BETWEEN range comparison expression.
    ///
    /// @signature (self, a: object, b: object) -> typing.Self
    fn not_between<'a>(
        slf: pyo3::PyRef<'a, Self>,
        a: &pyo3::Bound<'a, pyo3::PyAny>,
        b: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let a = Self::try_from(a)?;
        let b = Self::try_from(b)?;

        Ok(sea_query::ExprTrait::not_between(slf.0.clone(), a.0, b.0).into())
    }

    pub fn __repr__(&self) -> String {
        format!("<Expr {:?}>", self.0)
    }
}
