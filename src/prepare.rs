use std::collections::HashMap;

use crate::dialect::{DialectUnion, PlaceholderSyntax};
use crate::internal::alias;

/// Helps to create a prepared statement.
pub struct PreparedStatementContext<'c> {
    string: String,
    dialect: &'c DialectUnion,
    params: HashMap<String, alias::PyObject>,
}

impl<'c> PreparedStatementContext<'c> {
    /// Creates a new [`PreparedStatementContext`]
    #[inline]
    pub fn new(dialect: &'c DialectUnion) -> Self {
        Self {
            string: String::with_capacity(256),
            dialect,
            params: HashMap::new(),
        }
    }

    /// Returns the specified dialect
    pub fn dialect(&self) -> &DialectUnion {
        &self.dialect
    }

    /// Consumes `self` and returns SQL and parameters.
    pub fn into_parts(self) -> (String, HashMap<String, alias::PyObject>) {
        (self.string, self.params)
    }
}

impl PreparedStatementContext<'_> {
    /// Inserts a parameter into context and returns a placeholder that you can
    /// add it to SQL.
    #[inline]
    pub fn add_parameter(&mut self, param: alias::PyObject) -> String {
        let name = format!("param_{}", self.params.len());

        let placeholder = match self.dialect.placeholder_syntax() {
            PlaceholderSyntax::AtSign => String::from("@") + &name,
            PlaceholderSyntax::Colon => String::from(":") + &name,
            PlaceholderSyntax::DollarNamed => String::from("$") + &name,
        };

        self.params.insert(name, param);
        placeholder
    }

    /// Writes a character into SQL.
    #[inline]
    pub fn write_char(&mut self, c: char) {
        self.string.push(c);
    }

    /// Writes a string into SQL.
    #[inline]
    pub fn write_str(&mut self, s: &str) {
        self.string.push_str(s);
    }
}

pub trait PreparedStatement {
    fn prepare(&mut self, context: &mut PreparedStatementContext) -> pyo3::PyResult<()>;
}

pub fn prepare<'s, 'c, T: PreparedStatement>(
    stmt: &'s mut T,
    dialect: &'c DialectUnion,
) -> pyo3::PyResult<(String, HashMap<String, alias::PyObject>)> {
    let mut context = PreparedStatementContext::new(dialect);
    PreparedStatement::prepare(stmt, &mut context)?;
    Ok(context.into_parts())
}
