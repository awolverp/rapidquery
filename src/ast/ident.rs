implement_pyclass! {
    /// An identifier
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    [skip_from_py_object] PyIdent as "Ident" {
        pub value: arcstr::ArcStr
    }
}

#[pyo3::pymethods]
impl PyIdent {
    /// Create a new identifier with the given value
    #[new]
    fn __new__(value: String) -> Self {
        Self {
            value: value.into(),
        }
    }

    fn __str__(&self) -> String {
        self.value.to_string()
    }

    fn __repr__(&self) -> String {
        format!("Ident('{}')", self.value)
    }
}

impl crate::prepare::PreparedStatement for PyIdent {
    fn prepare(
        &mut self,
        context: &mut crate::prepare::PreparedStatementContext,
    ) -> pyo3::PyResult<()> {
        match context.dialect().identifier_quote() {
            Some(crate::dialect::IdentifierQuote::SingleQuote) => {
                context.write_char('\'');
                context.write_str(&self.value);
                context.write_char('\'');
            }
            Some(crate::dialect::IdentifierQuote::DoubleQuote) => {
                context.write_char('"');
                context.write_str(&self.value);
                context.write_char('"');
            }
            Some(crate::dialect::IdentifierQuote::Backtick) => {
                context.write_char('`');
                context.write_str(&self.value);
                context.write_char('`');
            }
            Some(crate::dialect::IdentifierQuote::SquareBracket) => {
                context.write_char('[');
                context.write_str(&self.value);
                context.write_char(']');
            }
            None => {
                context.write_str(&self.value);
            }
        }

        Ok(())
    }
}
