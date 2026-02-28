use pyo3::types::PyTupleMethods;
use sea_query::IntoColumnRef;

use crate::{common::PyColumnRef, expression::PyExpr};

#[derive(Debug, Default)]
pub enum ReturningClause {
    #[default]
    None,
    All,
    Columns(Vec<String>),
}

#[derive(Debug)]
pub struct OrderClause {
    pub target: PyExpr,
    pub order: sea_query::Order,
    pub null_order: Option<sea_query::NullOrdering>,
}

impl ReturningClause {
    #[inline]
    pub fn new(args: pyo3::Bound<'_, pyo3::types::PyTuple>) -> pyo3::PyResult<Self> {
        let mut columns = Vec::with_capacity(args.len());

        for col in args.iter() {
            let column_ref = crate::common::PyColumnRef::try_from(&col)?;

            match column_ref.name {
                Some(x) => columns.push(x.to_string()),
                None => {
                    return Ok(ReturningClause::All);
                }
            }
        }

        Ok(ReturningClause::Columns(columns))
    }
}

#[inline]
fn map_str_to_order(value: impl AsRef<str>) -> pyo3::PyResult<sea_query::Order> {
    match value.as_ref() {
        "asc" | "ASC" => Ok(sea_query::Order::Asc),
        "desc" | "DESC" => Ok(sea_query::Order::Desc),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "unknown order: {}",
            value.as_ref()
        ))),
    }
}

#[inline]
fn map_str_to_null_ordering(value: impl AsRef<str>) -> pyo3::PyResult<sea_query::NullOrdering> {
    match value.as_ref() {
        "first" | "FIRST" => Ok(sea_query::NullOrdering::First),
        "last" | "LAST" => Ok(sea_query::NullOrdering::Last),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "unknown null ordering: {}",
            value.as_ref()
        ))),
    }
}

impl OrderClause {
    #[inline]
    pub fn new(
        target: &pyo3::Bound<'_, pyo3::PyAny>,
        order: String,
        null_order: Option<String>,
    ) -> pyo3::PyResult<Self> {
        let order = map_str_to_order(order)?;

        let null_order = match null_order {
            Some(x) => Some(map_str_to_null_ordering(x)?),
            None => None,
        };

        unsafe {
            if pyo3::ffi::Py_TYPE(target.as_ptr()) == crate::typeref::EXPR_TYPE {
                return Ok(Self {
                    target: target.cast_unchecked::<PyExpr>().get().clone(),
                    order,
                    null_order,
                });
            }
        }

        let column_ref = PyColumnRef::try_from(target)?;

        Ok(Self {
            target: PyExpr(sea_query::SimpleExpr::Column(column_ref.into_column_ref())),
            order,
            null_order,
        })
    }
}

impl std::fmt::Display for OrderClause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.order {
            sea_query::Order::Asc => {
                write!(f, "{} ASC", self.target.__repr__())?;
            }
            sea_query::Order::Desc => {
                write!(f, "{} DESC", self.target.__repr__())?;
            }
            _ => unreachable!(),
        }

        if let Some(null_order) = self.null_order {
            match null_order {
                sea_query::NullOrdering::First => {
                    write!(f, "NULLS FIRST")?;
                }
                sea_query::NullOrdering::Last => {
                    write!(f, "NULLS LAST")?;
                }
            }
        }

        Ok(())
    }
}
