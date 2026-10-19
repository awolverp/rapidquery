//! `LIKE` / `ILIKE` / `LIKE ANY` / `SIMILAR TO` / `RLIKE` AST nodes.

use super::ExprInner;
use crate::dialect::PyDialect;
use crate::dialect::config;
use crate::internal::repr;
use crate::internal::sql::PrepareResult;
use crate::internal::sql::SqlWriter;
use crate::internal::sql::WriteSql;
use crate::internal::sql::write_separated;

pub(super) fn write_escape<W: SqlWriter>(dialect: &PyDialect, out: &mut W, escape: Option<char>) -> PrepareResult<()> {
    let Some(c) = escape else {
        return Ok(());
    };

    out.push_str(" ESCAPE ");
    let mut buf = [0u8; 4];
    dialect.0.write_string_literal(out, c.encode_utf8(&mut buf))?;

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
    dialect: &PyDialect,
    out: &mut W,
    expr: &ExprInner,
    wrap_lower: bool,
    paren_if_equal: bool,
) -> PrepareResult<()> {
    if wrap_lower {
        out.push_str("LOWER(");
        expr.write_sql(dialect, out)?;
        out.push_char(')');
        Ok(())
    } else {
        expr.write_operand(dialect, out, Like::PRECEDENCE, paren_if_equal)
    }
}

fn write_like_keyword<W: SqlWriter>(out: &mut W, ilike: bool) {
    out.push_str(if ilike { "ILIKE" } else { "LIKE" });
}

fn write_like_infix<W: SqlWriter>(out: &mut W, negated: bool, ilike: bool) {
    out.push_char(' ');
    if negated {
        out.push_str("NOT ");
    }
    write_like_keyword(out, ilike);
    out.push_char(' ');
}

fn write_like_pattern_list<W: SqlWriter>(
    dialect: &PyDialect,
    out: &mut W,
    patterns: &[ExprInner],
    wrap_lower: bool,
) -> PrepareResult<()> {
    if wrap_lower {
        for (i, pattern) in patterns.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
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
fn like_escape_allowed(syntax: config::LikeEscape, native_any: bool) -> bool {
    if native_any {
        syntax.contains(config::LikeEscape::ANY)
    } else {
        syntax.contains(config::LikeEscape::SCALAR)
    }
}

#[inline]
fn ensure_like_escape_is_supported(
    syntax: config::LikeEscape,
    escape: Option<char>,
    native_any: bool,
) -> PrepareResult<()> {
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
    )
    .into())
}

/// `[NOT] (x LIKE|ILIKE ANY (ARRAY?[…]) [ESCAPE char])`
fn write_like_any_native<W: SqlWriter>(
    dialect: &PyDialect,
    out: &mut W,
    like: &Like,
    patterns: &[ExprInner],
    ilike: bool,
    wrap_lower: bool,
    array: bool,
) -> PrepareResult<()> {
    if like.negated {
        out.push_str("NOT (");
    }
    write_like_operand(dialect, out, &like.expr, wrap_lower, false)?;
    out.push_char(' ');
    write_like_keyword(out, ilike);
    if array {
        out.push_str(" ANY (ARRAY[");
    } else {
        out.push_str(" ANY (");
    }
    write_like_pattern_list(dialect, out, patterns, wrap_lower)?;
    if array {
        out.push_str("])");
    } else {
        out.push_char(')');
    }
    write_escape(dialect, out, like.escape)?;
    if like.negated {
        out.push_char(')');
    }
    Ok(())
}

impl WriteSql for Like {
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> PrepareResult<()> {
        let (ilike, wrap_lower) = if self.case_sensitive {
            (false, false)
        } else {
            match dialect.0.ilike_expression {
                config::ILikeExpression::Standard => (true, false),
                config::ILikeExpression::UseLike => (false, false),
                config::ILikeExpression::UseLikeLower => (false, true),
            }
        };

        ensure_like_escape_is_supported(
            dialect.0.like_escape,
            self.escape,
            matches!(self.pattern, LikePattern::Any(_)) && !matches!(dialect.0.like_any, config::LikeAny::OrExpansion),
        )?;

        match &self.pattern {
            LikePattern::One(pattern) => {
                write_like_operand(dialect, out, &self.expr, wrap_lower, false)?;
                write_like_infix(out, self.negated, ilike);
                write_like_operand(dialect, out, pattern, wrap_lower, true)?;
                write_escape(dialect, out, self.escape)
            }
            LikePattern::Any(patterns) => match dialect.0.like_any {
                config::LikeAny::ArrayConstructor => {
                    write_like_any_native(dialect, out, self, patterns, ilike, wrap_lower, true)
                }
                config::LikeAny::PatternList => {
                    write_like_any_native(dialect, out, self, patterns, ilike, wrap_lower, false)
                }
                config::LikeAny::OrExpansion => {
                    let sep = if self.negated { " AND " } else { " OR " };
                    out.push_char('(');
                    for (i, pattern) in patterns.iter().enumerate() {
                        if i > 0 {
                            out.push_str(sep);
                        }
                        write_like_operand(dialect, out, &self.expr, wrap_lower, false)?;
                        write_like_infix(out, self.negated, ilike);
                        write_like_operand(dialect, out, pattern, wrap_lower, true)?;
                        write_escape(dialect, out, self.escape)?;
                    }
                    out.push_char(')');
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
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> PrepareResult<()> {
        if !dialect.0.supports_similar_to {
            return Err(
                rapidquery_macros::pyerr!(PyRuntimeError, "SIMILAR TO is not supported by this dialect",).into(),
            );
        }

        self.expr.write_operand(dialect, out, Self::PRECEDENCE, false)?;
        out.push_char(' ');
        if self.negated {
            out.push_str("NOT SIMILAR TO ");
        } else {
            out.push_str("SIMILAR TO ");
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
    dialect: &PyDialect,
    out: &mut W,
    name: &str,
    negated: bool,
    expr: &ExprInner,
    pattern: &ExprInner,
) -> PrepareResult<()> {
    if negated {
        out.push_str("NOT ");
    }
    out.push_str(name);
    out.push_char('(');
    expr.write_sql(dialect, out)?;
    out.push_str(", ");
    pattern.write_sql(dialect, out)?;
    out.push_char(')');
    Ok(())
}

impl WriteSql for RLike {
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> PrepareResult<()> {
        match dialect.0.rlike_expression {
            config::RLikeExpression::Unsupported => {
                Err(rapidquery_macros::pyerr!(PyRuntimeError, "RLIKE is not supported by this dialect",).into())
            }
            config::RLikeExpression::Standard => {
                self.expr.write_operand(dialect, out, Self::PRECEDENCE, false)?;
                if self.negated {
                    out.push_str(" NOT RLIKE ");
                } else {
                    out.push_str(" RLIKE ");
                }
                self.pattern.write_operand(dialect, out, Self::PRECEDENCE, true)
            }
            config::RLikeExpression::Tilde => {
                self.expr.write_operand(dialect, out, Self::PRECEDENCE, false)?;
                out.push_str(if self.negated { " !~ " } else { " ~ " });
                self.pattern.write_operand(dialect, out, Self::PRECEDENCE, true)
            }
            config::RLikeExpression::RegexpLike => {
                write_regex_call(dialect, out, "REGEXP_LIKE", self.negated, &self.expr, &self.pattern)
            }
            config::RLikeExpression::RegexpContains => {
                write_regex_call(dialect, out, "REGEXP_CONTAINS", self.negated, &self.expr, &self.pattern)
            }
            config::RLikeExpression::RegexpMatches => {
                write_regex_call(dialect, out, "regexp_matches", self.negated, &self.expr, &self.pattern)
            }
        }
    }
}
