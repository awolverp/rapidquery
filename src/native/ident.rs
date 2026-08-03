use crate::native::prepare::PrepareStatement;

/// Identifier (e.g. table name or column name).
///
/// # Note
/// We don't quote `"*"` string in preparing statement. We keep it as asterisk, which is so helpful.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ident(arcstr::ArcStr);

impl AsRef<str> for Ident {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for Ident {
    #[inline]
    fn from(value: &str) -> Self {
        Ident(value.into())
    }
}

impl From<String> for Ident {
    #[inline]
    fn from(value: String) -> Self {
        Ident(value.into())
    }
}

impl PrepareStatement for Ident {
    #[inline]
    fn prepare(
        &self,
        context: &mut crate::native::prepare::PrepareStatementContext,
    ) -> crate::native::prepare::PrepareResult<()> {
        // let Some(style) = context.dialect().identifier_quote() else {
        //     context.write_str(self.0.as_str());
        //     return Ok(());
        // };

        // let escaped = style.escape(&self.0);

        // match context.dialect().identifier_quote() {
        //     Some(IdentifierQuote::SingleQuote) => {
        //         context
        //             .write_char('\'')
        //             .write_str(&escaped)
        //             .write_char('\'');
        //     }
        //     Some(IdentifierQuote::DoubleQuote) => {
        //         context.write_char('"').write_str(&escaped).write_char('"');
        //     }
        //     Some(IdentifierQuote::Backtick) => {
        //         context.write_char('`').write_str(&escaped).write_char('`');
        //     }
        //     Some(IdentifierQuote::SquareBracket) => {
        //         context.write_char('[').write_str(&escaped).write_char(']');
        //     }
        //     None => {
        //         context.write_str(&escaped);
        //     }
        // }

        Ok(())
    }
}

/// Multipart identifier (e.g. `database.schema.table.column`)
pub struct CompoundIdent(pub Vec<Ident>);
