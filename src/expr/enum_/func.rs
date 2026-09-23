use super::ident::Ident;
use crate::sql::{PrepareResult, SqlWriter, WriteSql, write_separated};

use super::ExprEnum;

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
pub enum FunctionArguments {
    /// Used for special functions like `CURRENT_TIMESTAMP` that are invoked
    /// without parentheses.
    None,
    /// A normal function argument list, including `DISTINCT`.
    List { distinct: bool, args: Vec<ExprEnum> },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    /// The function name
    pub name: Ident,
    /// The arguments to the function, including any options specified within the
    /// delimiting parentheses.
    pub args: FunctionArguments,
    /// e.g. `x > 5` in `COUNT(x) FILTER (WHERE x > 5)`
    pub filter: Option<Box<ExprEnum>>,
    /// `IGNORE NULLS` / `RESPECT NULLS`. Inside the parens on BigQuery
    /// and DuckDB; after the call on other dialects.
    pub null_treatment: Option<NullTreatment>,
    // TODO: over, within_group
}

impl Function {
    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = format!("Function({})", self.name.python_repr());
        match &self.args {
            FunctionArguments::None => (),
            FunctionArguments::List { distinct, args } => {
                buf.push_str(".call(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        buf.push_str(", ");
                    }
                    buf.push_str(&arg.python_repr(py)?);
                }

                if *distinct {
                    if !args.is_empty() {
                        buf.push_str(", ");
                    }
                    buf.push_str("distinct=True");
                }
                buf.push(')');
            }
        }
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
        // TODO:
        // - `DuplicateTreatment::All` is not supported by SQLite/BigQuery, we should skip it
        // - BigQuery supports `WHERE condition` instead of `FILTER (WHERE condition)`

        self.name.write_sql(dialect, out)?;
        let nt_inside = matches!(
            dialect.0.null_treatment_syntax,
            crate::dialect::inner::NullTreatmentSyntax::InsideParens
        );

        match &self.args {
            FunctionArguments::None => {
                // No parentheses to put the clause in; always postfix.
                if let Some(nt) = self.null_treatment {
                    out.push_char(' ');
                    out.push_str(nt.as_sql());
                }
            }
            FunctionArguments::List { distinct, args } => {
                out.push_char('(');

                if *distinct && !args.is_empty() {
                    out.push_str("DISTINCT ");
                }

                write_separated(dialect, out, args, ", ")?;
                if nt_inside && let Some(nt) = self.null_treatment {
                    if !args.is_empty() {
                        out.push_char(' ');
                    }
                    out.push_str(nt.as_sql());
                }

                out.push_char(')');
                if !nt_inside && let Some(nt) = self.null_treatment {
                    out.push_char(' ');
                    out.push_str(nt.as_sql());
                }
            }
        }
        if let Some(filter) = &self.filter {
            out.push_str(" FILTER (WHERE ");
            filter.write_sql(dialect, out)?;
            out.push_char(')');
        }
        Ok(())
    }
}
