use sea_query::IntoColumnRef;

use crate::common::column_ref::PyColumnRef;
use crate::common::expression::PyExpr;
use crate::internal::BoundObject;

#[inline]
fn map_order_to_str(order: &sea_query::Order) -> String {
    match order {
        sea_query::Order::Asc => String::from("ASC"),
        sea_query::Order::Desc => String::from("DESC"),
        _ => unsafe { std::hint::unreachable_unchecked() },
    }
}

#[inline]
fn map_null_ordering_to_str(order: Option<sea_query::NullOrdering>) -> Option<String> {
    match order {
        Some(sea_query::NullOrdering::First) => Some(String::from("FIRST")),
        Some(sea_query::NullOrdering::Last) => Some(String::from("LAST")),
        None => None,
    }
}

crate::implement_pyclass! {
    // NOTE: It's a very simple clause, so I think it's OK to be a final type.

    /// Specifies ordering behavior for UPDATE, DELETE, and SELECT statements.
    ///
    /// @signature (
    ///     target: Expr | Column | ColumnRef | str,
    ///     order: typing.Literal["ASC", "DESC"] = "ASC",
    ///     null_ordering: typing.Literal["FIRST", "LAST"] | None = None,
    /// )
    #[derive(Debug, Clone)]
    [] PyOrdering as "Ordering" {
        pub target: PyExpr,
        pub order: sea_query::Order,
        pub null_order: Option<sea_query::NullOrdering>,
    }
}

#[pyo3::pymethods]
impl PyOrdering {
    #[new]
    #[pyo3(signature=(target, order = String::from("ASC"), null_order=None))]
    fn __new__(
        target: BoundObject<'_>,
        order: String,
        null_order: Option<String>,
    ) -> pyo3::PyResult<Self> {
        let target = unsafe {
            if pyo3::ffi::Py_TYPE(target.as_ptr()) == crate::typeref::EXPR_TYPE {
                target.cast_unchecked::<PyExpr>().get().clone()
            } else {
                let column_ref = PyColumnRef::try_from(&target)?;

                PyExpr(sea_query::SimpleExpr::Column(column_ref.into_column_ref()))
            }
        };

        let order = match order.to_ascii_lowercase().as_str() {
            "asc" => sea_query::Order::Asc,
            "desc" => sea_query::Order::Desc,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "unknown order: {order}"
                )))
            }
        };

        let null_order = match null_order {
            None => None,
            Some(x) => match x.to_ascii_lowercase().as_str() {
                "first" => Some(sea_query::NullOrdering::First),
                "last" => Some(sea_query::NullOrdering::Last),
                _ => {
                    return Err(pyo3::exceptions::PyValueError::new_err(format!(
                        "unknown null_order: {x}"
                    )))
                }
            },
        };

        Ok(Self {
            target,
            order,
            null_order,
        })
    }

    /// Target expression.
    ///
    /// @signature (self) -> Expr
    #[getter]
    fn target(&self) -> PyExpr {
        self.target.clone()
    }

    /// @signature (self) -> typing.Literal["ASC", "DESC"]
    #[getter]
    fn order(&self) -> String {
        map_order_to_str(&self.order)
    }

    /// @signature (self) -> typing.Literal["FIRST", "LAST"] | None
    #[getter]
    fn null_order(&self) -> Option<String> {
        map_null_ordering_to_str(self.null_order)
    }

    pub fn __repr__(&self) -> String {
        use std::io::Write;

        let mut s = Vec::<u8>::with_capacity(20);

        write!(
            s,
            "<OrderingClause {} {}",
            self.target.__repr__(),
            map_order_to_str(&self.order)
        )
        .unwrap();

        if let Some(x) = map_null_ordering_to_str(self.null_order) {
            write!(s, " NULLS {x}").unwrap();
        }

        write!(s, ">").unwrap();
        unsafe { String::from_utf8_unchecked(s) }
    }
}
