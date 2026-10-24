//! `LIKE` / `ILIKE` / `LIKE ANY` / `SIMILAR TO` / `RLIKE` AST nodes.

use super::ExprInner;
use crate::dialect;
use crate::internal::repr;
use crate::sql::SqlWriter;
use crate::sql::WriteSql;
use crate::sql::write_separated;

pub(super) fn write_escape<W: SqlWriter>(
    dialect: &dialect::PyDialect,
    out: &mut W,
    escape: Option<char>,
) -> pyo3::PyResult<()> {
    let Some(c) = escape else {
        return Ok(());
    };

    out.write_str(" ESCAPE ");
    let mut buf = [0u8; 4];
    dialect.string.write_string_literal(out, c.encode_utf8(&mut buf))?;

    Ok(())
}

/// Pattern side of [`Like`]: one expression, or a non-empty ANY list.
#[derive(Clone, Debug, PartialEq)]
pub enum LikePattern {
    /// `col LIKE p`
    One(Box<ExprInner>),
    /// `col LIKE ANY …` — never empty.
    Any(Vec<ExprInner>),
}

impl LikePattern {
    fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<repr::ReprFormatter> {
        match self {
            Self::One(pattern) => pattern.python_repr(py),
            Self::Any(patterns) => {
                let mut fmt = repr::ReprFormatter::new_list();

                for item in patterns {
                    fmt.positional(item.python_repr(py)?);
                }

                fmt.close();
                Ok(fmt)
            }
        }
    }
}

/// Pattern predicate: `LIKE`, `ILIKE`, or `LIKE ANY`, plus an optional escape.
///
/// One pattern is a scalar match. [`LikePattern::Any`] is `LIKE ANY`, spelled
/// as the dialect allows. The escape character is inlined, never bound.
///
/// ```
/// use rapidquery::expr::enum_::ExprInner;
/// use rapidquery::expr::enum_::Ident;
/// use rapidquery::expr::enum_::Like;
/// use rapidquery::expr::enum_::LikePattern;
/// use std::str::FromStr;
///
/// let expr = Like {
///     case_sensitive: true,
///     negated: false,
///     expr: Box::new(ExprInner::Ident(Ident::from_str("name").unwrap())),
///     pattern: LikePattern::One(Box::new(ExprInner::Ident(Ident::from_str("pat").unwrap()))),
///     escape: None,
/// };
/// assert!(expr.escape.is_none());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Like {
    pub case_sensitive: bool,
    pub negated: bool,
    pub expr: Box<ExprInner>,
    pub pattern: LikePattern,
    pub escape: Option<char>,
}

impl Like {
    pub(crate) const PRECEDENCE: u8 = 5;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<repr::ReprFormatter> {
        let mut fmt = repr::ReprFormatter::new_call("LikeExpr");

        fmt.positional(self.expr.python_repr(py)?)
            .positional(self.pattern.python_repr(py)?)
            .keyword_false("case_sensitive", self.case_sensitive)
            .keyword_true("negated", self.negated)
            .keyword("escape", self.escape)
            .close();

        Ok(fmt)
    }
}

fn write_like_operand<W: SqlWriter>(
    dialect: &dialect::PyDialect,
    out: &mut W,
    expr: &ExprInner,
    wrap_lower: bool,
    paren_if_equal: bool,
) -> pyo3::PyResult<()> {
    if wrap_lower {
        out.write_str("LOWER(");
        expr.write_sql(dialect, out)?;
        out.write(')');
        Ok(())
    } else {
        expr.write_operand(dialect, out, Like::PRECEDENCE, paren_if_equal)
    }
}

fn write_like_keyword<W: SqlWriter>(out: &mut W, ilike: bool) {
    out.write_str(if ilike { "ILIKE" } else { "LIKE" });
}

fn write_like_infix<W: SqlWriter>(out: &mut W, negated: bool, ilike: bool) {
    out.write(' ');
    if negated {
        out.write_str("NOT ");
    }
    write_like_keyword(out, ilike);
    out.write(' ');
}

fn write_like_pattern_list<W: SqlWriter>(
    dialect: &dialect::PyDialect,
    out: &mut W,
    patterns: &[ExprInner],
    wrap_lower: bool,
) -> pyo3::PyResult<()> {
    if wrap_lower {
        for (i, pattern) in patterns.iter().enumerate() {
            if i > 0 {
                out.write_str(", ");
            }
            write_like_operand(dialect, out, pattern, true, false)?;
        }
        Ok(())
    } else {
        write_separated(dialect, out, patterns, ", ")
    }
}

/// `ESCAPE` is legal on this form. `native_any` is `LIKE ANY` that is not an
/// `OR` expansion.
#[inline]
fn like_escape_allowed(syntax: dialect::LikeEscape, native_any: bool) -> bool {
    if native_any {
        syntax.contains(dialect::LikeEscape::ANY)
    } else {
        syntax.contains(dialect::LikeEscape::SCALAR)
    }
}

#[inline]
fn ensure_like_escape_is_supported(
    syntax: dialect::LikeEscape,
    escape: Option<char>,
    native_any: bool,
) -> pyo3::PyResult<()> {
    if escape.is_none() || like_escape_allowed(syntax, native_any) {
        return Ok(());
    }

    Err(rapidquery_macros::pyerr!(
        PyRuntimeError,
        if native_any {
            "LIKE ANY ESCAPE is not supported by this dialect"
        } else {
            "LIKE ESCAPE is not supported by this dialect"
        }
    ))
}

/// `[NOT] (x LIKE|ILIKE ANY (ARRAY?[…]) [ESCAPE char])`
fn write_like_any_native<W: SqlWriter>(
    dialect: &dialect::PyDialect,
    out: &mut W,
    like: &Like,
    patterns: &[ExprInner],
    ilike: bool,
    wrap_lower: bool,
    array: bool,
) -> pyo3::PyResult<()> {
    if like.negated {
        out.write_str("NOT (");
    }
    write_like_operand(dialect, out, &like.expr, wrap_lower, false)?;
    out.write(' ');
    write_like_keyword(out, ilike);
    if array {
        out.write_str(" ANY (ARRAY[");
    } else {
        out.write_str(" ANY (");
    }
    write_like_pattern_list(dialect, out, patterns, wrap_lower)?;
    if array {
        out.write_str("])");
    } else {
        out.write(')');
    }
    write_escape(dialect, out, like.escape)?;
    if like.negated {
        out.write(')');
    }
    Ok(())
}

impl WriteSql for Like {
    fn write_sql<W: SqlWriter>(&self, dialect: &dialect::PyDialect, out: &mut W) -> pyo3::PyResult<()> {
        let (ilike, wrap_lower) = if self.case_sensitive {
            (false, false)
        } else {
            match dialect.like.ilike {
                dialect::ILikeStrategy::Unsupported => {
                    return Err(rapidquery_macros::pyerr!(
                        PyRuntimeError,
                        "ILIKE is not supported by this dialect"
                    ));
                }
                dialect::ILikeStrategy::Standard => (true, false),
                dialect::ILikeStrategy::UseLikeLower => (false, true),
            }
        };

        ensure_like_escape_is_supported(
            dialect.like.escape,
            self.escape,
            matches!(self.pattern, LikePattern::Any(_)) && !matches!(dialect.like.any, dialect::LikeAny::OrExpansion),
        )?;

        match &self.pattern {
            LikePattern::One(pattern) => {
                write_like_operand(dialect, out, &self.expr, wrap_lower, false)?;
                write_like_infix(out, self.negated, ilike);
                write_like_operand(dialect, out, pattern, wrap_lower, true)?;
                write_escape(dialect, out, self.escape)
            }
            LikePattern::Any(patterns) => match dialect.like.any {
                dialect::LikeAny::ArrayConstructor => {
                    write_like_any_native(dialect, out, self, patterns, ilike, wrap_lower, true)
                }
                dialect::LikeAny::PatternList => {
                    write_like_any_native(dialect, out, self, patterns, ilike, wrap_lower, false)
                }
                dialect::LikeAny::OrExpansion => {
                    let sep = if self.negated { " AND " } else { " OR " };
                    out.write('(');
                    for (i, pattern) in patterns.iter().enumerate() {
                        if i > 0 {
                            out.write_str(sep);
                        }
                        write_like_operand(dialect, out, &self.expr, wrap_lower, false)?;
                        write_like_infix(out, self.negated, ilike);
                        write_like_operand(dialect, out, pattern, wrap_lower, true)?;
                        write_escape(dialect, out, self.escape)?;
                    }
                    out.write(')');
                    Ok(())
                }
            },
        }
    }
}

/// `expr [NOT] SIMILAR TO pattern`.
///
/// Rendered only when the dialect sets `supports_similar_to`. There is no
/// `ESCAPE` clause. Other dialects fail at render time.
///
/// ```
/// use rapidquery::expr::enum_::ExprInner;
/// use rapidquery::expr::enum_::Ident;
/// use rapidquery::expr::enum_::SimilarTo;
/// use std::str::FromStr;
///
/// let expr = SimilarTo {
///     negated: false,
///     expr: Box::new(ExprInner::Ident(Ident::from_str("name").unwrap())),
///     pattern: Box::new(ExprInner::Ident(Ident::from_str("pat").unwrap())),
/// };
/// assert!(!expr.negated);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct SimilarTo {
    pub negated: bool,
    pub expr: Box<ExprInner>,
    pub pattern: Box<ExprInner>,
}

impl SimilarTo {
    pub(crate) const PRECEDENCE: u8 = 5;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<repr::ReprFormatter> {
        let mut fmt = repr::ReprFormatter::new_call("SimilarToExpr");

        fmt.positional(self.expr.python_repr(py)?)
            .positional(self.pattern.python_repr(py)?)
            .keyword_true("negated", self.negated)
            .close();

        Ok(fmt)
    }
}

impl WriteSql for SimilarTo {
    fn write_sql<W: SqlWriter>(&self, dialect: &dialect::PyDialect, out: &mut W) -> pyo3::PyResult<()> {
        if !dialect.like.supports_similar_to {
            return Err(rapidquery_macros::pyerr!(
                PyRuntimeError,
                "SIMILAR TO is not supported by this dialect"
            ));
        }

        self.expr.write_operand(dialect, out, Self::PRECEDENCE, false)?;
        out.write(' ');
        if self.negated {
            out.write_str("NOT SIMILAR TO ");
        } else {
            out.write_str("SIMILAR TO ");
        }
        self.pattern.write_operand(dialect, out, Self::PRECEDENCE, true)
    }
}

/// Regex predicate. The dialect chooses `RLIKE`, `~`, or a function.
///
/// Dialects with no regex predicate fail at render time. There is no `ESCAPE`
/// clause. Function names written for this node stay unquoted.
///
/// ```
/// use rapidquery::expr::enum_::ExprInner;
/// use rapidquery::expr::enum_::Ident;
/// use rapidquery::expr::enum_::RLike;
/// use std::str::FromStr;
///
/// let expr = RLike {
///     negated: false,
///     expr: Box::new(ExprInner::Ident(Ident::from_str("name").unwrap())),
///     pattern: Box::new(ExprInner::Ident(Ident::from_str("pat").unwrap())),
/// };
/// assert!(!expr.negated);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct RLike {
    pub negated: bool,
    pub expr: Box<ExprInner>,
    pub pattern: Box<ExprInner>,
}

impl RLike {
    pub(crate) const PRECEDENCE: u8 = 5;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<repr::ReprFormatter> {
        let mut fmt = repr::ReprFormatter::new_call("RLikeExpr");

        fmt.positional(self.expr.python_repr(py)?)
            .positional(self.pattern.python_repr(py)?)
            .keyword_true("negated", self.negated)
            .close();

        Ok(fmt)
    }
}

/// `NOT? name(expr, pattern)` — unquoted function rewrite.
fn write_regex_call<W: SqlWriter>(
    dialect: &dialect::PyDialect,
    out: &mut W,
    name: &str,
    negated: bool,
    expr: &ExprInner,
    pattern: &ExprInner,
) -> pyo3::PyResult<()> {
    if negated {
        out.write_str("NOT ");
    }
    out.write_str(name);
    out.write('(');
    expr.write_sql(dialect, out)?;
    out.write_str(", ");
    pattern.write_sql(dialect, out)?;
    out.write(')');
    Ok(())
}

impl WriteSql for RLike {
    fn write_sql<W: SqlWriter>(&self, dialect: &dialect::PyDialect, out: &mut W) -> pyo3::PyResult<()> {
        match dialect.like.regex {
            dialect::RegexSpelling::Unsupported => Err(rapidquery_macros::pyerr!(
                PyRuntimeError,
                "RLIKE is not supported by this dialect",
            )),
            dialect::RegexSpelling::Standard => {
                self.expr.write_operand(dialect, out, Self::PRECEDENCE, false)?;
                if self.negated {
                    out.write_str(" NOT RLIKE ");
                } else {
                    out.write_str(" RLIKE ");
                }
                self.pattern.write_operand(dialect, out, Self::PRECEDENCE, true)
            }
            dialect::RegexSpelling::Tilde => {
                self.expr.write_operand(dialect, out, Self::PRECEDENCE, false)?;
                out.write_str(if self.negated { " !~ " } else { " ~ " });
                self.pattern.write_operand(dialect, out, Self::PRECEDENCE, true)
            }
            dialect::RegexSpelling::RegexpLike => {
                write_regex_call(dialect, out, "REGEXP_LIKE", self.negated, &self.expr, &self.pattern)
            }
            dialect::RegexSpelling::RegexpContains => {
                write_regex_call(dialect, out, "REGEXP_CONTAINS", self.negated, &self.expr, &self.pattern)
            }
            dialect::RegexSpelling::RegexpMatches => {
                write_regex_call(dialect, out, "regexp_matches", self.negated, &self.expr, &self.pattern)
            }
        }
    }
}
