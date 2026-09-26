//! `LIKE` / `ILIKE` / `LIKE ANY` / `SIMILAR TO` / `RLIKE` AST nodes.

use super::{ExprInner, nodes::write_escape};
use crate::dialect::{
    PySqlDialect,
    inner::{LikeAnySyntax, RLikeExpressionOptions},
};
use crate::sql::{PrepareResult, SqlWriter, WriteSql, write_separated};
use std::fmt::Write;

/// Pattern side of [`Like`]: one expression, or a non-empty ANY list.
#[derive(Clone, Debug, PartialEq)]
pub enum LikePattern {
    /// `col LIKE p`
    One(Box<ExprInner>),
    /// `col LIKE ANY …` — never empty.
    Any(Vec<ExprInner>),
}

impl LikePattern {
    fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        match self {
            Self::One(p) => p.python_repr(py),
            Self::Any(ps) => {
                let mut buf = String::from("[");
                for (i, item) in ps.iter().enumerate() {
                    if i > 0 {
                        buf.push_str(", ");
                    }
                    buf.push_str(&item.python_repr(py)?);
                }
                buf.push(']');
                Ok(buf)
            }
        }
    }
}

/// `expr [NOT] LIKE / ILIKE [ANY] pattern [ESCAPE char]`.
///
/// `case_sensitive == false` follows the dialect's `ilike_expression_options`
/// (`ILIKE`, plain `LIKE`, or `LOWER(expr) LIKE LOWER(pattern)`). `ESCAPE` is
/// a single inlined character (never a bind).
///
/// LIKE ANY follows [`LikeAnySyntax`]: Postgres `ANY (ARRAY[…])` wraps
/// negation as `NOT (x LIKE ANY …)` because `NOT LIKE ANY` is a quantified
/// NOT LIKE; Snowflake/Spark/Databricks use `LIKE ANY (p, …)`; everyone else
/// expands to parenthesized `OR` of scalar `LIKE`s (`AND` of `NOT LIKE` when
/// negated).
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

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = format!(
            "LikeExpr({}, {}",
            self.expr.python_repr(py)?,
            self.pattern.python_repr(py)?
        );
        if !self.case_sensitive {
            buf.push_str(", case_sensitive=False");
        }
        if self.negated {
            buf.push_str(", negated=True");
        }
        if let Some(c) = self.escape {
            write!(buf, ", escape={c:?}").unwrap();
        }
        buf.push(')');
        Ok(buf)
    }
}

fn write_like_operand<W: SqlWriter>(
    dialect: &PySqlDialect,
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
    dialect: &PySqlDialect,
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

/// `[NOT] (x LIKE|ILIKE ANY (ARRAY?[…]))`
fn write_like_any_native<W: SqlWriter>(
    dialect: &PySqlDialect,
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
    if like.negated {
        out.push_char(')');
    }
    Ok(())
}

impl WriteSql for Like {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        let (ilike, wrap_lower) = if self.case_sensitive {
            (false, false)
        } else {
            dialect.0.ilike_expression_options.case_insensitive_sql()
        };

        match &self.pattern {
            LikePattern::One(pattern) => {
                write_like_operand(dialect, out, &self.expr, wrap_lower, false)?;
                write_like_infix(out, self.negated, ilike);
                write_like_operand(dialect, out, pattern, wrap_lower, true)?;
                write_escape(dialect, out, self.escape)
            }
            LikePattern::Any(patterns) => match dialect.0.like_any_syntax {
                LikeAnySyntax::ArrayConstructor => {
                    write_like_any_native(dialect, out, self, patterns, ilike, wrap_lower, true)
                }
                LikeAnySyntax::PatternList => {
                    write_like_any_native(dialect, out, self, patterns, ilike, wrap_lower, false)
                }
                LikeAnySyntax::OrExpansion => {
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
/// Emitted only when [`Dialect::supports_similar_to`](crate::dialect::inner::Dialect::supports_similar_to)
/// is set (PostgreSQL, Redshift, DuckDB). Other dialects raise.
///
/// No `ESCAPE` clause: regex engines use `\` (and dialect string-literal
/// rules). See [PostgreSQL pattern matching](https://www.postgresql.org/docs/current/functions-matching.html),
/// [Redshift SIMILAR TO](https://docs.aws.amazon.com/redshift/latest/dg/pattern-matching-conditions-similar-to.html),
/// [DuckDB pattern matching](https://duckdb.org/docs/current/sql/functions/pattern_matching.html).
#[derive(Clone, Debug, PartialEq)]
pub struct SimilarTo {
    pub negated: bool,
    pub expr: Box<ExprInner>,
    pub pattern: Box<ExprInner>,
}

impl SimilarTo {
    pub(crate) const PRECEDENCE: u8 = 5;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = format!(
            "SimilarToExpr({}, {}",
            self.expr.python_repr(py)?,
            self.pattern.python_repr(py)?
        );
        if self.negated {
            buf.push_str(", negated=True");
        }
        buf.push(')');
        Ok(buf)
    }
}

impl WriteSql for SimilarTo {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        if !dialect.0.supports_similar_to {
            return Err(rapidquery_macros::pyerr!(
                PyRuntimeError,
                "SIMILAR TO is not supported by this dialect",
            )
            .into());
        }

        self.expr
            .write_operand(dialect, out, Self::PRECEDENCE, false)?;
        out.push_char(' ');
        if self.negated {
            out.push_str("NOT SIMILAR TO ");
        } else {
            out.push_str("SIMILAR TO ");
        }
        self.pattern
            .write_operand(dialect, out, Self::PRECEDENCE, true)
    }
}

/// Regex match: spelling follows [`RLikeExpressionOptions`].
///
/// - MySQL / Hive / Spark / Databricks / Snowflake: `expr [NOT] RLIKE pattern`
///   ([MySQL REGEXP](https://dev.mysql.com/doc/refman/8.4/en/regexp.html),
///   [Spark LIKE/RLIKE](https://spark.apache.org/docs/latest/sql-ref-syntax-qry-select-like.html),
///   [Snowflake RLIKE](https://docs.snowflake.com/en/sql-reference/functions/rlike)).
///   Snowflake implicitly anchors the pattern (`^…$`); that rewrite is later.
/// - PostgreSQL / Redshift: `expr ~ pattern` / `expr !~ pattern`
///   ([PostgreSQL POSIX](https://www.postgresql.org/docs/current/functions-matching.html)).
/// - Oracle: `REGEXP_LIKE(expr, pattern)`.
/// - BigQuery: `REGEXP_CONTAINS(expr, pattern)`.
/// - DuckDB: `regexp_matches(expr, pattern)` (substring; DuckDB `~` is full match).
/// - MsSql / Sqlite / Teradata: raise.
///
/// Function names are unquoted. No `ESCAPE` clause.
#[derive(Clone, Debug, PartialEq)]
pub struct RLike {
    pub negated: bool,
    pub expr: Box<ExprInner>,
    pub pattern: Box<ExprInner>,
}

impl RLike {
    pub(crate) const PRECEDENCE: u8 = 5;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = format!(
            "RLikeExpr({}, {}",
            self.expr.python_repr(py)?,
            self.pattern.python_repr(py)?
        );
        if self.negated {
            buf.push_str(", negated=True");
        }
        buf.push(')');
        Ok(buf)
    }
}

/// `NOT? name(expr, pattern)` — unquoted function rewrite.
fn write_regex_call<W: SqlWriter>(
    dialect: &PySqlDialect,
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
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        let Some(opt) = dialect.0.rlike_expression_options else {
            return Err(rapidquery_macros::pyerr!(
                PyRuntimeError,
                "RLIKE is not supported by this dialect",
            )
            .into());
        };

        match opt {
            RLikeExpressionOptions::InfixRLike => {
                self.expr
                    .write_operand(dialect, out, Self::PRECEDENCE, false)?;
                if self.negated {
                    out.push_str(" NOT RLIKE ");
                } else {
                    out.push_str(" RLIKE ");
                }
                self.pattern
                    .write_operand(dialect, out, Self::PRECEDENCE, true)
            }
            RLikeExpressionOptions::Tilde => {
                self.expr
                    .write_operand(dialect, out, Self::PRECEDENCE, false)?;
                out.push_str(if self.negated { " !~ " } else { " ~ " });
                self.pattern
                    .write_operand(dialect, out, Self::PRECEDENCE, true)
            }
            RLikeExpressionOptions::RegexpLike => write_regex_call(
                dialect,
                out,
                "REGEXP_LIKE",
                self.negated,
                &self.expr,
                &self.pattern,
            ),
            RLikeExpressionOptions::RegexpContains => write_regex_call(
                dialect,
                out,
                "REGEXP_CONTAINS",
                self.negated,
                &self.expr,
                &self.pattern,
            ),
            RLikeExpressionOptions::RegexpMatches => write_regex_call(
                dialect,
                out,
                "regexp_matches",
                self.negated,
                &self.expr,
                &self.pattern,
            ),
        }
    }
}
