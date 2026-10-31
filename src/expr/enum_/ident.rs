//! SQL identifiers.

use crate::dialect::PyDialect;
use crate::error;
use crate::internal::repr;
use crate::sql::SqlWriter;
use crate::sql::WriteSql;
use crate::sql::write_separated;

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
    pub fn quoted(s: impl AsRef<str>) -> Result<Self, IdentParseError> {
        let s = s.as_ref();
        if s.is_empty() {
            return Err(IdentParseError::EmptyChunk);
        }
        Ok(Self::Quote(arcstr::ArcStr::from(s)))
    }

    /// Unquoted part whose body is `s`.
    ///
    /// Rejects anything that is not `[A-Za-z_][A-Za-z0-9_]*`.
    pub fn unquoted(s: impl AsRef<str>) -> Result<Self, IdentParseError> {
        let s = s.as_ref();
        if s.is_empty() {
            return Err(IdentParseError::EmptyChunk);
        }
        if !is_safe_unquoted(s) {
            return Err(IdentParseError::InvalidUnquoted(arcstr::ArcStr::from(s)));
        }
        Ok(Self::Unquote(arcstr::ArcStr::from(s)))
    }
}

impl<S: AsRef<str>> From<S> for IdentPart {
    /// `"*"` becomes [`IdentPart::Asterisk`]. Any other string is quoted.
    ///
    /// Empty chunks are not rejected here. [`Ident::from_str`] checks those first.
    fn from(value: S) -> Self {
        let s = value.as_ref();
        if s == "*" {
            IdentPart::Asterisk
        } else {
            IdentPart::Quote(arcstr::ArcStr::from(s))
        }
    }
}

impl PartialEq<&str> for IdentPart {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Self::Quote(x) => x.as_str() == *other,
            Self::Unquote(x) => x.as_str() == *other,
            Self::Asterisk => "*" == *other,
        }
    }
}

impl WriteSql for IdentPart {
    #[inline]
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> error::Result<()> {
        match self {
            Self::Quote(name) => {
                dialect.write_identifier(out, name.as_str());
            }
            Self::Unquote(name) => {
                out.write_str(name.as_str());
            }
            Self::Asterisk => {
                out.write('*');
            }
        }
        Ok(())
    }
}

/// One or more identifier parts, joined with `.` when rendered.
///
/// A validated ident is never empty and holds at most [`Ident::MAX_PARTS`] parts.
/// [`IdentPart::Asterisk`] is legal only as the last part.
///
/// ```
/// use std::str::FromStr;
///
/// use rapidquery::expr::enum_::Ident;
///
/// let ident = Ident::from_str("users.id").unwrap();
/// assert_eq!(ident.len(), 2);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ident(smallvec::SmallVec<[IdentPart; 3]>);

/// Why building an [`Ident`] failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentParseError {
    /// No parts (empty string, empty constructor).
    Empty,
    /// A quoted or unquoted part was the empty string (`"a..b"`, `""` as a part).
    EmptyChunk,
    /// More than [`Ident::MAX_PARTS`] parts.
    TooManyParts,
    /// `*` appeared before the last part (`*.id`).
    AsteriskNotLast,
    /// Unquoted body was not a simple ASCII identifier. Holds the rejected text.
    InvalidUnquoted(arcstr::ArcStr),
}

impl std::fmt::Display for IdentParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.write_str("identifier is empty"),
            Self::EmptyChunk => f.write_str("identifier has an empty part"),
            Self::TooManyParts => write!(f, "identifier has too many parts; the maximum is {}", Ident::MAX_PARTS),
            Self::AsteriskNotLast => f.write_str("asterisk must be the last identifier part"),
            Self::InvalidUnquoted(name) => write!(
                f,
                "unquoted identifier {name:?} is not a simple ASCII name; expected [A-Za-z_][A-Za-z0-9_]*"
            ),
        }
    }
}

impl std::error::Error for IdentParseError {}

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
    pub fn into_parts(self) -> smallvec::SmallVec<[IdentPart; 3]> {
        self.0
    }

    pub fn python_repr(&self) -> repr::ReprFormatter {
        let mut fmt = repr::ReprFormatter::new_call("Ident");

        for item in self.parts() {
            match item {
                IdentPart::Quote(name) => {
                    fmt.positional(name.as_str());
                }
                IdentPart::Unquote(name) => {
                    let mut unquote = repr::ReprFormatter::new_call("Unquote");
                    unquote.positional(name.as_str());
                    fmt.positional(unquote);
                }
                IdentPart::Asterisk => {
                    fmt.positional_raw("Asterisk");
                }
            }
        }

        fmt.close();
        fmt
    }

    /// One already-validated part. Prefer this over [`from_parts`] for a
    /// single `str` / `Unquote` / `Asterisk`.
    #[inline]
    pub fn from_one(part: IdentPart) -> Self {
        Self(smallvec::smallvec![part])
    }

    /// Build from parts. Composition only: empty ident, empty chunk,
    /// [`MAX_PARTS`](Self::MAX_PARTS), asterisk-not-last.
    ///
    /// Unquoted charset is enforced by [`IdentPart::unquoted`], not here.
    #[cfg_attr(feature = "nightly", optimize(speed))]
    pub fn from_parts(iter: impl IntoIterator<Item = IdentPart>) -> Result<Self, IdentParseError> {
        let mut parts = smallvec::SmallVec::<[IdentPart; 3]>::new();

        for part in iter {
            if let Some(IdentPart::Asterisk) = parts.last() {
                return Err(IdentParseError::AsteriskNotLast);
            }
            if parts.len() >= Self::MAX_PARTS {
                return Err(IdentParseError::TooManyParts);
            }
            match &part {
                IdentPart::Quote(name) | IdentPart::Unquote(name) if name.is_empty() => {
                    return Err(IdentParseError::EmptyChunk);
                }
                IdentPart::Quote(_) | IdentPart::Unquote(_) | IdentPart::Asterisk => {}
            }
            parts.push(part);
        }

        if parts.is_empty() {
            return Err(IdentParseError::Empty);
        }

        Ok(Self(parts))
    }
}

impl std::str::FromStr for Ident {
    type Err = IdentParseError;

    #[cfg_attr(feature = "nightly", optimize(speed))]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(IdentParseError::Empty);
        }

        let mut parts = smallvec::SmallVec::<[IdentPart; 3]>::new();

        for chunk in s.splitn(Self::MAX_PARTS + 1, '.') {
            if chunk.is_empty() {
                return Err(IdentParseError::EmptyChunk);
            }
            if parts.len() >= Self::MAX_PARTS {
                return Err(IdentParseError::TooManyParts);
            }
            // A bare `*` only makes sense as the final part (e.g. `table.*`).
            // If another part follows, demote the previous `*` to a quoted star.
            if let Some(last) = parts.last_mut()
                && matches!(last, IdentPart::Asterisk)
            {
                *last = IdentPart::Quote(arcstr::ArcStr::from("*"));
            }
            parts.push(chunk.into());
        }

        Ok(Self(parts))
    }
}

impl WriteSql for Ident {
    #[inline]
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> error::Result<()> {
        write_separated(dialect, out, &self.0, ".")
    }
}
