//! SQL identifiers.
//!
//! An [`Ident`] is a dotted name (`schema.table.column`) made of
//! [`IdentPart`]s. Each part is one of:
//!
//! * [`IdentPart::Quote`] — wrapped in the dialect's identifier delimiters
//!   (and escaped). This is the default for a Python `str`.
//! * [`IdentPart::Unquote`] — emitted as a raw identifier body. **Not** a
//!   raw-SQL hole: the body must be a simple ASCII name
//!   (`[A-Za-z_][A-Za-z0-9_]*`). `COUNT(*)` and similar fragments belong on
//!   a later expression type.
//! * [`IdentPart::Asterisk`] — the SQL wildcard `*`. Must be the last part
//!   (`t.*`, never `*.id`).
//!
//! # Python vs Rust
//!
//! The Python constructor takes one argument per part (`Ident("schema",
//! "table")`). A Python `str` is always [`IdentPart::Quote`], including
//! `"*"` (a column named star). The wildcard is the `Asterisk` object.
//!
//! [`Ident::from_str`] is a **Rust** dotted-name parser: `"t.*"` ends in
//! [`IdentPart::Asterisk`], and a `*` that is not last is demoted to
//! [`IdentPart::Quote`]. Python does not call this parser.

use smallvec::SmallVec;

use crate::native::{
    dialect::SqlDialect,
    prepare_statement::{PrepareResult, PrepareStatement, prepare_separated},
};

/// One segment of a dotted identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IdentPart {
    /// Identifier that should be quoted and escaped for the dialect.
    Quote(arcstr::ArcStr),
    /// Identifier emitted without delimiters. Must be a simple ASCII name.
    Unquote(arcstr::ArcStr),
    /// SQL wildcard `*`.
    Asterisk,
}

impl IdentPart {
    /// Quoted part whose body is `s`.
    ///
    /// `"*"` is a quoted identifier named star, not [`IdentPart::Asterisk`].
    pub fn quoted(s: impl AsRef<str>) -> Result<Self, ParseIdentError> {
        let s = s.as_ref();
        if s.is_empty() {
            return Err(ParseIdentError::EmptyChunk);
        }
        Ok(Self::Quote(s.into()))
    }

    /// Unquoted part whose body is `s`.
    ///
    /// Rejects anything that is not `[A-Za-z_][A-Za-z0-9_]*`.
    pub fn unquoted(s: impl AsRef<str>) -> Result<Self, ParseIdentError> {
        let s = s.as_ref();
        if s.is_empty() {
            return Err(ParseIdentError::EmptyChunk);
        }
        if !is_safe_unquoted(s) {
            return Err(ParseIdentError::InvalidUnquoted);
        }
        Ok(Self::Unquote(s.into()))
    }
}

impl<S: AsRef<str>> From<S> for IdentPart {
    /// Parser helper used by [`Ident::from_str`].
    ///
    /// A literal `"*"` becomes [`IdentPart::Asterisk`]. Everything else is
    /// [`IdentPart::Quote`] with no emptiness check — [`Ident::from_str`]
    /// rejects empty chunks first. Python `str` extract does **not** use
    /// this impl (it always goes through [`IdentPart::quoted`]).
    fn from(value: S) -> Self {
        let s = value.as_ref();
        if s == "*" {
            IdentPart::Asterisk
        } else {
            IdentPart::Quote(s.into())
        }
    }
}

impl PrepareStatement for IdentPart {
    #[inline]
    fn prepare(
        &self,
        dialect: &SqlDialect,
        context: &mut crate::native::prepare_statement::PrepareContext<'_>,
    ) -> PrepareResult<()> {
        match self {
            Self::Quote(name) => {
                dialect.quote_identifier(context, name.as_str())?;
            }
            Self::Unquote(name) => {
                context.push_str(name.as_str());
            }
            Self::Asterisk => {
                context.push_char('*');
            }
        }
        Ok(())
    }
}

/// Dotted identifier (`schema.table.column`).
///
/// Never empty. At most [`Ident::MAX_PARTS`] parts. An [`IdentPart::Asterisk`]
/// may appear only as the last part.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ident(SmallVec<[IdentPart; 4]>);

/// Why building an [`Ident`] failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseIdentError {
    /// No parts (empty string, empty constructor).
    Empty,
    /// A quoted or unquoted part was the empty string (`"a..b"`, `""` as a part).
    EmptyChunk,
    /// More than [`Ident::MAX_PARTS`] parts.
    TooManyParts,
    /// `*` appeared before the last part (`*.id`).
    AsteriskNotLast,
    /// Unquoted body was not a simple ASCII identifier.
    InvalidUnquoted,
}

impl std::fmt::Display for ParseIdentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Empty => "identifier is empty",
            Self::EmptyChunk => "identifier has an empty part",
            Self::TooManyParts => "identifier has too many parts",
            Self::AsteriskNotLast => "asterisk must be the last identifier part",
            Self::InvalidUnquoted => "unquoted identifier is not a simple ASCII name",
        })
    }
}

impl std::error::Error for ParseIdentError {}

/// `[A-Za-z_][A-Za-z0-9_]*`. Empty string is not safe.
fn is_safe_unquoted(s: &str) -> bool {
    let mut bytes = s.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() && first != b'_' {
        return false;
    }
    bytes.all(|c| c.is_ascii_alphanumeric() || c == b'_')
}

impl Ident {
    /// Maximum number of dotted parts.
    pub const MAX_PARTS: usize = 10;

    /// Borrowed parts in order.
    #[inline]
    pub fn parts(&self) -> &[IdentPart] {
        &self.0
    }

    /// Number of parts. Always ≥ 1 for a successfully built `Ident`.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// `true` when there are no parts. Unreachable for a validated `Ident`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Consume and yield the parts.
    #[inline]
    pub fn into_parts(self) -> SmallVec<[IdentPart; 4]> {
        self.0
    }

    /// Build from parts, validating emptiness, length, unquoted charset, and
    /// asterisk position.
    pub fn from_parts(iter: impl IntoIterator<Item = IdentPart>) -> Result<Self, ParseIdentError> {
        let mut parts = SmallVec::<[IdentPart; 4]>::new();

        for part in iter {
            if let Some(IdentPart::Asterisk) = parts.last() {
                return Err(ParseIdentError::AsteriskNotLast);
            }
            if parts.len() >= Self::MAX_PARTS {
                return Err(ParseIdentError::TooManyParts);
            }
            match &part {
                IdentPart::Quote(name) if name.is_empty() => {
                    return Err(ParseIdentError::EmptyChunk);
                }
                IdentPart::Unquote(name) => {
                    if name.is_empty() {
                        return Err(ParseIdentError::EmptyChunk);
                    }
                    if !is_safe_unquoted(name) {
                        return Err(ParseIdentError::InvalidUnquoted);
                    }
                }
                IdentPart::Quote(_) | IdentPart::Asterisk => {}
            }
            parts.push(part);
        }

        if parts.is_empty() {
            return Err(ParseIdentError::Empty);
        }

        Ok(Self(parts))
    }
}

impl std::str::FromStr for Ident {
    type Err = ParseIdentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseIdentError::Empty);
        }

        let mut parts = SmallVec::<[IdentPart; 4]>::new();

        for chunk in s.splitn(Self::MAX_PARTS + 1, '.') {
            if chunk.is_empty() {
                return Err(ParseIdentError::EmptyChunk);
            }
            if parts.len() >= Self::MAX_PARTS {
                return Err(ParseIdentError::TooManyParts);
            }
            // A bare `*` only makes sense as the final part (e.g. `table.*`).
            // If another part follows, demote the previous `*` to a quoted star.
            if let Some(last) = parts.last_mut()
                && matches!(last, IdentPart::Asterisk)
            {
                *last = IdentPart::Quote(arcstr::literal!("*"));
            }
            parts.push(chunk.into());
        }

        Self::from_parts(parts)
    }
}

impl PrepareStatement for Ident {
    #[inline]
    fn prepare(
        &self,
        dialect: &SqlDialect,
        context: &mut crate::native::prepare_statement::PrepareContext<'_>,
    ) -> PrepareResult<()> {
        prepare_separated(dialect, context, &self.0, ".")
    }
}
