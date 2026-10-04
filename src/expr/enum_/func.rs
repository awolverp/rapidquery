use super::ExprInner;
use super::ident::Ident;

use crate::dialect::flags;
use crate::internal::repr::ReprFormatter;
use crate::sql::PrepareResult;
use crate::sql::SqlWriter;
use crate::sql::WriteSql;
use crate::sql::write_separated;

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
    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<ReprFormatter> {
        let mut fmt = ReprFormatter::new_call("Function");
        fmt.positional_formatter(self.name.python_repr());
        for arg in &self.args {
            fmt.positional_formatter(arg.python_repr(py)?);
        }
        fmt.keyword_if_true("distinct", self.distinct);
        fmt.close();

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
            let argument = filter.python_repr(py)?;
            fmt.append_call("filter", &argument);
        }
        Ok(fmt)
    }
}

impl WriteSql for Function {
    fn write_sql<W: SqlWriter>(&self, dialect: &crate::dialect::PyDialect, out: &mut W) -> PrepareResult<()> {
        // "name"([DISTINCT] args [IGNORE|RESPECT NULLS] [WHERE expr])
        // [FILTER ([WHERE] expr)] [IGNORE|RESPECT NULLS]
        self.name.write_sql(dialect, out)?;

        let args_available = !self.args.is_empty();

        out.push_char('(');
        out.push_str_if(self.distinct && args_available, "DISTINCT ");

        write_separated(dialect, out, &self.args, ", ")?;

        let null_treatment_syntax = dialect.0.null_treatment();
        let function_filter_syntax = dialect.0.function_filter();

        if self.null_treatment.is_some() && matches!(null_treatment_syntax, flags::NullTreatment::Unsupported) {
            return Err(
                rapidquery_macros::pyerr!(PyRuntimeError, "null_treatment is not supported by this dialect",).into(),
            );
        }
        if self.filter.is_some() && matches!(function_filter_syntax, flags::FunctionFilter::Unsupported) {
            return Err(
                rapidquery_macros::pyerr!(PyRuntimeError, "function filter is not supported by this dialect",).into(),
            );
        }

        // Inside parens IGNORE/RESPECT NULLS
        if let Some(x) = self.null_treatment
            && matches!(null_treatment_syntax, flags::NullTreatment::InsideParens)
        {
            out.push_char_if(args_available, ' ');
            out.push_str(x.as_sql());
        }

        // Inside parens WHERE filter
        if let Some(x) = &self.filter
            && matches!(function_filter_syntax, flags::FunctionFilter::InsideWhere)
        {
            out.push_char_if(args_available, ' ');
            out.push_str("WHERE ");
            x.write_sql(dialect, out)?;
        }

        out.push_char(')');

        // FILTER, then postfix IGNORE/RESPECT NULLS (immediately before OVER).
        if let Some(x) = &self.filter {
            match function_filter_syntax {
                flags::FunctionFilter::FilterWhere => {
                    out.push_char(' ');
                    out.push_str("FILTER (WHERE ");
                    x.write_sql(dialect, out)?;
                    out.push_char(')');
                }
                flags::FunctionFilter::Filter => {
                    out.push_char(' ');
                    out.push_str("FILTER (");
                    x.write_sql(dialect, out)?;
                    out.push_char(')');
                }
                _ => {}
            }
        }

        if let Some(x) = self.null_treatment
            && matches!(null_treatment_syntax, flags::NullTreatment::Postfix)
        {
            out.push_char(' ');
            out.push_str(x.as_sql());
        }

        Ok(())
    }
}
