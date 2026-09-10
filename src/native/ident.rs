//! SQL identifiers.

use std::fmt;

use arcstr::ArcStr;
use smallvec::{SmallVec, smallvec};

use crate::native::{
    dialect::SqlDialect,
    sql::{PrepareResult, SqlWriter, WriteSql, write_separated},
};

/// One segment of a dotted identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IdentPart {
    /// Identifier that should be quoted and escaped for the dialect.
    Quote(ArcStr),
    /// Identifier emitted without delimiters. Must be a simple ASCII name.
    Unquote(ArcStr),
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
        Ok(Self::Quote(ArcStr::from(s)))
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
        Ok(Self::Unquote(ArcStr::from(s)))
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
            IdentPart::Quote(ArcStr::from(s))
        }
    }
}

impl WriteSql for IdentPart {
    #[inline]
    fn write_sql<W: SqlWriter>(&self, dialect: &SqlDialect, out: &mut W) -> PrepareResult<()> {
        match self {
            Self::Quote(name) => {
                dialect.quote_identifier(out, name.as_str())?;
            }
            Self::Unquote(name) => {
                out.push_str(name.as_str());
            }
            Self::Asterisk => {
                out.push_char('*');
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

impl fmt::Display for ParseIdentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

    /// One already-validated part. Prefer this over [`from_parts`] for a
    /// single `str` / `Unquote` / `Asterisk`.
    #[inline]
    pub fn from_one(part: IdentPart) -> Self {
        Self(smallvec![part])
    }

    /// Build from parts. Composition only: empty ident, empty chunk,
    /// [`MAX_PARTS`](Self::MAX_PARTS), asterisk-not-last.
    ///
    /// Unquoted charset is enforced by [`IdentPart::unquoted`], not here.
    #[cfg_attr(feature = "nightly", optimize(speed))]
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
                IdentPart::Quote(name) | IdentPart::Unquote(name) if name.is_empty() => {
                    return Err(ParseIdentError::EmptyChunk);
                }
                IdentPart::Quote(_) | IdentPart::Unquote(_) | IdentPart::Asterisk => {}
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

    #[cfg_attr(feature = "nightly", optimize(speed))]
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
                *last = IdentPart::Quote(ArcStr::from("*"));
            }
            parts.push(chunk.into());
        }

        Ok(Self(parts))
    }
}

impl WriteSql for Ident {
    #[inline]
    fn write_sql<W: SqlWriter>(&self, dialect: &SqlDialect, out: &mut W) -> PrepareResult<()> {
        write_separated(dialect, out, &self.0, ".")
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    fn ident_from(parts: impl IntoIterator<Item = IdentPart>) -> Ident {
        Ident::from_parts(parts).unwrap()
    }

    fn quote(s: &str) -> IdentPart {
        IdentPart::quoted(s).unwrap()
    }

    #[test]
    fn from_str_splits_on_dots() {
        let ident = Ident::from_str("public.users.id").unwrap();
        assert_eq!(
            ident.parts(),
            &[quote("public"), quote("users"), quote("id"),]
        );
    }

    #[test]
    fn from_str_bare_star_is_asterisk() {
        let ident = Ident::from_str("*").unwrap();
        assert_eq!(ident.parts(), &[IdentPart::Asterisk]);
    }

    #[test]
    fn from_str_table_star_keeps_asterisk_last() {
        let ident = Ident::from_str("t.*").unwrap();
        assert_eq!(ident.parts(), &[quote("t"), IdentPart::Asterisk]);
    }

    #[test]
    fn from_str_mid_star_is_demoted_to_quoted() {
        let ident = Ident::from_str("*.id").unwrap();
        assert_eq!(ident.parts(), &[quote("*"), quote("id")]);
    }

    #[test]
    fn from_str_rejects_empty_and_empty_chunks() {
        assert_eq!(Ident::from_str("").unwrap_err(), ParseIdentError::Empty);
        assert_eq!(
            Ident::from_str("a.").unwrap_err(),
            ParseIdentError::EmptyChunk
        );
        assert_eq!(
            Ident::from_str(".a").unwrap_err(),
            ParseIdentError::EmptyChunk
        );
        assert_eq!(
            Ident::from_str("a..b").unwrap_err(),
            ParseIdentError::EmptyChunk
        );
    }

    #[test]
    fn from_str_rejects_too_many_parts() {
        let ten = (0..Ident::MAX_PARTS)
            .map(|i| format!("p{i}"))
            .collect::<Vec<_>>()
            .join(".");
        assert!(Ident::from_str(&ten).is_ok());

        let eleven = format!("{ten}.extra");
        assert_eq!(
            Ident::from_str(&eleven).unwrap_err(),
            ParseIdentError::TooManyParts
        );
    }

    #[test]
    fn from_parts_rejects_empty() {
        assert_eq!(Ident::from_parts([]).unwrap_err(), ParseIdentError::Empty);
    }

    #[test]
    fn from_parts_rejects_empty_quoted_part() {
        assert_eq!(
            Ident::from_parts([IdentPart::Quote(ArcStr::from(""))]).unwrap_err(),
            ParseIdentError::EmptyChunk
        );
    }

    #[test]
    fn from_parts_rejects_asterisk_not_last() {
        assert_eq!(
            Ident::from_parts([IdentPart::Asterisk, quote("id")]).unwrap_err(),
            ParseIdentError::AsteriskNotLast
        );
    }

    #[test]
    fn unquoted_rejects_invalid() {
        for bad in ["", "*", "a;b", "a.b", "1id", "a-b", "a b", "café"] {
            let err = IdentPart::unquoted(bad).unwrap_err();
            assert!(
                err == ParseIdentError::InvalidUnquoted || err == ParseIdentError::EmptyChunk,
                "{bad:?} -> {err:?}"
            );
        }
    }

    #[test]
    fn unquoted_accepts_simple_ascii_names() {
        for ok in ["Users", "id", "_x", "a1", "created_at"] {
            let part = IdentPart::unquoted(ok).unwrap();
            match part {
                IdentPart::Unquote(n) => assert_eq!(n.as_str(), ok),
                _ => panic!("expected Unquote"),
            }
        }
    }

    #[test]
    fn quoted_star_is_not_asterisk() {
        let part = quote("*");
        assert_ne!(part, IdentPart::Asterisk);
        match part {
            IdentPart::Quote(n) => assert_eq!(n.as_str(), "*"),
            _ => panic!("expected Quote"),
        }
    }

    #[test]
    fn quote_unquote_asterisk_are_not_equal() {
        let q = quote("id");
        let u = IdentPart::unquoted("id").unwrap();
        assert_ne!(q, u);
        assert_ne!(q, IdentPart::Asterisk);
        assert_ne!(u, IdentPart::Asterisk);
        assert_ne!(ident_from([q.clone()]), ident_from([u.clone()]));
    }

    #[test]
    fn from_one_is_single_part() {
        let ident = Ident::from_one(quote("id"));
        assert_eq!(ident.len(), 1);
        assert_eq!(ident.parts(), &[quote("id")]);
    }

    #[test]
    fn parse_ident_error_display_is_not_todo() {
        let msg = ParseIdentError::Empty.to_string();
        assert!(!msg.is_empty());
        assert!(!msg.contains("todo"));
    }

    #[test]
    fn prepare_uses_dialect_delimiters() {
        fn render(ident: &Ident, dialect: SqlDialect) -> String {
            let mut buf = String::new();
            for (i, part) in ident.parts().iter().enumerate() {
                if i > 0 {
                    buf.push('.');
                }
                match part {
                    IdentPart::Quote(n) => {
                        dialect.quote_identifier(&mut buf, n.as_str()).unwrap();
                    }
                    IdentPart::Unquote(n) => buf.push_str(n.as_str()),
                    IdentPart::Asterisk => buf.push('*'),
                }
            }
            buf
        }

        let ident = ident_from([quote("id")]);
        assert_eq!(render(&ident, SqlDialect::PostgreSql), "\"id\"");
        assert_eq!(render(&ident, SqlDialect::MySql), "`id`");
        assert_eq!(render(&ident, SqlDialect::MsSql), "[id]");
        assert_eq!(render(&ident, SqlDialect::Teradata), "'id'");

        let ident = ident_from([quote("a\"b")]);
        assert_eq!(render(&ident, SqlDialect::PostgreSql), "\"a\"\"b\"");

        let ident = ident_from([quote("a`b")]);
        assert_eq!(render(&ident, SqlDialect::MySql), "`a``b`");

        let ident = ident_from([quote("a]b")]);
        assert_eq!(render(&ident, SqlDialect::MsSql), "[a]]b]");

        let ident = ident_from([quote("a'b")]);
        assert_eq!(render(&ident, SqlDialect::Teradata), "'a''b'");

        let ident = ident_from([quote("a; DROP TABLE t--")]);
        assert_eq!(
            render(&ident, SqlDialect::PostgreSql),
            "\"a; DROP TABLE t--\""
        );

        let ident = ident_from([
            quote("public"),
            IdentPart::unquoted("Users").unwrap(),
            quote("id"),
        ]);
        assert_eq!(
            render(&ident, SqlDialect::PostgreSql),
            "\"public\".Users.\"id\""
        );

        let ident = ident_from([quote("t"), IdentPart::Asterisk]);
        assert_eq!(render(&ident, SqlDialect::MySql), "`t`.*");
    }
}
