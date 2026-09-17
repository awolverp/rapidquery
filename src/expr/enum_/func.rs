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
    // TODO: null_treatment, over, within_group
}

impl Function {
    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = format!("Function({}", self.name.python_repr());
        match &self.args {
            FunctionArguments::None => (),
            FunctionArguments::List {
                duplicate_treatment,
                args,
            } => {
                for arg in args {
                    buf.push_str(", ");
                    buf.push_str(&arg.python_repr(py)?);
                }
                if let Some(dt) = duplicate_treatment {
                    match dt {
                        DuplicateTreatment::Distinct => buf.push_str(", distinct=True"),
                        DuplicateTreatment::All => buf.push_str(", distinct=False"),
                    }
                }
            }
        }
        if let Some(filter) = &self.filter {
            buf.push_str(", filter=");
            buf.push_str(&filter.python_repr(py)?);
        }
        buf.push(')');
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
        match &self.args {
            FunctionArguments::None => {}
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
                out.push_char(')');
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
