use super::{
    BinaryOp, CastKind, ExprInner, Ident, IsKind, NormalizationForm, TrimWhere, UnaryOp,
    UnarySpacing,
};
use crate::dialect::{PySqlDialect, inner::LikeAnySyntax};
use crate::sql::{PrepareResult, SqlWriter, WriteSql, write_separated};
use std::fmt::Write;

fn join_reprs(py: pyo3::Python<'_>, buf: &mut String, items: &[ExprInner]) -> pyo3::PyResult<()> {
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            buf.push_str(", ");
        }
        buf.push_str(&item.python_repr(py)?);
    }
    Ok(())
}

fn write_escape<W: SqlWriter>(
    dialect: &PySqlDialect,
    out: &mut W,
    escape: Option<char>,
) -> PrepareResult<()> {
    let Some(c) = escape else {
        return Ok(());
    };

    out.push_str(" ESCAPE ");
    let mut buf = [0u8; 4];
    dialect
        .0
        .write_string_literal(out, c.encode_utf8(&mut buf))
        .map_err(|_| crate::sql::PrepareError::Write)?;

    Ok(())
}

/// Parenthesized row `(a, b, …)`. No trailing comma on a singleton.
#[derive(Clone, Debug, PartialEq)]
pub struct Tuple {
    pub items: Vec<ExprInner>,
}

impl Tuple {
    pub(crate) const PRECEDENCE: u8 = 0;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = String::from("TupleExpr(");
        join_reprs(py, &mut buf, &self.items)?;
        buf.push(')');
        Ok(buf)
    }
}

impl WriteSql for Tuple {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        out.push_char('(');
        write_separated(dialect, out, &self.items, ", ")?;
        out.push_char(')');
        Ok(())
    }
}

/// Unary operation (`NOT x`, `-x`, `|/x`, postfix `x!`).
///
/// Precedence 1. Nested equal-precedence operands are parenthesized.
#[derive(Clone, Debug, PartialEq)]
pub struct Unary {
    pub op: UnaryOp,
    pub expr: Box<ExprInner>,
}

impl Unary {
    pub(crate) const PRECEDENCE: u8 = 1;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        Ok(format!(
            "UnaryExpr({:?}, {})",
            self.op.as_sql(),
            self.expr.python_repr(py)?
        ))
    }
}

impl WriteSql for Unary {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        match self.op.spacing() {
            UnarySpacing::Postfix => {
                self.expr
                    .write_operand(dialect, out, Self::PRECEDENCE, true)?;
                out.push_char('!');
                Ok(())
            }
            UnarySpacing::SpacedPrefix => {
                out.push_str(self.op.as_sql());
                out.push_char(' ');
                self.expr
                    .write_operand(dialect, out, Self::PRECEDENCE, true)
            }
            UnarySpacing::GluedPrefix => {
                out.push_str(self.op.as_sql());
                self.expr
                    .write_operand(dialect, out, Self::PRECEDENCE, true)
            }
        }
    }
}

/// Binary operation (`a + b`, `a AND b`, custom operators).
///
/// `!=` is stored and emitted as `<>`. Precedence follows [`BinaryOp::precedence`].
/// The right operand is parenthesized when it has equal precedence.
#[derive(Clone, Debug, PartialEq)]
pub struct Binary {
    pub left: Box<ExprInner>,
    pub op: BinaryOp,
    pub right: Box<ExprInner>,
}

impl Binary {
    #[inline]
    pub(crate) fn precedence(&self) -> u8 {
        self.op.precedence()
    }

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        Ok(format!(
            "BinaryExpr({}, {:?}, {})",
            self.left.python_repr(py)?,
            self.op.as_sql(),
            self.right.python_repr(py)?
        ))
    }
}

impl WriteSql for Binary {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        let prec = self.precedence();
        self.left.write_operand(dialect, out, prec, false)?;
        out.push_char(' ');
        out.push_str(self.op.as_sql());
        out.push_char(' ');
        self.right.write_operand(dialect, out, prec, true)
    }
}

/// `expr IS [NOT] {NULL|TRUE|FALSE|UNKNOWN}`.
#[derive(Clone, Debug, PartialEq)]
pub struct Is {
    pub expr: Box<ExprInner>,
    pub negated: bool,
    pub kind: IsKind,
}

impl Is {
    pub(crate) const PRECEDENCE: u8 = 5;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = format!(
            "IsExpr({}, {:?}",
            self.expr.python_repr(py)?,
            self.kind.as_sql()
        );
        if self.negated {
            buf.push_str(", negated=True");
        }
        buf.push(')');
        Ok(buf)
    }
}

impl WriteSql for Is {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        self.expr
            .write_operand(dialect, out, Self::PRECEDENCE, false)?;
        if self.negated {
            out.push_str(" IS NOT ");
        } else {
            out.push_str(" IS ");
        }
        out.push_str(self.kind.as_sql());
        Ok(())
    }
}

/// `expr IS [NOT] [form] NORMALIZED`.
///
/// When [`form`](Self::form) is `None`, the form token is omitted (`IS NORMALIZED`).
#[derive(Clone, Debug, PartialEq)]
pub struct IsNormalized {
    pub expr: Box<ExprInner>,
    pub negated: bool,
    pub form: Option<NormalizationForm>,
}

impl IsNormalized {
    pub(crate) const PRECEDENCE: u8 = 5;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = format!("IsNormalizedExpr({}", self.expr.python_repr(py)?);
        if let Some(form) = self.form {
            write!(buf, ", {:?}", form.as_sql()).unwrap();
        }
        if self.negated {
            buf.push_str(", negated=True");
        }
        buf.push(')');
        Ok(buf)
    }
}

impl WriteSql for IsNormalized {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        self.expr
            .write_operand(dialect, out, Self::PRECEDENCE, false)?;
        if self.negated {
            out.push_str(" IS NOT ");
        } else {
            out.push_str(" IS ");
        }
        if let Some(form) = self.form {
            out.push_str(form.as_sql());
            out.push_char(' ');
        }
        out.push_str("NORMALIZED");
        Ok(())
    }
}

/// `left IS [NOT] DISTINCT FROM right`.
#[derive(Clone, Debug, PartialEq)]
pub struct IsDistinctFrom {
    pub left: Box<ExprInner>,
    pub right: Box<ExprInner>,
    pub negated: bool,
}

impl IsDistinctFrom {
    pub(crate) const PRECEDENCE: u8 = 5;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = format!(
            "IsDistinctFromExpr({}, {}",
            self.left.python_repr(py)?,
            self.right.python_repr(py)?
        );
        if self.negated {
            buf.push_str(", negated=True");
        }
        buf.push(')');
        Ok(buf)
    }
}

impl WriteSql for IsDistinctFrom {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        self.left
            .write_operand(dialect, out, Self::PRECEDENCE, false)?;
        if self.negated {
            out.push_str(" IS NOT DISTINCT FROM ");
        } else {
            out.push_str(" IS DISTINCT FROM ");
        }
        self.right
            .write_operand(dialect, out, Self::PRECEDENCE, true)
    }
}

/// `expr [NOT] IN (list…)`.
///
/// An empty list emits `IN ()`.
#[derive(Clone, Debug, PartialEq)]
pub struct InList {
    pub expr: Box<ExprInner>,
    pub list: Vec<ExprInner>,
    pub negated: bool,
}

impl InList {
    pub(crate) const PRECEDENCE: u8 = 5;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = format!("InListExpr({}, (", self.expr.python_repr(py)?);
        join_reprs(py, &mut buf, &self.list)?;
        buf.push(')');
        if self.negated {
            buf.push_str(", negated=True");
        }
        buf.push(')');
        Ok(buf)
    }
}

impl WriteSql for InList {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        self.expr
            .write_operand(dialect, out, Self::PRECEDENCE, false)?;
        if self.negated {
            out.push_str(" NOT IN (");
        } else {
            out.push_str(" IN (");
        }
        write_separated(dialect, out, &self.list, ", ")?;
        out.push_char(')');
        Ok(())
    }
}

/// `expr [NOT] BETWEEN low AND high`.
///
/// No `SYMMETRIC` clause.
#[derive(Clone, Debug, PartialEq)]
pub struct Between {
    pub expr: Box<ExprInner>,
    pub negated: bool,
    pub low: Box<ExprInner>,
    pub high: Box<ExprInner>,
}

impl Between {
    pub(crate) const PRECEDENCE: u8 = 5;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = format!(
            "BetweenExpr({}, {}, {}",
            self.expr.python_repr(py)?,
            self.low.python_repr(py)?,
            self.high.python_repr(py)?
        );
        if self.negated {
            buf.push_str(", negated=True");
        }
        buf.push(')');
        Ok(buf)
    }
}

impl WriteSql for Between {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        self.expr
            .write_operand(dialect, out, Self::PRECEDENCE, false)?;
        if self.negated {
            out.push_str(" NOT BETWEEN ");
        } else {
            out.push_str(" BETWEEN ");
        }
        self.low
            .write_operand(dialect, out, Self::PRECEDENCE, true)?;
        out.push_str(" AND ");
        self.high
            .write_operand(dialect, out, Self::PRECEDENCE, true)
    }
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
    fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        match self {
            Self::One(p) => p.python_repr(py),
            Self::Any(ps) => {
                let mut buf = String::from("[");
                join_reprs(py, &mut buf, ps)?;
                buf.push(']');
                Ok(buf)
            }
        }
    }
}

/// `expr [NOT] LIKE / ILIKE [ANY] pattern [ESCAPE char]`.
///
/// `case_sensitive == false` emits `ILIKE`. `ESCAPE` is a single inlined
/// character (never a bind).
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

fn write_like_op<W: SqlWriter>(out: &mut W, case_sensitive: bool, negated: bool) {
    if negated {
        out.push_str("NOT ");
    }
    if case_sensitive {
        out.push_str("LIKE");
    } else {
        out.push_str("ILIKE");
    }
}

impl WriteSql for Like {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        let prec = Self::PRECEDENCE;
        match &self.pattern {
            LikePattern::One(pattern) => {
                self.expr.write_operand(dialect, out, prec, false)?;
                out.push_char(' ');
                write_like_op(out, self.case_sensitive, self.negated);
                out.push_char(' ');
                pattern.write_operand(dialect, out, prec, true)?;
                write_escape(dialect, out, self.escape)
            }
            LikePattern::Any(patterns) => match dialect.0.like_any_syntax {
                LikeAnySyntax::ArrayConstructor => {
                    // `NOT LIKE ANY` in Postgres is a quantified NOT LIKE
                    // (exists a non-match), not "matches none".
                    if self.negated {
                        out.push_str("NOT (");
                    }
                    self.expr.write_operand(dialect, out, prec, false)?;
                    out.push_char(' ');
                    write_like_op(out, self.case_sensitive, false);
                    out.push_str(" ANY (ARRAY[");
                    write_separated(dialect, out, patterns, ", ")?;
                    out.push_str("])");
                    write_escape(dialect, out, self.escape)?;
                    if self.negated {
                        out.push_char(')');
                    }
                    Ok(())
                }
                LikeAnySyntax::PatternList => {
                    self.expr.write_operand(dialect, out, prec, false)?;
                    out.push_char(' ');
                    write_like_op(out, self.case_sensitive, self.negated);
                    out.push_str(" ANY (");
                    write_separated(dialect, out, patterns, ", ")?;
                    out.push_char(')');
                    write_escape(dialect, out, self.escape)
                }
                LikeAnySyntax::OrExpansion => {
                    out.push_char('(');
                    let sep = if self.negated { " AND " } else { " OR " };
                    for (i, pattern) in patterns.iter().enumerate() {
                        if i > 0 {
                            out.push_str(sep);
                        }
                        self.expr.write_operand(dialect, out, prec, false)?;
                        out.push_char(' ');
                        write_like_op(out, self.case_sensitive, self.negated);
                        out.push_char(' ');
                        pattern.write_operand(dialect, out, prec, true)?;
                        write_escape(dialect, out, self.escape)?;
                    }
                    out.push_char(')');
                    Ok(())
                }
            },
        }
    }
}

/// `expr [NOT] SIMILAR TO pattern [ESCAPE char]`.
#[derive(Clone, Debug, PartialEq)]
pub struct SimilarTo {
    pub negated: bool,
    pub expr: Box<ExprInner>,
    pub pattern: Box<ExprInner>,
    pub escape: Option<char>,
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
        if let Some(c) = self.escape {
            write!(buf, ", escape={c:?}").unwrap();
        }
        buf.push(')');
        Ok(buf)
    }
}

impl WriteSql for SimilarTo {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        self.expr
            .write_operand(dialect, out, Self::PRECEDENCE, false)?;
        out.push_char(' ');
        if self.negated {
            out.push_str("NOT SIMILAR TO ");
        } else {
            out.push_str("SIMILAR TO ");
        }

        self.pattern
            .write_operand(dialect, out, Self::PRECEDENCE, true)?;
        write_escape(dialect, out, self.escape)
    }
}

/// `expr [NOT] RLIKE pattern [ESCAPE char]`.
#[derive(Clone, Debug, PartialEq)]
pub struct RLike {
    pub negated: bool,
    pub expr: Box<ExprInner>,
    pub pattern: Box<ExprInner>,
    pub escape: Option<char>,
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
        if let Some(c) = self.escape {
            write!(buf, ", escape={c:?}").unwrap();
        }
        buf.push(')');
        Ok(buf)
    }
}

impl WriteSql for RLike {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        self.expr
            .write_operand(dialect, out, Self::PRECEDENCE, false)?;
        out.push_char(' ');
        if self.negated {
            out.push_str("NOT RLIKE ");
        } else {
            out.push_str("RLIKE ");
        }

        self.pattern
            .write_operand(dialect, out, Self::PRECEDENCE, true)?;
        write_escape(dialect, out, self.escape)
    }
}

/// `KIND(expr AS type [FORMAT format [AT TIME ZONE tz]])`.
///
/// [`kind`](Self::kind) is `CAST` / `TRY_CAST` / `SAFE_CAST`. `FORMAT` and
/// `AT TIME ZONE` are omitted when unset; timezone requires format.
#[derive(Clone, Debug, PartialEq)]
pub struct Cast {
    pub kind: CastKind,
    pub expr: Box<ExprInner>,
    pub data_type_name: crate::data_type::DataTypeName,
    pub format: Option<Box<ExprInner>>,
    pub at_time_zone: Option<Box<ExprInner>>,
}

impl Cast {
    pub(crate) const PRECEDENCE: u8 = 0;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = format!(
            "CastExpr({}, {:?}",
            self.expr.python_repr(py)?,
            self.data_type_name.as_str()
        );
        if self.kind != CastKind::Cast {
            write!(buf, ", kind={:?}", self.kind.as_sql()).unwrap();
        }
        if let Some(format) = &self.format {
            write!(buf, ", format={}", format.python_repr(py)?).unwrap();
        }
        if let Some(tz) = &self.at_time_zone {
            write!(buf, ", at_time_zone={}", tz.python_repr(py)?).unwrap();
        }
        buf.push(')');
        Ok(buf)
    }
}

impl WriteSql for Cast {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        out.push_str(self.kind.as_sql());
        out.push_char('(');
        self.expr.write_sql(dialect, out)?;
        out.push_str(" AS ");
        self.data_type_name.write_sql(dialect, out)?;
        if let Some(format) = &self.format {
            out.push_str(" FORMAT ");
            format.write_sql(dialect, out)?;
            if let Some(tz) = &self.at_time_zone {
                out.push_str(" AT TIME ZONE ");
                tz.write_sql(dialect, out)?;
            }
        }
        out.push_char(')');
        Ok(())
    }
}

/// `TRIM([BOTH|LEADING|TRAILING] [<what> FROM] <expr>[, chars…])`.
///
/// [`what`](Self::what) and [`characters`](Self::characters) are mutually
/// exclusive. `characters` is never empty when `Some`.
#[derive(Clone, Debug, PartialEq)]
pub struct Trim {
    pub expr: Box<ExprInner>,
    pub where_: Option<TrimWhere>,
    pub what: Option<Box<ExprInner>>,
    /// Comma-list form `TRIM(expr, c1, c2)`. Never empty when `Some`.
    pub characters: Option<Vec<ExprInner>>,
}

impl Trim {
    pub(crate) const PRECEDENCE: u8 = 0;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = format!("TrimExpr({}", self.expr.python_repr(py)?);
        if let Some(where_) = self.where_ {
            write!(buf, ", where={:?}", where_.as_sql()).unwrap();
        }
        if let Some(what) = &self.what {
            write!(buf, ", what={}", what.python_repr(py)?).unwrap();
        }
        if let Some(characters) = &self.characters {
            buf.push_str(", characters=[");
            join_reprs(py, &mut buf, characters)?;
            buf.push(']');
        }
        buf.push(')');
        Ok(buf)
    }
}

impl WriteSql for Trim {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        out.push_str("TRIM(");
        if let Some(where_) = self.where_ {
            out.push_str(where_.as_sql());
            out.push_char(' ');
        }
        if let Some(what) = &self.what {
            what.write_sql(dialect, out)?;
            out.push_str(" FROM ");
            self.expr.write_sql(dialect, out)?;
        } else if self.where_.is_some() {
            out.push_str("FROM ");
            self.expr.write_sql(dialect, out)?;
        } else {
            self.expr.write_sql(dialect, out)?;
        }
        if let Some(characters) = &self.characters {
            for c in characters {
                out.push_str(", ");
                c.write_sql(dialect, out)?;
            }
        }
        out.push_char(')');
        Ok(())
    }
}

/// `expr COLLATE collation`.
///
/// Precedence 0, so a binary operand is parenthesized.
#[derive(Clone, Debug, PartialEq)]
pub struct Collate {
    pub expr: Box<ExprInner>,
    pub collation: Ident,
}

impl Collate {
    pub(crate) const PRECEDENCE: u8 = 0;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        Ok(format!(
            "CollateExpr({}, {})",
            self.expr.python_repr(py)?,
            self.collation.python_repr()
        ))
    }
}

impl WriteSql for Collate {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        self.expr
            .write_operand(dialect, out, Self::PRECEDENCE, false)?;
        out.push_str(" COLLATE ");
        self.collation.write_sql(dialect, out)
    }
}

/// `CASE [operand] WHEN … THEN … [ELSE …] END`.
#[derive(Clone, Debug, PartialEq)]
pub struct Case {
    pub operand: Option<Box<ExprInner>>,
    pub when: Vec<(ExprInner, ExprInner)>,
    pub else_result: Option<Box<ExprInner>>,
}

impl Case {
    pub(crate) const PRECEDENCE: u8 = 0;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = String::from("CaseExpr([");
        for (i, (when, then)) in self.when.iter().enumerate() {
            if i > 0 {
                buf.push_str(", ");
            }
            write!(
                buf,
                "({}, {})",
                when.python_repr(py)?,
                then.python_repr(py)?
            )
            .unwrap();
        }
        buf.push(']');
        if let Some(operand) = &self.operand {
            write!(buf, ", operand={}", operand.python_repr(py)?).unwrap();
        }
        if let Some(else_result) = &self.else_result {
            write!(buf, ", else_={}", else_result.python_repr(py)?).unwrap();
        }
        buf.push(')');
        Ok(buf)
    }
}

impl WriteSql for Case {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        out.push_str("CASE");
        if let Some(operand) = &self.operand {
            out.push_char(' ');
            operand.write_sql(dialect, out)?;
        }
        for (when, then) in &self.when {
            out.push_str(" WHEN ");
            when.write_sql(dialect, out)?;
            out.push_str(" THEN ");
            then.write_sql(dialect, out)?;
        }
        if let Some(else_result) = &self.else_result {
            out.push_str(" ELSE ");
            else_result.write_sql(dialect, out)?;
        }
        out.push_str(" END");
        Ok(())
    }
}

/// Escape-hatch SQL template. `{}` slots are filled by argument expressions.
/// `{{` / `}}` emit literal braces. Slot count must match [`args`](Self::args).
#[derive(Clone, Debug, PartialEq)]
pub struct Custom {
    pub sql: arcstr::ArcStr,
    pub args: Vec<ExprInner>,
}

impl Custom {
    pub(crate) const PRECEDENCE: u8 = 0;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = format!("CustomExpr({:?}", self.sql);
        for arg in &self.args {
            buf.push_str(", ");
            buf.push_str(&arg.python_repr(py)?);
        }
        buf.push(')');
        Ok(buf)
    }
}

impl WriteSql for Custom {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        write_custom(&self.sql, &self.args, dialect, out)
    }
}

#[cfg_attr(feature = "nightly", optimize(speed))]
fn write_custom<W: SqlWriter>(
    sql: &str,
    args: &[ExprInner],
    dialect: &PySqlDialect,
    out: &mut W,
) -> PrepareResult<()> {
    let bytes = sql.as_bytes();
    let mut i = 0;
    let mut start = 0;
    let mut used = 0usize;

    while i < bytes.len() {
        if bytes[i] == b'{' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                out.push_str(&sql[start..i]);
                out.push_char('{');
                i += 2;
                start = i;
                continue;
            }
            if i + 1 < bytes.len() && bytes[i + 1] == b'}' {
                out.push_str(&sql[start..i]);
                if used >= args.len() {
                    return Err(crate::sql::PrepareError::CustomPlaceholders {
                        expected: used + 1,
                        got: args.len(),
                    });
                }
                args[used].write_sql(dialect, out)?;
                used += 1;
                i += 2;
                start = i;
                continue;
            }
        } else if bytes[i] == b'}' && i + 1 < bytes.len() && bytes[i + 1] == b'}' {
            out.push_str(&sql[start..i]);
            out.push_char('}');
            i += 2;
            start = i;
            continue;
        }
        i += 1;
    }
    out.push_str(&sql[start..]);

    if used != args.len() {
        return Err(crate::sql::PrepareError::CustomPlaceholders {
            expected: used,
            got: args.len(),
        });
    }
    Ok(())
}
