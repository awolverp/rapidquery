use super::ident::Ident;
use crate::sql::{PrepareResult, SqlWriter, WriteSql, write_separated};

use super::ExprEnum;

/// How duplicate values are treated inside function argument lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DuplicateTreatment {
    /// Consider only unique values.
    Distinct,
    /// Retain all duplicate values (the default).
    All,
}

impl DuplicateTreatment {
    pub fn as_sql(self) -> &'static str {
        match self {
            Self::Distinct => "DISTINCT",
            Self::All => "ALL",
        }
    }
}

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
    /// A normal function argument list, including `DISTINCT` / `ALL`.
    List {
        duplicate_treatment: Option<DuplicateTreatment>,
        args: Vec<ExprEnum>,
    },
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
            FunctionArguments::List {
                duplicate_treatment,
                args,
            } => {
                buf.push_str(".call(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        buf.push_str(", ");
                    }
                    buf.push_str(&arg.python_repr(py)?);
                }
                if let Some(dt) = duplicate_treatment {
                    if !args.is_empty() {
                        buf.push_str(", ");
                    }
                    match dt {
                        DuplicateTreatment::Distinct => buf.push_str("distinct=True"),
                        DuplicateTreatment::All => buf.push_str("distinct=False"),
                    }
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
        dialect: &crate::dialect::SqlDialect,
        out: &mut W,
    ) -> PrepareResult<()> {
        self.name.write_sql(dialect, out)?;
        let nt_inside = dialect.null_treatment_inside_parens();
        match &self.args {
            FunctionArguments::None => {
                // No parentheses to put the clause in; always postfix.
                if let Some(nt) = self.null_treatment {
                    out.push_char(' ');
                    out.push_str(nt.as_sql());
                }
            }
            FunctionArguments::List {
                duplicate_treatment,
                args,
            } => {
                out.push_char('(');

                if let Some(dt) = duplicate_treatment {
                    if !args.is_empty() {
                        out.push_str(dt.as_sql());
                        out.push_char(' ');
                    }
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
