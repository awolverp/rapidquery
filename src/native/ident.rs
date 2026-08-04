use crate::native::{
    dialect::SqlDialect,
    prepare::{PrepareStatement, prepare_separated},
};

/// Identifier (e.g. table name or column name).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Ident {
    Content(arcstr::ArcStr),
    Asterisk,
}

impl<S: AsRef<str>> From<S> for Ident {
    /// Always treats a literal `"*"` as `Ident::Asterisk`.
    /// Use `Ident::Content(s.into())` directly if you don't want that.
    fn from(value: S) -> Self {
        let s = value.as_ref();
        if s == "*" {
            Ident::Asterisk
        } else {
            Ident::Content(s.into())
        }
    }
}

impl PrepareStatement for Ident {
    #[inline]
    fn prepare(
        &self,
        dialect: &SqlDialect,
        context: &mut crate::native::prepare::PrepareStatementContext,
    ) -> crate::native::prepare::PrepareResult<()> {
        match self {
            Self::Asterisk => {
                context.write_char('*');
            }
            Self::Content(c) => {
                dialect.escape_identifier(context, c.as_str(), true)?;
            }
        }
        Ok(())
    }
}

/// Multipart identifier (e.g. `database.schema.table.column`)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdentList(pub Vec<Ident>);

#[derive(Debug, Clone, Copy, ::derive_more::Display, ::derive_more::Error)]
pub enum ParseIdentError {
    #[display("identifier string is empty")]
    Empty,
    #[display("identifier has an empty chunk")]
    EmptyChunk,
    #[display("identifier has too many parts")]
    TooManyParts,
}

impl std::str::FromStr for IdentList {
    type Err = ParseIdentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // The considered maximum splits is `10`
        const MAX_SPLITS: usize = 10;

        if s.is_empty() {
            return Err(ParseIdentError::Empty);
        }

        let iter = s.splitn(MAX_SPLITS + 1, '.');
        let mut parts = Vec::<Ident>::with_capacity(MAX_SPLITS);

        for chunk in iter {
            if chunk.is_empty() {
                return Err(ParseIdentError::EmptyChunk);
            }
            if parts.len() >= MAX_SPLITS {
                return Err(ParseIdentError::TooManyParts);
            }
            parts.push(chunk.into());
        }

        if let Some(last) = parts.last_mut()
            && matches!(last, Ident::Content(c) if c.as_str() == "*")
        {
            *last = Ident::Asterisk;
        }

        Ok(Self(parts))
    }
}

impl From<Ident> for IdentList {
    fn from(value: Ident) -> Self {
        Self(vec![value])
    }
}

impl From<Vec<Ident>> for IdentList {
    fn from(value: Vec<Ident>) -> Self {
        Self(value)
    }
}

impl PrepareStatement for IdentList {
    #[inline]
    fn prepare(
        &self,
        dialect: &SqlDialect,
        context: &mut crate::native::prepare::PrepareStatementContext,
    ) -> crate::native::prepare::PrepareResult<()> {
        prepare_separated(dialect, context, &self.0, '.')
    }
}
