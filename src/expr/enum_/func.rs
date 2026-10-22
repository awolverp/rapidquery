use super::ExprInner;
use super::ident::Ident;

use crate::dialect::config;
use crate::internal::repr;
use crate::internal::sql::PrepareResult;
use crate::internal::sql::SqlWriter;
use crate::internal::sql::WriteSql;
use crate::internal::sql::write_separated;

/// How `NULL`s are treated in some window/aggregate functions.
///
/// Placement is dialect-specific: postfix `LAG(x) IGNORE NULLS`, or
/// inside the argument list on BigQuery/DuckDB (`LAG(x IGNORE NULLS)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NullTreatment {
    /// `IGNORE NULLS`
    IgnoreNulls,
    /// `RESPECT NULLS`
    RespectNulls,
}

impl NullTreatment {
    pub fn as_sql(self) -> &'static str {
        match self {
            Self::IgnoreNulls => "IGNORE NULLS",
            Self::RespectNulls => "RESPECT NULLS",
        }
    }
}

/// Function call: name, arguments, `DISTINCT`, filter, and null treatment.
///
/// The name is an [`Ident`]. Arguments are expression nodes. `filter` and
/// `null_treatment` are omitted from the SQL when they are `None`.
///
/// ```
/// use std::str::FromStr;
///
/// use rapidquery::expr::enum_::Function;
/// use rapidquery::expr::enum_::Ident;
///
/// let func = Function {
///     name: Ident::from_str("count").unwrap(),
///     distinct: false,
///     args: Vec::new(),
///     filter: None,
///     null_treatment: None,
/// };
/// assert_eq!(func.name.len(), 1);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    /// The function name
    pub name: Ident,
    /// `DISTINCT` keyword
    pub distinct: bool,
    /// The arguments to the function, including any options specified within the
    /// delimiting parentheses.
    pub args: Vec<ExprInner>,
    /// e.g. `x > 5` in `COUNT(x) FILTER (WHERE x > 5)`
    pub filter: Option<Box<ExprInner>>,
    /// `IGNORE NULLS` / `RESPECT NULLS`. Inside the parens on BigQuery
    /// and DuckDB; after the call on other dialects.
    pub null_treatment: Option<NullTreatment>,
    // TODO: over, within_group
}

impl Function {
    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<repr::ReprFormatter> {
        let mut fmt = repr::ReprFormatter::new_call("Function");

        fmt.positional(self.name.python_repr());

        for arg in &self.args {
            fmt.positional(arg.python_repr(py)?);
        }

        fmt.keyword_true("distinct", self.distinct).close();

        match self.null_treatment {
            Some(NullTreatment::IgnoreNulls) => {
                fmt.append_suffix(".ignore_nulls()");
            }
            Some(NullTreatment::RespectNulls) => {
                fmt.append_suffix(".respect_nulls()");
            }
            None => {}
        }

        if let Some(filter) = &self.filter {
            fmt.append_call("filter").positional(filter.python_repr(py)?);
        }

        Ok(fmt)
    }
}

impl WriteSql for Function {
    fn write_sql<W: SqlWriter>(&self, dialect: &crate::dialect::PyDialect, out: &mut W) -> PrepareResult<()> {
        // "name"([DISTINCT] args [{IGNORE|RESPECT} NULLS] [WHERE expr])
        // [FILTER ([WHERE] expr)] [{IGNORE|RESPECT} NULLS]
        self.name.write_sql(dialect, out)?;

        let args_available = !self.args.is_empty();

        out.push_char('(');
        out.push_str_if(self.distinct && args_available, "DISTINCT ");

        write_separated(dialect, out, &self.args, ", ")?;

        if self.null_treatment.is_some() && matches!(dialect.0.null_treatment, config::NullTreatment::Unsupported) {
            return Err(
                rapidquery_macros::pyerr!(PyRuntimeError, "null_treatment is not supported by this dialect",).into(),
            );
        }
        if self.filter.is_some() && matches!(dialect.0.function_filter, config::FunctionFilter::Unsupported) {
            return Err(
                rapidquery_macros::pyerr!(PyRuntimeError, "function filter is not supported by this dialect",).into(),
            );
        }

        // Inside parens IGNORE/RESPECT NULLS
        if let Some(x) = self.null_treatment
            && matches!(dialect.0.null_treatment, config::NullTreatment::InsideParens)
        {
            out.push_char_if(args_available, ' ');
            out.push_str(x.as_sql());
        }

        // Inside parens WHERE filter
        if let Some(x) = &self.filter
            && matches!(dialect.0.function_filter, config::FunctionFilter::InsideWhere)
        {
            out.push_char_if(args_available, ' ');
            out.push_str("WHERE ");
            x.write_sql(dialect, out)?;
        }

        out.push_char(')');

        // FILTER, then postfix IGNORE/RESPECT NULLS (immediately before OVER).
        if let Some(x) = &self.filter {
            match dialect.0.function_filter {
                config::FunctionFilter::FilterWhere => {
                    out.push_char(' ');
                    out.push_str("FILTER (WHERE ");
                    x.write_sql(dialect, out)?;
                    out.push_char(')');
                }
                config::FunctionFilter::Filter => {
                    out.push_char(' ');
                    out.push_str("FILTER (");
                    x.write_sql(dialect, out)?;
                    out.push_char(')');
                }
                _ => {}
            }
        }

        if let Some(x) = self.null_treatment
            && matches!(dialect.0.null_treatment, config::NullTreatment::Postfix)
        {
            out.push_char(' ');
            out.push_str(x.as_sql());
        }

        Ok(())
    }
}
