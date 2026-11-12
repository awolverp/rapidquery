use std::str::FromStr;

use crate::ddl::common;
use crate::exc;
use crate::expr::enum_::ExprInner;
use crate::expr::extractors::NameExprExtractor;
use crate::internal::alias;
use crate::internal::repr;
use crate::sql;

/// One key of a `CREATE INDEX` column list.
#[rapidquery_macros::pyclass("rapidquery.ddl.IndexColumn")]
#[derive(Debug, Clone)]
pub struct PyIndexColumn {
    /// Index column expression.
    pub column: ExprInner,

    /// Index column order.
    pub order: common::Order,
}

impl PyIndexColumn {
    /// Returns Python `__repr__` result
    #[inline]
    pub fn python_repr(&self, py: pyo3::Python<'_>) -> exc::Result<repr::ReprFormatter> {
        let mut fmt = repr::ReprFormatter::new_call("IndexColumn");

        fmt.positional(self.column.python_repr(py)?);
        if matches!(self.order, common::Order::Desc) {
            fmt.keyword("order", "DESC");
        }

        Ok(fmt)
    }
}

impl sql::WriteSql for PyIndexColumn {
    #[inline]
    fn write_sql<W: sql::SqlWriter>(&self, dialect: &crate::dialect::PyDialect, out: &mut W) -> exc::Result<()> {
        let column_order = dialect
            .index
            .features
            .contains(crate::dialect::IndexFeatures::COLUMN_ORDER);

        if !column_order && matches!(self.order, common::Order::Desc) {
            return Err(exc::Error::dialect("CREATE INDEX ... ASC/DESC"));
        }

        match &self.column {
            ExprInner::Ident(ident) => ident.write_sql(dialect, out)?,
            other => {
                if !dialect
                    .index
                    .features
                    .contains(crate::dialect::IndexFeatures::EXPRESSIONS)
                {
                    return Err(exc::Error::dialect("CREATE INDEX on an expression"));
                }

                out.write('(');
                other.write_sql(dialect, out)?;
                out.write(')');
            }
        }

        if column_order {
            out.write(' ');
            out.write_str(self.order.into());
        }

        Ok(())
    }
}

#[pyo3::pymethods]
impl PyIndexColumn {
    #[new]
    #[pyo3(signature = (column, *, order = "ASC"))]
    fn new(column: NameExprExtractor, order: &str) -> exc::Result<Self> {
        let order = common::Order::from_str(order)
            .map_err(|_| exc::Error::invalid("order", format!("expected 'ASC' or 'DESC', received {order}")))?;

        Ok(Self {
            column: column.0,
            order,
        })
    }

    #[getter]
    fn column<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<alias::BoundObject<'a>> {
        crate::expr::base::into_subclasses(py, self.column.clone())
    }

    #[getter]
    fn order(&self) -> &'static str {
        self.order.into()
    }

    fn __repr__(&self, py: pyo3::Python<'_>) -> exc::Result<String> {
        self.python_repr(py).map(repr::ReprFormatter::into_string)
    }
}
