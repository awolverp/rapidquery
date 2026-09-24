use super::ExprEnum;
use super::ident::Ident;

use crate::dialect::inner::{FunctionFilterSyntax, NullTreatmentSyntax};
use crate::sql::{PrepareResult, SqlWriter, WriteSql, write_separated};

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
    pub args: Vec<ExprEnum>,
    /// e.g. `x > 5` in `COUNT(x) FILTER (WHERE x > 5)`
    pub filter: Option<Box<ExprEnum>>,
    /// `IGNORE NULLS` / `RESPECT NULLS`. Inside the parens on BigQuery
    /// and DuckDB; after the call on other dialects.
    pub null_treatment: Option<NullTreatment>,
    // TODO: over, within_group
}

impl Function {
    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = format!("Function({}", self.name.python_repr());

        for arg in self.args.iter() {
            buf.push_str(", ");
            buf.push_str(&arg.python_repr(py)?);
        }

        if self.distinct {
            if !self.args.is_empty() {
                buf.push_str(", ");
            }
            buf.push_str("distinct=True");
        }
        buf.push(')');

        match self.null_treatment {
            Some(NullTreatment::IgnoreNulls) => buf.push_str(".ignore_nulls()"),
            Some(NullTreatment::RespectNulls) => buf.push_str(".respect_nulls()"),
            None => {}
        }
        if let Some(filter) = &self.filter {
            buf.push_str(".filter(");
            buf.push_str(&filter.python_repr(py)?);
            buf.push(')');
        }
        Ok(buf)
    }
}

impl WriteSql for Function {
    fn write_sql<W: SqlWriter>(
        &self,
        dialect: &crate::dialect::PySqlDialect,
        out: &mut W,
    ) -> PrepareResult<()> {
        // "name"([DISTINCT] args [IGNORE|RESPECT NULLS] [WHERE expr] ) [IGNORE|RESPECT NULLS] [FILTER ([WHERE] expr)]
        self.name.write_sql(dialect, out)?;

        let args_available = !self.args.is_empty();

        out.push_char('(');
        out.push_str_if(self.distinct && args_available, "DISTINCT ");

        write_separated(dialect, out, &self.args, ", ")?;

        if self.null_treatment.is_some() && dialect.0.null_treatment_syntax.is_none() {
            return Err(rapidquery_macros::pyerr!(
                PyRuntimeError,
                "null_treatment is not supported by this dialect",
            )
            .into());
        }
        if self.filter.is_some() && dialect.0.function_filter_syntax.is_none() {
            return Err(rapidquery_macros::pyerr!(
                PyRuntimeError,
                "function filter is not supported by this dialect",
            )
            .into());
        }

        // Inside parens IGNORE/RESPECT NULLS
        if let Some(x) = self.null_treatment
            && matches!(
                dialect.0.null_treatment_syntax,
                Some(NullTreatmentSyntax::InsideParens)
            )
        {
            out.push_char_if(args_available, ' ');
            out.push_str(x.as_sql());
        }

        // Inside parens WHERE filter
        if let Some(x) = &self.filter
            && matches!(
                dialect.0.function_filter_syntax,
                Some(FunctionFilterSyntax::InsideWhere)
            )
        {
            out.push_char_if(args_available, ' ');
            out.push_str("WHERE ");
            x.write_sql(dialect, out)?;
        }

        out.push_char(')');

        // Postfix IGNORE/RESPECT NULLS
        if let Some(x) = self.null_treatment
            && matches!(
                dialect.0.null_treatment_syntax,
                Some(NullTreatmentSyntax::Postfix)
            )
        {
            out.push_char(' ');
            out.push_str(x.as_sql());
        }

        // Postfix WHERE filter
        if let Some(x) = &self.filter {
            match dialect.0.function_filter_syntax {
                Some(FunctionFilterSyntax::FilterWhere) => {
                    out.push_char(' ');
                    out.push_str("FILTER (WHERE ");
                    x.write_sql(dialect, out)?;
                    out.push_char(')');
                }
                Some(FunctionFilterSyntax::Filter) => {
                    out.push_char(' ');
                    out.push_str("FILTER (");
                    x.write_sql(dialect, out)?;
                    out.push_char(')');
                }
                _ => {}
            }
        }

        Ok(())
    }
}
